# Contributing to pcli2-batch-uploader

Thank you for your interest in contributing! This document provides guidelines to make the process smooth for everyone.

## Getting Started

1. Fork the repository
2. Clone your fork and create a feature branch from `develop`:
   ```sh
   git clone https://github.com/<your-username>/pcli2-batch-uploader.git
   cd pcli2-batch-uploader
   git checkout develop
   git checkout -b feature/your-feature-name
   ```
3. Make your changes
4. Ensure the project builds and tests pass:
   ```sh
   cargo build
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```
5. Commit your changes with a clear, descriptive message
6. Push to your fork and open a Pull Request against `develop`

## Branching Model

This project follows **Git Flow**:

| Branch | Purpose |
|---|---|
| `main` | Production releases, tagged with semver |
| `develop` | Integration branch for the next release |
| `feature/*` | New features, branched from `develop` |
| `release/*` | Release preparation, merged into `main` and `develop` |
| `hotfix/*` | Urgent production fixes, merged into `main` and `develop` |

- All feature PRs target `develop`, **not** `main`.
- Branch names should be descriptive: `feature/add-retry-logic`, `hotfix/fix-auth-header`.

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` and fix all warnings
- Write tests for new functionality
- Keep commits focused and atomic

## Pull Request Guidelines

- Provide a clear description of what the PR does and why
- Reference any related issues
- Keep PRs focused on a single concern
- Ensure CI passes before requesting review

## Reporting Bugs

Open an issue on [GitHub Issues](https://github.com/jchultarsky101/pcli2-batch-uploader/issues) with:

- A clear title and description
- Steps to reproduce
- Expected vs. actual behavior
- Your environment (OS, Rust version)

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
