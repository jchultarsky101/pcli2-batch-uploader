use calamine::{Data, Range, Reader, Xls, Xlsx, open_workbook};
use std::path::Path;
use tracing::{debug, info, warn};

use crate::error::Error;

/// A single entry from the upload manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    pub file_name: String,
}

/// Result of parsing the manifest, including filtering stats.
#[derive(Debug)]
pub struct ManifestResult {
    pub entries: Vec<ManifestEntry>,
    pub total_rows: usize,
    pub skipped: usize,
}

const FILE_NAME_HEADER: &str = "File Name";
const SUPPORTED_EXTENSIONS: &[&str] = &[".ipt", ".iam"];

/// Parses an Excel manifest and returns the list of files to upload.
///
/// Supports both `.xlsx` and `.xls` formats. Tries the format matching the
/// file extension first, then falls back to the other format.
/// Locates the "File Name" column in the header row and extracts entries from it.
pub fn parse_manifest(path: &Path) -> Result<ManifestResult, Error> {
    if !path.exists() {
        return Err(Error::ManifestNotFound(path.to_path_buf()));
    }

    info!(path = %path.display(), "reading manifest");

    let (sheet_names, range) = open_first_sheet(path)?;
    let first_sheet = sheet_names.first().ok_or(Error::ManifestEmpty)?;
    debug!(sheet = %first_sheet, "using first worksheet");

    extract_entries(&range)
}

fn open_first_sheet(path: &Path) -> Result<(Vec<String>, Range<Data>), Error> {
    if let Ok(result) = try_open_xlsx(path) {
        return Ok(result);
    }
    if let Ok(result) = try_open_xls(path) {
        return Ok(result);
    }
    Err(Error::ManifestRead {
        source: calamine::Error::Msg("unable to open as .xlsx or .xls"),
    })
}

fn try_open_xlsx(path: &Path) -> Result<(Vec<String>, Range<Data>), Error> {
    let mut wb: Xlsx<_> = open_workbook(path)
        .map_err(|e: calamine::XlsxError| Error::ManifestRead { source: e.into() })?;
    let names = wb.sheet_names().to_vec();
    let first = names.first().ok_or(Error::ManifestEmpty)?.clone();
    let range = wb
        .worksheet_range(&first)
        .map_err(|e: calamine::XlsxError| Error::ManifestRead { source: e.into() })?;
    Ok((names, range))
}

fn try_open_xls(path: &Path) -> Result<(Vec<String>, Range<Data>), Error> {
    let mut wb: Xls<_> = open_workbook(path)
        .map_err(|e: calamine::XlsError| Error::ManifestRead { source: e.into() })?;
    let names = wb.sheet_names().to_vec();
    let first = names.first().ok_or(Error::ManifestEmpty)?.clone();
    let range = wb
        .worksheet_range(&first)
        .map_err(|e: calamine::XlsError| Error::ManifestRead { source: e.into() })?;
    Ok((names, range))
}

fn find_file_name_column(header: &[Data]) -> Option<usize> {
    header
        .iter()
        .position(|cell| cell.to_string().eq_ignore_ascii_case(FILE_NAME_HEADER))
}

fn extract_entries(range: &Range<Data>) -> Result<ManifestResult, Error> {
    let mut rows = range.rows();

    let header = rows.next().ok_or(Error::ManifestEmpty)?;
    let col_idx = find_file_name_column(header).ok_or(Error::ManifestColumnNotFound {
        column: FILE_NAME_HEADER.to_string(),
    })?;
    debug!(column = col_idx, "found '{}' column", FILE_NAME_HEADER);

    let mut entries = Vec::new();
    let mut total_rows = 0usize;
    let mut skipped = 0usize;

    for (row_idx, row) in rows.enumerate() {
        let file_name = match row.get(col_idx) {
            Some(cell) => {
                let value = cell.to_string();
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    warn!(row = row_idx + 2, "skipping empty row");
                    continue;
                }
                trimmed
            }
            None => {
                warn!(row = row_idx + 2, "skipping row with missing column");
                continue;
            }
        };

        total_rows += 1;

        let lower = file_name.to_lowercase();
        if !SUPPORTED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext)) {
            debug!(row = row_idx + 2, file_name = %file_name, "skipping unsupported file type");
            skipped += 1;
            continue;
        }

        debug!(row = row_idx + 2, file_name = %file_name, "found entry");
        entries.push(ManifestEntry { file_name });
    }

    info!(
        count = entries.len(),
        "parsed manifest entries (filtered to Inventor files)"
    );
    Ok(ManifestResult {
        entries,
        total_rows,
        skipped,
    })
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

    #[test]
    fn find_column_by_header() {
        let header = vec![
            Data::String("ID".to_string()),
            Data::String("Extension".to_string()),
            Data::String("File Name".to_string()),
        ];
        assert_eq!(find_file_name_column(&header), Some(2));
    }

    #[test]
    fn find_column_case_insensitive() {
        let header = vec![Data::String("file name".to_string())];
        assert_eq!(find_file_name_column(&header), Some(0));
    }

    #[test]
    fn find_column_missing() {
        let header = vec![Data::String("Something".to_string())];
        assert_eq!(find_file_name_column(&header), None);
    }

    #[test]
    fn extract_entries_filters_to_inventor_files() {
        let range = Range::from_sparse(vec![
            calamine::Cell::new((0, 0), Data::String("File Name".to_string())),
            calamine::Cell::new((1, 0), Data::String("part.ipt".to_string())),
            calamine::Cell::new((2, 0), Data::String("drawing.idw".to_string())),
            calamine::Cell::new((3, 0), Data::String("assembly.iam".to_string())),
            calamine::Cell::new((4, 0), Data::String("other.ipt".to_string())),
        ]);
        let result = extract_entries(&range).unwrap();
        assert_eq!(result.entries.len(), 3);
        assert_eq!(result.entries[0].file_name, "part.ipt");
        assert_eq!(result.entries[1].file_name, "assembly.iam");
        assert_eq!(result.entries[2].file_name, "other.ipt");
        assert_eq!(result.total_rows, 4);
        assert_eq!(result.skipped, 1);
    }
}
