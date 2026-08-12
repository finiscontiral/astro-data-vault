//! Astro Data Vault Database Library

use adv_core::record::{OrbitFormat, OrbitRecord, SatelliteIdRecord};
use bytes::Bytes;
use sqlx::{
    Pool, Sqlite,
    prelude::FromRow,
    sqlite::SqlitePoolOptions,
    types::chrono::{DateTime, Utc},
};

#[derive(Debug)]
pub struct Vault {
    pool: Pool<Sqlite>,
}

impl Vault {
    pub async fn try_new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new().connect(url).await?;

        Self::init_db(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_db(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        // satellite catalog id table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS satellites (
                catalog_id TEXT PRIMARY KEY,
                satellite_name TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // satellite Orbit Record
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orbits (
                db_id INTEGER PRIMARY KEY AUTOINCREMENT,
                format TEXT NOT NULL,
                catalog_id TEXT NOT NULL,
                epoch TEXT NOT NULL,
                raw_data BLOB NOT NULL,
                FOREIGN KEY (catalog_id) REFERENCES satellites(catalog_id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_satellite_id(&self, record: &SatelliteIdRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO satellites (catalog_id, satellite_name)
            VALUES (?, ?)
            ON CONFLICT(catalog_id) DO UPDATE SET satellite_name = excluded.satellite_name
            "#,
        )
        .bind(&record.catalog_id)
        .bind(&record.satellite_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_orbit(&self, record: &OrbitRecord) -> Result<usize, sqlx::Error> {
        let fmt_str = record.format.to_string();
        let raw_bytes: &[u8] = &record.raw_data;

        let result = sqlx::query(
            r#"
            INSERT INTO orbits (format, catalog_id, epoch, raw_data)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(fmt_str)
        .bind(&record.catalog_id)
        .bind(record.epoch) // sqlx は chrono::DateTime<Utc> を TEXT（ISO8601）として直接バインド可能
        .bind(raw_bytes)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid() as usize)
    }

    /// get all data
    pub async fn get_orbit_from_catalog_id(
        &self,
        catalog_id: &str,
    ) -> Result<Vec<OrbitRecord>, sqlx::Error> {
        #[derive(FromRow)]
        struct OrbitRow {
            db_id: i64,
            format: String,
            catalog_id: String,
            epoch: DateTime<Utc>,
            raw_data: Vec<u8>,
        }

        let rows = sqlx::query_as::<_, OrbitRow>(
            r#"
            SELECT db_id, format, catalog_id, epoch, raw_data 
            FROM orbits 
            WHERE catalog_id = ?
            "#,
        )
        .bind(catalog_id)
        .fetch_all(&self.pool)
        .await?;

        let records = rows
            .into_iter()
            .map(|row| {
                let format = match row.format.as_str() {
                    "OMM" => OrbitFormat::OMM,
                    _ => OrbitFormat::TLE, //todo
                };

                OrbitRecord {
                    db_id: row.db_id as usize,
                    format,
                    catalog_id: row.catalog_id,
                    epoch: row.epoch,
                    raw_data: Bytes::from(row.raw_data),
                }
            })
            .collect();

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_vault() -> Vault {
        Vault::try_new("sqlite::memory:")
            .await
            .expect("Failed to create test vault")
    }

    #[tokio::test]
    async fn test_upsert_and_get_orbit() {
        let vault = setup_test_vault().await;

        // 1.
        let sat_record = SatelliteIdRecord::new("25544".to_string(), "ISS (ZARYA)".to_string());
        vault.upsert_satellite_id(&sat_record).await.unwrap();

        // 2.
        let orbit_record = OrbitRecord::new(
            0,
            OrbitFormat::TLE,
            "25544".to_string(),
            Utc::now(),
            Bytes::from("1 25544U..."),
        );

        let inserted_id = vault.insert_orbit(&orbit_record).await.unwrap();
        assert!(inserted_id > 0);

        // 3. test
        let orbits = vault.get_orbit_from_catalog_id("25544").await.unwrap();
        assert_eq!(orbits.len(), 1);
        assert_eq!(orbits[0].catalog_id, "25544");
        assert_eq!(orbits[0].format, OrbitFormat::TLE);
        assert_eq!(orbits[0].raw_data, Bytes::from("1 25544U..."));
    }
}
