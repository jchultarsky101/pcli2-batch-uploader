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

1. A **PLM system** produces an Excel manifest (`.xlsx`) listing files that have changed
2. The changed files are packaged into a **ZIP archive**
3. This tool reads the manifest, locates each file in the archive, and uploads it via **pcli2**

It is designed to be fast, scriptable, and easy to integrate into automation workflows.

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest) page.

#### macOS / Linux

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest/download/pcli2-batch-uploader-installer.sh | sh
```

#### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/jchultarsky101/pcli2-batch-uploader/releases/latest/download/pcli2-batch-uploader-installer.ps1 | iex"
```

### Build from Source

Requires [Rust](https://rustup.rs/) (edition 2024).

```sh
git clone https://github.com/jchultarsky101/pcli2-batch-uploader.git
cd pcli2-batch-uploader
cargo build --release
```

The binary will be at `target/release/pcli2-batch-uploader`.

## Usage

```
pcli2-batch-uploader [OPTIONS] --manifest <MANIFEST> --archive <ARCHIVE>

Options:
  -m, --manifest <MANIFEST>  Path to the Excel manifest file (.xlsx)
  -z, --archive <ARCHIVE>    Path to the ZIP archive containing the files
  -v, --verbose...           Enable verbose output
      --dry-run              Dry run — simulate uploads without invoking pcli2
  -h, --help                 Print help
  -V, --version              Print version
```

### Example

```sh
pcli2-batch-uploader --manifest changes.xlsx --archive parts.zip
```

### Dry Run

Preview what would be uploaded without actually invoking pcli2:

```sh
pcli2-batch-uploader --manifest changes.xlsx --archive parts.zip --dry-run
```

### Manifest Format

The Excel manifest (`.xlsx`) should have file names in the first column, with a header row:

| FileName     |
|-------------|
| part1.stp   |
| part2.stp   |
| assembly.step |

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
