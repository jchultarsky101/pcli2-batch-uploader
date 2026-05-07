mod archive;
mod cli;
mod error;
mod manifest;
mod uploader;

use std::fs;
use std::path::Path;

use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, error, info, warn};

use crate::archive::Archive;
use crate::cli::Cli;
use crate::manifest::parse_manifest;
use crate::uploader::{UploadResult, upload};

fn main() {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "off",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    if let Err(e) = run(&cli) {
        eprintln!("\n{} {e}", style("  Error:").red().bold());
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), error::Error> {
    let manifest = parse_manifest(&cli.manifest)?;
    let entries = manifest.entries;
    let mut archive = Archive::open(&cli.archive)?;

    eprintln!(
        "\n{}  Manifest: {} total rows, {} Inventor files (.ipt/.iam), {} skipped",
        style("\u{1f4cb}").dim(),
        style(manifest.total_rows).cyan().bold(),
        style(entries.len()).cyan().bold(),
        style(manifest.skipped).dim()
    );
    eprintln!(
        "{}  Archive:  {} files indexed",
        style("\u{1f4e6}").dim(),
        style(archive.len()).cyan().bold()
    );

    info!(
        archive_files = archive.len(),
        manifest_entries = entries.len(),
        "processing batch"
    );

    if entries.is_empty() || archive.is_empty() {
        eprintln!("\n{}  Nothing to process.", style("\u{2139}\u{fe0f}").dim());
        return Ok(());
    }

    let temp_dir = tempfile::tempdir()?;
    debug!(path = %temp_dir.path().display(), "created temp directory for extraction");

    let mut success_count = 0u32;
    let mut skip_count = 0u32;
    let mut fail_count = 0u32;
    let uploadable: Vec<_> = entries
        .iter()
        .filter(|e| archive.contains(&e.file_name))
        .collect();
    let missing: Vec<_> = entries
        .iter()
        .filter(|e| !archive.contains(&e.file_name))
        .collect();

    let missing_count = missing.len() as u32;
    for entry in &missing {
        warn!(file_name = %entry.file_name, "not found in archive");
    }

    if missing_count > 0 {
        eprintln!(
            "{}  {} files not found in archive",
            style("\u{26a0}\u{fe0f}").dim(),
            style(missing_count).yellow().bold()
        );
    }

    if uploadable.is_empty() {
        eprintln!(
            "\n{}  No matching files to upload.",
            style("\u{2139}\u{fe0f}").dim()
        );
        print_summary(
            &cli.folder,
            cli.dry_run,
            success_count,
            skip_count,
            fail_count,
            missing_count,
            entries.len(),
        );
        return Ok(());
    }

    if cli.dry_run {
        eprintln!(
            "\n{}  Dry run \u{2014} previewing {} commands:\n",
            style("\u{1f50d}").dim(),
            style(uploadable.len()).cyan().bold()
        );
    } else {
        eprintln!(
            "\n{}  Uploading {} files to {}...\n",
            style("\u{1f680}").dim(),
            style(uploadable.len()).cyan().bold(),
            style(&cli.folder).green()
        );
    }

    let pb = if cli.dry_run {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(uploadable.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("   {bar:40.cyan/dim} {pos}/{len}  {msg}")
                .unwrap()
                .progress_chars("\u{2588}\u{2592}\u{2591}"),
        );
        pb
    };

    for entry in &uploadable {
        pb.set_message(entry.file_name.clone());

        let data = archive.extract(&entry.file_name)?;
        let file_path = temp_dir.path().join(&entry.file_name);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &data)?;

        match upload_file(&entry.file_name, &file_path, &cli.folder, cli.dry_run) {
            Ok(UploadResult::Success { file_name }) => {
                info!(file_name = %file_name, "uploaded successfully");
                success_count += 1;
            }
            Ok(UploadResult::Skipped {
                file_name,
                reason,
                command,
            }) => {
                info!(file_name = %file_name, reason = %reason, "skipped");
                if let Some(cmd) = command {
                    eprintln!("   {} {}", style("\u{25b6}").cyan(), style(&cmd).dim());
                }
                skip_count += 1;
            }
            Err(e) => {
                error!(file_name = %entry.file_name, error = %e, "upload failed");
                pb.suspend(|| {
                    eprintln!(
                        "   {} {} \u{2014} {}",
                        style("\u{274c}").red(),
                        entry.file_name,
                        e
                    );
                });
                fail_count += 1;
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    print_summary(
        &cli.folder,
        cli.dry_run,
        success_count,
        skip_count,
        fail_count,
        missing_count,
        entries.len(),
    );

    Ok(())
}

fn print_summary(
    folder: &str,
    dry_run: bool,
    success: u32,
    skipped: u32,
    failed: u32,
    missing: u32,
    total: usize,
) {
    eprintln!();

    if dry_run {
        eprintln!(
            "   {}  {} files would be uploaded to {}",
            style("\u{1f4cb}").dim(),
            style(skipped).cyan().bold(),
            style(folder).green()
        );
    } else if success > 0 {
        eprintln!(
            "   {}  {} uploaded successfully",
            style("\u{2705}").dim(),
            style(success).green().bold()
        );
    }

    if failed > 0 {
        eprintln!(
            "   {}  {} failed",
            style("\u{274c}").dim(),
            style(failed).red().bold()
        );
    }

    if missing > 0 {
        eprintln!(
            "   {}  {} not found in archive",
            style("\u{1f50d}").dim(),
            style(missing).yellow().bold()
        );
    }

    eprintln!();

    if failed > 0 {
        eprintln!(
            "   {}",
            style(format!(
                "Completed with errors ({failed} of {total} failed)."
            ))
            .red()
        );
    } else if dry_run {
        eprintln!(
            "   {}",
            style("Dry run complete \u{2014} no files were uploaded.").dim()
        );
    } else {
        eprintln!(
            "   {} {}",
            style("\u{1f389}").dim(),
            style("Batch upload complete!").green().bold()
        );
    }

    eprintln!();
}

fn upload_file(
    file_name: &str,
    file_path: &Path,
    folder: &str,
    dry_run: bool,
) -> Result<UploadResult, error::Error> {
    upload(file_name, file_path, folder, dry_run)
}
