//! Error types for bundle operations.

use thiserror::Error;

/// Errors that can occur during bundle operations.
#[derive(Debug, Error)]
pub enum BundleError {
    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing or serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// ZIP archive error.
    #[error("Archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Manifest validation error.
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// Checksum mismatch.
    #[error("Checksum mismatch for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    /// Platform not supported in this bundle.
    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    /// Missing required file in bundle.
    #[error("Missing required file: {0}")]
    MissingFile(String),

    /// Library file not found.
    #[error("Library not found: {0}")]
    LibraryNotFound(String),
}
