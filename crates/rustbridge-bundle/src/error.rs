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

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn BundleError___io___displays_message() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: BundleError = io_err.into();

        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn BundleError___invalid_manifest___displays_message() {
        let err = BundleError::InvalidManifest("missing name".to_string());

        assert_eq!(err.to_string(), "Invalid manifest: missing name");
    }

    #[test]
    fn BundleError___checksum_mismatch___displays_all_fields() {
        let err = BundleError::ChecksumMismatch {
            path: "lib/test.so".to_string(),
            expected: "sha256:expected".to_string(),
            actual: "sha256:actual".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("lib/test.so"));
        assert!(msg.contains("sha256:expected"));
        assert!(msg.contains("sha256:actual"));
    }

    #[test]
    fn BundleError___unsupported_platform___displays_platform() {
        let err = BundleError::UnsupportedPlatform("linux-arm".to_string());

        assert_eq!(err.to_string(), "Platform not supported: linux-arm");
    }

    #[test]
    fn BundleError___missing_file___displays_path() {
        let err = BundleError::MissingFile("manifest.json".to_string());

        assert_eq!(err.to_string(), "Missing required file: manifest.json");
    }

    #[test]
    fn BundleError___library_not_found___displays_path() {
        let err = BundleError::LibraryNotFound("/path/to/lib.so".to_string());

        assert_eq!(err.to_string(), "Library not found: /path/to/lib.so");
    }

    #[test]
    fn BundleError___from_io_error___converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let bundle_err: BundleError = io_err.into();

        assert!(matches!(bundle_err, BundleError::Io(_)));
    }
}
