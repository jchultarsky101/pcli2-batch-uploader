use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::{debug, info};
use zip::ZipArchive;

use crate::error::Error;

/// An opened ZIP archive ready for file extraction.
#[derive(Debug)]
pub struct Archive {
    inner: ZipArchive<File>,
    file_names: HashSet<String>,
}

impl Archive {
    /// Opens a ZIP archive from the given path.
    pub fn open(path: &Path) -> Result<Self, Error> {
        if !path.exists() {
            return Err(Error::ArchiveNotFound(path.to_path_buf()));
        }

        info!(path = %path.display(), "opening archive");

        let file = File::open(path)?;
        let inner = ZipArchive::new(file).map_err(|source| Error::ArchiveOpen { source })?;

        let file_names: HashSet<String> = (0..inner.len())
            .filter_map(|i| inner.name_for_index(i).map(|n| n.to_string()))
            .collect();

        info!(count = file_names.len(), "indexed archive entries");
        for name in &file_names {
            debug!(name = %name, "archive entry");
        }

        Ok(Self { inner, file_names })
    }

    /// Returns `true` if the archive contains a file with the given name.
    ///
    /// Matches by file name only (ignoring directory prefixes in the archive).
    pub fn contains(&self, file_name: &str) -> bool {
        self.file_names.contains(file_name)
            || self
                .file_names
                .iter()
                .any(|n| n.ends_with(&format!("/{file_name}")))
    }

    /// Resolves the full archive path for a given file name.
    fn resolve_name(&self, file_name: &str) -> Option<String> {
        if self.file_names.contains(file_name) {
            return Some(file_name.to_string());
        }
        self.file_names
            .iter()
            .find(|n| n.ends_with(&format!("/{file_name}")))
            .cloned()
    }

    /// Extracts the contents of a file from the archive.
    pub fn extract(&mut self, file_name: &str) -> Result<Vec<u8>, Error> {
        let resolved = self
            .resolve_name(file_name)
            .ok_or_else(|| Error::ArchiveExtract {
                name: file_name.to_string(),
                source: zip::result::ZipError::FileNotFound,
            })?;

        debug!(name = %resolved, "extracting file");

        let mut entry = self
            .inner
            .by_name(&resolved)
            .map_err(|source| Error::ArchiveExtract {
                name: file_name.to_string(),
                source,
            })?;

        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Returns the number of files in the archive.
    pub fn len(&self) -> usize {
        self.file_names.len()
    }

    /// Returns `true` if the archive contains no files.
    pub fn is_empty(&self) -> bool {
        self.file_names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use zip::write::SimpleFileOptions;

    fn create_test_zip(entries: &[(&str, &[u8])]) -> tempfile::NamedTempFile {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(tmp.as_file().try_clone().unwrap());
        let options = SimpleFileOptions::default();
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
        tmp
    }

    #[test]
    fn missing_archive_returns_error() {
        let path = PathBuf::from("/nonexistent/archive.zip");
        let result = Archive::open(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ArchiveNotFound(_)));
    }

    #[test]
    fn open_and_list_entries() {
        let zip = create_test_zip(&[("a.stp", b"data_a"), ("b.stp", b"data_b")]);
        let archive = Archive::open(zip.path()).unwrap();
        assert_eq!(archive.len(), 2);
        assert!(archive.contains("a.stp"));
        assert!(archive.contains("b.stp"));
        assert!(!archive.contains("c.stp"));
    }

    #[test]
    fn extract_file_contents() {
        let zip = create_test_zip(&[("part.stp", b"step_data")]);
        let mut archive = Archive::open(zip.path()).unwrap();
        let data = archive.extract("part.stp").unwrap();
        assert_eq!(data, b"step_data");
    }

    #[test]
    fn extract_missing_file_returns_error() {
        let zip = create_test_zip(&[("a.stp", b"data")]);
        let mut archive = Archive::open(zip.path()).unwrap();
        let result = archive.extract("missing.stp");
        assert!(result.is_err());
    }

    #[test]
    fn contains_matches_nested_paths() {
        let zip = create_test_zip(&[("subdir/part.stp", b"data")]);
        let archive = Archive::open(zip.path()).unwrap();
        assert!(archive.contains("part.stp"));
    }

    #[test]
    fn extract_nested_file_by_name() {
        let zip = create_test_zip(&[("subdir/part.stp", b"nested_data")]);
        let mut archive = Archive::open(zip.path()).unwrap();
        let data = archive.extract("part.stp").unwrap();
        assert_eq!(data, b"nested_data");
    }
}
