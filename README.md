<p align="center">
  <h1 align="center">pcli2-batch-uploader</h1>
  <p align="center">
    A fast, reliable batch uploader CLI for Physna pcli2
    <br />
    <a href="#installation">Installation</a>
    &middot;
    <a href="#usage">Usage</a>
    &middot;
    <a href="#contributing">Contributing</a>
    &middot;
    <a href="https://github.com/jchultarsky101/pcli2-batch-uploader/issues">Report Bug</a>
  </p>
</p>

<p align="center">
  <a href="https://github.com/jchultarsky101/pcli2-batch-uploader/actions/workflows/release.yml">
    <img src="https://github.com/jchultarsky101/pcli2-batch-uploader/actions/workflows/release.yml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest">
    <img src="https://img.shields.io/github/v/release/jchultarsky101/pcli2-batch-uploader?style=flat&color=blue" alt="Latest Release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License: MIT">
  </a>
</p>

---

## Overview

**pcli2-batch-uploader** is a command-line tool for uploading files in batch to Physna using the pcli2 pipeline. It implements a crude Change Data Capture (CDC) workflow:

1. A **PLM system** produces an Excel manifest (`.xls` or `.xlsx`) listing files that have changed
2. The changed files are packaged into a **ZIP archive**
3. This tool reads the manifest, locates each Autodesk Inventor file in the archive, extracts it, and uploads it via **pcli2**

Only Inventor files (`.ipt` parts and `.iam` assemblies) are processed. All other file types in the manifest are automatically filtered out.

## Prerequisites

**pcli2** must be installed and available on your `PATH`. The batch uploader invokes `pcli2 asset create` for each file, so pcli2 must be configured and authenticated before running this tool.

## Installation

### Option 1: Pre-built Binaries (Recommended)

Pre-built binaries are available for macOS, Linux, and Windows. Download the latest release for your platform from the [Releases](https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest) page.

#### macOS (Apple Silicon and Intel)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest/download/pcli2-batch-uploader-installer.sh | sh
```

This installs the binary to `~/.cargo/bin/`. Make sure this directory is in your `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

Verify the installation:

```sh
pcli2-batch-uploader --version
```

#### Linux (x86_64 and ARM64)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest/download/pcli2-batch-uploader-installer.sh | sh
```

#### Windows

**PowerShell installer:**

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest/download/pcli2-batch-uploader-installer.ps1 | iex"
```

**MSI installer:**

Alternatively, download the `.msi` installer from the [Releases](https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest) page and run it. This provides a standard Windows installer experience.

Verify the installation:

```powershell
pcli2-batch-uploader --version
```

### Option 2: Build from Source

Requires [Rust](https://rustup.rs/) (edition 2024).

```sh
git clone https://github.com/jchultarsky101/pcli2-batch-uploader.git
cd pcli2-batch-uploader
cargo build --release
```

The binary will be at `target/release/pcli2-batch-uploader`. You can copy it to a directory in your `PATH`:

```sh
cp target/release/pcli2-batch-uploader /usr/local/bin/
```

On Windows, the binary will be at `target\release\pcli2-batch-uploader.exe`.

## Usage

```
pcli2-batch-uploader --manifest <MANIFEST> --archive <ARCHIVE> --folder <FOLDER> [OPTIONS]
```

### Arguments

| Argument | Short | Required | Description |
|----------|-------|----------|-------------|
| `--manifest` | `-m` | Yes | Path to the Excel manifest file (`.xls` or `.xlsx`) |
| `--archive` | `-z` | Yes | Path to the ZIP archive containing the files |
| `--folder` | `-f` | Yes | Physna folder path where files will be uploaded |
| `--dry-run` | | No | Simulate uploads without invoking pcli2 |
| `--verbose` | `-v` | No | Increase log verbosity (`-v` for info, `-vv` for debug, `-vvv` for trace) |
| `--help` | `-h` | No | Print help |
| `--version` | `-V` | No | Print version |

### Examples

#### Basic upload

Upload all Inventor files listed in the manifest to a Physna folder:

**macOS / Linux:**

```sh
pcli2-batch-uploader --manifest changes.xls --archive parts.zip --folder /models/dahu
```

**Windows (PowerShell):**

```powershell
pcli2-batch-uploader.exe --manifest changes.xls --archive parts.zip --folder /models/dahu
```

#### Dry run

Preview the pcli2 commands that would be executed without actually uploading anything:

```sh
pcli2-batch-uploader --manifest changes.xls --archive parts.zip --folder /models/dahu --dry-run
```

This will print the exact `pcli2 asset create` command for each file that would be uploaded, allowing you to verify the operation before committing to it. Example output:

```
📋  Manifest: 130 total rows, 68 Inventor files (.ipt/.iam), 62 skipped
📦  Archive:  38 files indexed

⚠️  65 manifest files not found in archive:
   • bracket.ipt
   • housing.iam
   • ... (remaining files listed)

🔍  Dry run — previewing 3 commands:

   ▶ pcli2 asset create --file /tmp/.tmpABC123/part1.ipt --folder-path /models/dahu --override --restore-metadata
   ▶ pcli2 asset create --file /tmp/.tmpABC123/part2.ipt --folder-path /models/dahu --override --restore-metadata
   ▶ pcli2 asset create --file /tmp/.tmpABC123/assembly.iam --folder-path /models/dahu --override --restore-metadata

   📋  3 files would be uploaded to /models/dahu
   🔍  65 not found in archive

   Dry run complete — no files were uploaded.
```

#### Verbose output

Use `-v` for debug-level logging or `-vv` for trace-level logging:

```sh
pcli2-batch-uploader --manifest changes.xls --archive parts.zip --folder /models/dahu -v
```

### Manifest Format

The Excel manifest (`.xls` or `.xlsx`) must contain a column with the header **"File Name"** (case-insensitive). The tool searches the header row to locate this column dynamically, so other columns can be present in any order.

Only Inventor files (`.ipt` and `.iam`) are processed. All other file types are automatically filtered out.

| ID | Extension | File Name | Revision | State |
|----|-----------|-----------|----------|-------|
| 101 | .ipt | part1.ipt | A | Released |
| 102 | .iam | assembly.iam | B | Released |
| 103 | .idw | drawing.idw | C | In Work |

In this example, `part1.ipt` and `assembly.iam` would be processed. The `.idw` drawing is ignored.

### How It Works

1. The manifest is parsed and filtered to Inventor files (`.ipt`, `.iam`) only
2. The ZIP archive is opened and its entries are indexed
3. For each manifest entry found in the archive, the file is extracted to a temporary directory
4. The extracted file is uploaded via `pcli2 asset create --file <path> --folder-path <folder> --override --restore-metadata`
5. A summary is printed showing the total, successful, skipped, failed, and missing counts

Files listed in the manifest but not found in the archive are printed by name so you can identify what is missing, and skipped. Upload failures for individual files do not stop the batch -- all remaining files are still processed.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
