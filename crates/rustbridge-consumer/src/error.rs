//! Error types for the consumer crate.

use rustbridge_core::LifecycleState;
use thiserror::Error;

/// Errors that can occur when loading or calling plugins.
#[derive(Error, Debug)]
pub enum ConsumerError {
    /// Failed to load the shared library.
    #[error("failed to load library: {0}")]
    LibraryLoad(#[from] libloading::Error),

    /// Plugin initialization failed.
    #[error("plugin initialization failed: {0}")]
    InitFailed(String),

    /// Plugin call failed.
    #[error("plugin call failed: {0}")]
    CallFailed(#[from] rustbridge_core::PluginError),

    /// Bundle loading error.
    #[error("bundle error: {0}")]
    Bundle(#[from] rustbridge_bundle::BundleError),

    /// Failed to parse response from plugin.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// Plugin is not in Active state.
    #[error("plugin not active (state: {0:?})")]
    NotActive(LifecycleState),

    /// Plugin initialization returned null handle.
    #[error("null handle returned from plugin")]
    NullHandle,

    /// Required FFI symbol not found in library.
    #[error("missing symbol: {0}")]
    MissingSymbol(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for consumer operations.
pub type ConsumerResult<T> = Result<T, ConsumerError>;

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn ConsumerError___library_load___displays_message() {
        // Create a libloading error by trying to load a nonexistent library
        let lib_result: Result<libloading::Library, _> =
            unsafe { libloading::Library::new("/nonexistent/library.so") };

        let err = ConsumerError::from(lib_result.unwrap_err());

        assert!(err.to_string().contains("failed to load library"));
    }

    #[test]
    fn ConsumerError___init_failed___displays_message() {
        let err = ConsumerError::InitFailed("config parse error".to_string());

        assert_eq!(
            err.to_string(),
            "plugin initialization failed: config parse error"
        );
    }

    #[test]
    fn ConsumerError___not_active___displays_state() {
        let err = ConsumerError::NotActive(LifecycleState::Stopped);

        assert!(err.to_string().contains("Stopped"));
    }

    #[test]
    fn ConsumerError___null_handle___displays_message() {
        let err = ConsumerError::NullHandle;

        assert_eq!(err.to_string(), "null handle returned from plugin");
    }

    #[test]
    fn ConsumerError___missing_symbol___displays_symbol_name() {
        let err = ConsumerError::MissingSymbol("plugin_init".to_string());

        assert_eq!(err.to_string(), "missing symbol: plugin_init");
    }

    #[test]
    fn ConsumerError___invalid_response___displays_reason() {
        let err = ConsumerError::InvalidResponse("malformed JSON".to_string());

        assert_eq!(err.to_string(), "invalid response: malformed JSON");
    }
}
