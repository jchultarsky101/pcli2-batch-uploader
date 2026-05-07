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
        header.add_cell("File Name");
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

fn base_cmd(manifest: &std::path::Path, archive: &std::path::Path) -> assert_cmd::Command {
    let mut cmd = Command::cargo_bin("pcli2-batch-uploader").unwrap();
    cmd.args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", archive.to_str().unwrap()])
        .args(["--folder", "/models/test"]);
    cmd
}

#[test]
fn run_with_matching_ipt_files() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["part1.ipt", "part2.ipt"]);
    let archive = create_test_zip(&dir, &[("part1.ipt", b"data1"), ("part2.ipt", b"data2")]);

    base_cmd(&manifest, &archive)
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run complete"));
}

#[test]
fn non_inventor_files_are_filtered_out() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["drawing.idw", "assembly.iam", "part.ipt"]);
    let archive = create_test_zip(
        &dir,
        &[
            ("drawing.idw", b"d"),
            ("assembly.iam", b"a"),
            ("part.ipt", b"p"),
        ],
    );

    base_cmd(&manifest, &archive)
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("2 Inventor files"))
        .stderr(predicate::str::contains("1 skipped"));
}

#[test]
fn run_with_missing_file_in_archive() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["exists.ipt", "missing.ipt"]);
    let archive = create_test_zip(&dir, &[("exists.ipt", b"data")]);

    base_cmd(&manifest, &archive)
        .assert()
        .success()
        .stderr(predicate::str::contains("not found in archive"));
}

#[test]
fn dry_run_skips_uploads() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["part.ipt"]);
    let archive = create_test_zip(&dir, &[("part.ipt", b"data")]);

    base_cmd(&manifest, &archive)
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run complete"));
}

#[test]
fn missing_manifest_fails() {
    let dir = TempDir::new().unwrap();
    let archive = create_test_zip(&dir, &[("a.ipt", b"data")]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", "/nonexistent/manifest.xlsx"])
        .args(["--archive", archive.to_str().unwrap()])
        .args(["--folder", "/models/test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest not found"));
}

#[test]
fn missing_archive_fails() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["a.ipt"]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", "/nonexistent/archive.zip"])
        .args(["--folder", "/models/test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("archive not found"));
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
    let archive = create_test_zip(&dir, &[("a.ipt", b"data")]);

    base_cmd(&manifest, &archive)
        .assert()
        .success()
        .stderr(predicate::str::contains("Nothing to process"));
}

#[test]
fn manifest_with_only_non_inventor_files_has_nothing_to_process() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["drawing.idw", "report.pdf"]);
    let archive = create_test_zip(&dir, &[("drawing.idw", b"d"), ("report.pdf", b"r")]);

    base_cmd(&manifest, &archive)
        .assert()
        .success()
        .stderr(predicate::str::contains("Nothing to process"));
}

#[test]
fn missing_folder_arg_fails() {
    let dir = TempDir::new().unwrap();
    let manifest = create_test_xlsx(&dir, &["part.ipt"]);
    let archive = create_test_zip(&dir, &[("part.ipt", b"data")]);

    Command::cargo_bin("pcli2-batch-uploader")
        .unwrap()
        .args(["--manifest", manifest.to_str().unwrap()])
        .args(["--archive", archive.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--folder"));
}
