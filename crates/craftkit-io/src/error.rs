//! Errors returned by the I/O layer.

use std::path::PathBuf;

/// Failure while reading, validating or writing an application file.
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    /// The operating system refused a read, write or rename.
    #[error("I/O error on {path}: {source}")]
    Io {
        /// File or directory involved.
        path: PathBuf,
        /// Underlying operating-system error.
        #[source]
        source: std::io::Error,
    },
    /// The file is not valid JSON for the expected document type.
    #[error("invalid project JSON in {path}: {source}")]
    Json {
        /// File being parsed.
        path: PathBuf,
        /// Serde error with line and column information.
        #[source]
        source: serde_json::Error,
    },
    /// The file belongs to another application or uses a newer schema.
    #[error(transparent)]
    Format(#[from] craftkit_core::ProjectFormatError),
    /// The document has never been saved and no path is known.
    #[error("no file path yet; use Save As")]
    NoPath,
}

pub(crate) fn io_error(path: &std::path::Path, source: std::io::Error) -> IoError {
    IoError::Io {
        path: path.to_path_buf(),
        source,
    }
}

pub(crate) fn json_error(path: &std::path::Path, source: serde_json::Error) -> IoError {
    IoError::Json {
        path: path.to_path_buf(),
        source,
    }
}
