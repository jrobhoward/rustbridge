//! Error types for JNI operations.

use thiserror::Error;

/// Errors that can occur during JNI operations.
#[derive(Debug, Error)]
pub enum JniError {
    /// Failed to convert a Java string to Rust.
    #[error("String conversion failed: {0}")]
    StringConversion(String),

    /// Failed to access a Java array.
    #[error("Array access failed: {0}")]
    ArrayAccess(String),

    /// Failed to load the plugin library.
    #[error("Failed to load library: {0}")]
    LibraryLoad(String),

    /// Failed to find a required symbol in the library.
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    /// Plugin initialization failed.
    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    /// Plugin call failed.
    #[error("Plugin call failed: {message}")]
    PluginCall { code: u32, message: String },

    /// Failed to parse JSON.
    #[error("JSON parse error: {0}")]
    JsonParse(String),
}

impl JniError {
    /// Get the error code for this error.
    ///
    /// These codes match the rustbridge error codes where applicable.
    pub fn code(&self) -> u32 {
        match self {
            JniError::StringConversion(_) => 4, // InvalidRequest
            JniError::ArrayAccess(_) => 4,      // InvalidRequest
            JniError::LibraryLoad(_) => 1,      // InvalidHandle
            JniError::SymbolNotFound(_) => 1,   // InvalidHandle
            JniError::InitFailed(_) => 11,      // InternalError
            JniError::PluginCall { code, .. } => *code,
            JniError::JsonParse(_) => 5, // SerializationError
        }
    }
}
