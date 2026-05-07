use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("manifest not found: {0}")]
    ManifestNotFound(PathBuf),

    #[error("archive not found: {0}")]
    ArchiveNotFound(PathBuf),

    #[error("failed to read manifest: {source}")]
    ManifestRead {
        #[source]
        source: calamine::XlsxError,
    },

    #[error("manifest contains no worksheets")]
    ManifestEmpty,

    #[error("failed to open archive: {source}")]
    ArchiveOpen {
        #[source]
        source: zip::result::ZipError,
    },

    #[error("failed to extract '{name}' from archive: {source}")]
    ArchiveExtract {
        name: String,
        #[source]
        source: zip::result::ZipError,
    },

    #[error("I/O error: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Self::Io { source }
    }
}
