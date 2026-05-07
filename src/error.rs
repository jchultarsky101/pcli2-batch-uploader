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
        source: calamine::Error,
    },

    #[error("manifest contains no worksheets")]
    ManifestEmpty,

    #[error("manifest missing required column: '{column}'")]
    ManifestColumnNotFound { column: String },

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

    #[error("upload failed for '{name}': {reason}")]
    Upload { name: String, reason: String },

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
