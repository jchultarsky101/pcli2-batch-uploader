use std::path::Path;
use std::process::Command;
use tracing::{debug, info};

use crate::error::Error;

/// Result of a single file upload attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum UploadResult {
    Success {
        file_name: String,
    },
    Skipped {
        file_name: String,
        reason: String,
        command: Option<String>,
    },
}

/// Uploads a file to Physna via pcli2.
pub fn upload(
    file_name: &str,
    file_path: &Path,
    folder: &str,
    dry_run: bool,
) -> Result<UploadResult, Error> {
    let file_path_str = file_path.to_str().ok_or_else(|| Error::Upload {
        name: file_name.to_string(),
        reason: "file path contains invalid UTF-8".to_string(),
    })?;

    let quote = |s: &str| -> String {
        if s.contains(' ') {
            format!("\"{s}\"")
        } else {
            s.to_string()
        }
    };

    let command = format!(
        "pcli2 asset create --file {} --folder-path {} --override --restore-metadata",
        quote(file_path_str),
        quote(folder)
    );

    if dry_run {
        info!(file_name = %file_name, command = %command, "dry run — would execute");
        return Ok(UploadResult::Skipped {
            file_name: file_name.to_string(),
            reason: "dry run".to_string(),
            command: Some(command),
        });
    }

    info!(file_name = %file_name, command = %command, "executing");

    let output = Command::new("pcli2")
        .args([
            "asset",
            "create",
            "--file",
            file_path_str,
            "--folder-path",
            folder,
            "--override",
            "--restore-metadata",
        ])
        .output()
        .map_err(|e| Error::Upload {
            name: file_name.to_string(),
            reason: format!("failed to execute pcli2: {e}"),
        })?;

    if output.status.success() {
        debug!(
            file_name = %file_name,
            stdout = %String::from_utf8_lossy(&output.stdout),
            "pcli2 output"
        );
        Ok(UploadResult::Success {
            file_name: file_name.to_string(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(Error::Upload {
            name: file_name.to_string(),
            reason: format!("pcli2 exited with {}: {}{}", output.status, stdout, stderr),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn dry_run_skips_upload() {
        let path = PathBuf::from("/tmp/test.ipt");
        let result = upload("test.ipt", &path, "/models/dahu", true).unwrap();
        assert!(matches!(result, UploadResult::Skipped { reason, .. } if reason == "dry run"));
    }
}
