use std::{fs, io, path::Path};

use clap::{Parser, ValueEnum};

use adv_core::record::OrbitFormat;

/// AstroDataVault CLI
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// File path
    file_path: String,

    /// Data format
    #[arg(long, short, value_enum, default_value_t = OdmFormat::Tle, ignore_case = true)]
    odm_fmt: OdmFormat,
}

impl Cli {
    fn run(&self) -> io::Result<()> {
        let f_path = Path::new(&self.file_path);

        let metadata = fs::metadata(f_path)?;

        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is not a file: {:?}", f_path),
            ));
        }

        if metadata.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("File is empty: {:?}", f_path),
            ));
        }

        let fmt = OrbitFormat::from(self.odm_fmt);

        println!("File Path: {:?}", f_path);
        println!("ODM Format: {:?}", fmt);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "upper")]
enum OdmFormat {
    Tle,
    Omm,
}

impl From<OdmFormat> for OrbitFormat {
    fn from(format: OdmFormat) -> Self {
        match format {
            OdmFormat::Tle => OrbitFormat::TLE,
            OdmFormat::Omm => OrbitFormat::OMM,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.run() {
        eprintln!("Error: {:#?}", e);
        std::process::exit(1);
    }
}
