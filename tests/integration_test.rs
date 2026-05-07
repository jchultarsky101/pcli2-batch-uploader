use assert_cmd::Command;
use predicates::prelude::*;
use simple_excel_writer::{Row, Workbook};
use std::io::Write;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;

fn create_test_xlsx(dir: &TempDir, entries: &[&str]) -> std::path::PathBuf {
    let path = dir.path().join("manifest.xlsx");
    let mut wb = Workbook::create(path.to_str().unwrap());
    let mut sheet = wb.create_sheet("Sheet1");

    wb.write_sheet(&mut sheet, |sw| {
        let mut header = Row::new();
        header.add_cell("FileName");
        sw.append_row(header)?;

        for name in entries {
            let mut row = Row::new();
            row.add_cell(*name);
            sw.append_row(row)?;
        }
        Ok(())
    })
    .unwrap();

    wb.close().unwrap();
    path
}

fn create_test_zip(dir: &TempDir, entries: &[(&str, &[u8])]) -> std::path::PathBuf {
    let path = dir.path().join("archive.zip");
    let file = std::fs::File::create(&path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default();
    for (name, content) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(content).unwrap();
    }
    writer.finish().unwrap();
    path
}

#[test]
fn run_with_matching_files() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["part1.stp", "part2.stp"]);
    let archive = create_test_zip(&dir, &[("part1.stp", b"data1"), ("part2.stp", b"data2")]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("batch upload complete"));
}

#[test]
fn run_with_missing_file_in_archive() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["exists.stp", "missing.stp"]);
    let archive = create_test_zip(&dir, &[("exists.stp", b"data")]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("not found in archive"));
}

#[test]
fn dry_run_skips_uploads() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["part.stp"]);
    let archive = create_test_zip(&dir, &[("part.stp", b"data")]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", archive.to_str().unwrap()])
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));
}

#[test]
fn missing_manifest_fails() {
    let dir = TempDir::new().unwrap();
    let archive = create_test_zip(&dir, &[("a.stp", b"data")]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", "/nonexistent/manifest.xlsx"])
        .args(["--archive", archive.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("manifest not found"));
}

#[test]
fn missing_archive_fails() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["a.stp"]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", "/nonexistent/archive.zip"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("archive not found"));
}

#[test]
fn no_args_shows_help() {
    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn run_with_empty_manifest() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &[]);
    let archive = create_test_zip(&dir, &[("a.stp", b"data")]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to process"));
}
