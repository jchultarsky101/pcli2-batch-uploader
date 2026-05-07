mod archive;
mod cli;
mod error;
mod manifest;
mod uploader;

use clap::Parser;
use tracing::{error, info, warn};

use crate::archive::Archive;
use crate::cli::Cli;
use crate::manifest::parse_manifest;
use crate::uploader::{UploadResult, upload};

fn main() {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    if let Err(e) = run(&cli) {
        error!("{e}");
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), error::Error> {
    let entries = parse_manifest(&cli.manifest)?;
    let mut archive = Archive::open(&cli.archive)?;

    info!(
        archive_files = archive.len(),
        manifest_entries = entries.len(),
        "processing batch"
    );

    if entries.is_empty() || archive.is_empty() {
        info!("nothing to process");
        return Ok(());
    }

    let mut success_count = 0u32;
    let mut skip_count = 0u32;
    let mut missing_count = 0u32;

    for entry in &entries {
        if !archive.contains(&entry.file_name) {
            warn!(file_name = %entry.file_name, "not found in archive — skipping");
            missing_count += 1;
            continue;
        }

        let data = archive.extract(&entry.file_name)?;

        match upload(&entry.file_name, &data, cli.dry_run)? {
            UploadResult::Success { file_name } => {
                info!(file_name = %file_name, "uploaded successfully");
                success_count += 1;
            }
            UploadResult::Skipped { file_name, reason } => {
                info!(file_name = %file_name, reason = %reason, "skipped");
                skip_count += 1;
            }
        }
    }

    info!(
        total = entries.len(),
        success = success_count,
        skipped = skip_count,
        missing = missing_count,
        "batch upload complete"
    );

    Ok(())
}
