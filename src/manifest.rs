use calamine::{Reader, Xlsx, open_workbook};
use std::path::Path;
use tracing::{debug, info, warn};

use crate::error::Error;

/// A single entry from the upload manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    pub file_name: String,
}

/// Parses an Excel manifest and returns the list of files to upload.
///
/// Reads the first worksheet and treats the first column of each row as a
/// file name. The first row is assumed to be a header and is skipped.
pub fn parse_manifest(path: &Path) -> Result<Vec<ManifestEntry>, Error> {
    if !path.exists() {
        return Err(Error::ManifestNotFound(path.to_path_buf()));
    }

    info!(path = %path.display(), "reading manifest");

    let mut workbook: Xlsx<_> =
        open_workbook(path).map_err(|source| Error::ManifestRead { source })?;

    let sheet_names = workbook.sheet_names().to_vec();
    let first_sheet = sheet_names.first().ok_or(Error::ManifestEmpty)?;

    debug!(sheet = %first_sheet, "using first worksheet");

    let range = workbook
        .worksheet_range(first_sheet)
        .map_err(|source| Error::ManifestRead { source })?;

    let mut entries = Vec::new();

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            debug!("skipping header row");
            continue;
        }

        let file_name = match row.first() {
            Some(cell) => {
                let value = cell.to_string();
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    warn!(row = row_idx + 1, "skipping empty row");
                    continue;
                }
                trimmed
            }
            None => {
                warn!(row = row_idx + 1, "skipping row with no columns");
                continue;
            }
        };

        debug!(row = row_idx + 1, file_name = %file_name, "found entry");
        entries.push(ManifestEntry { file_name });
    }

    info!(count = entries.len(), "parsed manifest entries");
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn missing_manifest_returns_error() {
        let path = PathBuf::from("/nonexistent/manifest.xlsx");
        let result = parse_manifest(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ManifestNotFound(_)));
    }
}
