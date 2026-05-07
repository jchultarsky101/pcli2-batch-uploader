use tracing::{info, warn};

use crate::error::Error;

/// Result of a single file upload attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum UploadResult {
    Success { file_name: String },
    Skipped { file_name: String, reason: String },
}

/// Uploads file data via pcli2.
///
/// Currently simulates the upload. Will be replaced with an actual pcli2
/// invocation in a future iteration.
pub fn upload(file_name: &str, _data: &[u8], dry_run: bool) -> Result<UploadResult, Error> {
    if dry_run {
        info!(file_name = %file_name, "dry run — skipping upload");
        return Ok(UploadResult::Skipped {
            file_name: file_name.to_string(),
            reason: "dry run".to_string(),
        });
    }

    // TODO: Replace with actual pcli2 CLI invocation
    info!(file_name = %file_name, "[simulated] uploading via pcli2");
    warn!(file_name = %file_name, "pcli2 integration not yet implemented — simulating success");

    Ok(UploadResult::Success {
        file_name: file_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_skips_upload() {
        let result = upload("test.stp", b"data", true).unwrap();
        assert_eq!(
            result,
            UploadResult::Skipped {
                file_name: "test.stp".to_string(),
                reason: "dry run".to_string(),
            }
        );
    }

    #[test]
    fn simulated_upload_succeeds() {
        let result = upload("test.stp", b"data", false).unwrap();
        assert_eq!(
            result,
            UploadResult::Success {
                file_name: "test.stp".to_string(),
            }
        );
    }
}
