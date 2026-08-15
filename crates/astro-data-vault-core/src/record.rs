use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod tle;
pub use tle::Tle;

/// Data Format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrbitFormat {
    TLE,
    OMM,
}

impl std::fmt::Display for OrbitFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OrbitFormat::TLE => "TLE",
            OrbitFormat::OMM => "OMM",
        };
        write!(f, "{}", s)
    }
}

/// Satellite ID record.
///
/// Since satellite names correspond to satellite IDs,
/// `catalog_id` is used as the primary key.
///
/// ## Note
/// Equality and ordering for this struct are determined solely
/// by `catalog_id`, ignoring `satellite_name`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatelliteIdRecord {
    /// Satellite catalog id
    pub catalog_id: String,

    /// Satellite name
    pub satellite_name: String,
}

impl SatelliteIdRecord {
    pub fn new(catalog_id: String, satellite_name: String) -> Self {
        Self {
            catalog_id,
            satellite_name,
        }
    }
}

impl Ord for SatelliteIdRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.catalog_id.cmp(&other.catalog_id)
    }
}

impl PartialOrd for SatelliteIdRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SatelliteIdRecord {}

impl PartialEq for SatelliteIdRecord {
    fn eq(&self, other: &Self) -> bool {
        self.catalog_id == other.catalog_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrbitRecord {
    /// DB main key
    pub db_id: usize,

    pub format: OrbitFormat,

    /// satellite catalog id
    pub catalog_id: String,

    pub epoch: DateTime<Utc>,

    /// raw binary
    pub raw_data: Bytes,
}

impl OrbitRecord {
    pub fn new(
        db_id: usize,
        format: OrbitFormat,
        catalog_id: String,
        epoch: DateTime<Utc>,
        raw_data: Bytes,
    ) -> Self {
        Self {
            db_id,
            format,
            catalog_id,
            epoch,
            raw_data,
        }
    }
}

impl Ord for OrbitRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.db_id.cmp(&other.db_id)
    }
}

impl PartialOrd for OrbitRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
