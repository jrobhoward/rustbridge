//! Error types for rustbridge plugins

use thiserror::Error;

/// Result type alias for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Error type for plugin operations
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin is not in a valid state for the requested operation
    #[error("invalid lifecycle state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    /// Failed to initialize the plugin
    #[error("initialization failed: {0}")]
    InitializationFailed(String),

    /// Failed to shutdown the plugin
    #[error("shutdown failed: {0}")]
    ShutdownFailed(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Serialization/deserialization error
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Unknown message type tag
    #[error("unknown message type: {0}")]
    UnknownMessageType(String),

    /// Handler returned an error
    #[error("handler error: {0}")]
    HandlerError(String),

    /// Async runtime error
    #[error("runtime error: {0}")]
    RuntimeError(String),

    /// Request was cancelled
    #[error("request cancelled")]
    Cancelled,

    /// Request timed out
    #[error("request timed out")]
    Timeout,

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),

    /// FFI error
    #[error("FFI error: {0}")]
    FfiError(String),
}

impl PluginError {
    /// Returns an error code suitable for FFI
    pub fn error_code(&self) -> u32 {
        match self {
            PluginError::InvalidState { .. } => 1,
            PluginError::InitializationFailed(_) => 2,
            PluginError::ShutdownFailed(_) => 3,
            PluginError::ConfigError(_) => 4,
            PluginError::SerializationError(_) => 5,
            PluginError::UnknownMessageType(_) => 6,
            PluginError::HandlerError(_) => 7,
            PluginError::RuntimeError(_) => 8,
            PluginError::Cancelled => 9,
            PluginError::Timeout => 10,
            PluginError::Internal(_) => 11,
            PluginError::FfiError(_) => 12,
        }
    }

    /// Create an error from an error code and message (for FFI deserialization)
    pub fn from_code(code: u32, message: String) -> Self {
        match code {
            1 => PluginError::InvalidState {
                expected: String::new(),
                actual: message,
            },
            2 => PluginError::InitializationFailed(message),
            3 => PluginError::ShutdownFailed(message),
            4 => PluginError::ConfigError(message),
            5 => PluginError::SerializationError(message),
            6 => PluginError::UnknownMessageType(message),
            7 => PluginError::HandlerError(message),
            8 => PluginError::RuntimeError(message),
            9 => PluginError::Cancelled,
            10 => PluginError::Timeout,
            12 => PluginError::FfiError(message),
            _ => PluginError::Internal(message),
        }
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
#[path = "error/error_tests.rs"]
mod error_tests;
