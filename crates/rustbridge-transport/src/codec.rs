//! Codec trait and JSON implementation

use rustbridge_core::PluginError;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// Errors that can occur during encoding/decoding
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

impl From<serde_json::Error> for CodecError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_data() || err.is_syntax() || err.is_eof() {
            CodecError::Deserialization(err.to_string())
        } else {
            CodecError::Serialization(err.to_string())
        }
    }
}

impl From<CodecError> for PluginError {
    fn from(err: CodecError) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

/// Trait for message encoding and decoding
pub trait Codec: Send + Sync {
    /// Encode a value to bytes
    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, CodecError>;

    /// Decode bytes to a value
    fn decode<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, CodecError>;

    /// Get the content type for this codec
    fn content_type(&self) -> &'static str;
}

/// JSON codec implementation using serde_json
#[derive(Debug, Clone, Default)]
pub struct JsonCodec {
    /// Whether to pretty-print output (default: false for efficiency)
    pretty: bool,
}

impl JsonCodec {
    /// Create a new JSON codec
    pub fn new() -> Self {
        Self { pretty: false }
    }

    /// Create a JSON codec that pretty-prints output
    pub fn pretty() -> Self {
        Self { pretty: true }
    }

    /// Encode a value directly to a JSON string
    pub fn encode_string<T: Serialize>(&self, value: &T) -> Result<String, CodecError> {
        if self.pretty {
            serde_json::to_string_pretty(value).map_err(Into::into)
        } else {
            serde_json::to_string(value).map_err(Into::into)
        }
    }

    /// Decode a JSON string to a value
    pub fn decode_str<T: DeserializeOwned>(&self, data: &str) -> Result<T, CodecError> {
        serde_json::from_str(data).map_err(Into::into)
    }
}

impl Codec for JsonCodec {
    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, CodecError> {
        if self.pretty {
            serde_json::to_vec_pretty(value).map_err(Into::into)
        } else {
            serde_json::to_vec(value).map_err(Into::into)
        }
    }

    fn decode<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, CodecError> {
        serde_json::from_slice(data).map_err(Into::into)
    }

    fn content_type(&self) -> &'static str {
        "application/json"
    }
}

#[cfg(test)]
#[path = "codec/codec_tests.rs"]
mod codec_tests;
