//! Port driver error types.

use rustbridge_consumer::ConsumerError;
use thiserror::Error;

/// Port-driver-specific error codes (200+ range).
/// Plugin errors (1-13) pass through from `rustbridge-consumer`.
pub const CODE_PROTOCOL_ERROR: u32 = 200;
pub const CODE_PLUGIN_NOT_LOADED: u32 = 201;
pub const CODE_PLUGIN_ALREADY_LOADED: u32 = 202;
pub const CODE_BASE64_DECODE: u32 = 203;

/// Errors specific to the port driver process.
#[derive(Error, Debug)]
pub enum PortError {
    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("plugin not loaded")]
    PluginNotLoaded,

    #[error("plugin already loaded")]
    PluginAlreadyLoaded,

    #[error("base64 decode error: {0}")]
    Base64Decode(String),

    #[error("consumer error: {0}")]
    Consumer(#[from] ConsumerError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl PortError {
    /// Return the numeric error code for this error.
    pub fn code(&self) -> u32 {
        match self {
            PortError::Protocol(_) => CODE_PROTOCOL_ERROR,
            PortError::PluginNotLoaded => CODE_PLUGIN_NOT_LOADED,
            PortError::PluginAlreadyLoaded => CODE_PLUGIN_ALREADY_LOADED,
            PortError::Base64Decode(_) => CODE_BASE64_DECODE,
            PortError::Consumer(e) => consumer_error_code(e),
            PortError::Io(_) => CODE_PROTOCOL_ERROR,
        }
    }
}

/// Extract the numeric error code from a ConsumerError.
fn consumer_error_code(err: &ConsumerError) -> u32 {
    match err {
        ConsumerError::CallFailed(plugin_err) => plugin_err.error_code(),
        ConsumerError::LibraryLoad(_) => 2,
        ConsumerError::InitFailed(_) => 2,
        ConsumerError::NotActive(_) => 1,
        ConsumerError::NullHandle => 2,
        ConsumerError::MissingSymbol(_) => 12,
        ConsumerError::InvalidResponse(_) => 5,
        ConsumerError::Io(_) => 11,
        ConsumerError::Serialization(_) => 5,
        ConsumerError::Bundle(_) => 11,
    }
}
