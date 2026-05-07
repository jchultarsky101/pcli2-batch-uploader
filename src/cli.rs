use clap::Parser;
use std::path::PathBuf;

/// Batch uploader for Physna pcli2.
///
/// Reads an Excel manifest (from a PLM system) listing files to upload,
/// locates each file in a ZIP archive, and uploads it via pcli2.
#[derive(Debug, Parser)]
#[command(name = "pcli2-batch-uploader", version, about)]
pub struct Cli {
    /// Path to the Excel manifest file (.xlsx)
    #[arg(short, long)]
    pub manifest: PathBuf,

    /// Path to the ZIP archive containing the files
    #[arg(short = 'z', long)]
    pub archive: PathBuf,

    /// Physna folder path for uploading files
    #[arg(short, long)]
    pub folder: String,

    /// Enable verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Dry run — simulate uploads without invoking pcli2
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
