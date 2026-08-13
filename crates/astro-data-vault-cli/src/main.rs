use clap::{Parser, ValueEnum};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "upper")]
enum OdmFormat {
    Tle,
    Omm,
}

fn main() {
    let cli = Cli::parse();

    println!("File Path: {}", cli.file_path);
    println!("ODM Format: {:?}", cli.odm_fmt);
}
