//! Request and response envelope types for FFI transport

use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Request envelope wrapping a message for FFI transport
///
/// The type_tag identifies the handler, and payload contains the
/// serialized request data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope {
    /// Message type identifier (e.g., "user.create", "order.submit")
    pub type_tag: String,

    /// Serialized request payload (JSON)
    pub payload: serde_json::Value,

    /// Optional request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,

    /// Optional correlation ID for distributed tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl RequestEnvelope {
    /// Create a new request envelope
    pub fn new(type_tag: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            type_tag: type_tag.into(),
            payload,
            request_id: None,
            correlation_id: None,
        }
    }

    /// Set request ID
    pub fn with_request_id(mut self, id: u64) -> Self {
        self.request_id = Some(id);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Create from type tag and serializable payload
    pub fn from_typed<T: Serialize>(
        type_tag: impl Into<String>,
        payload: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            type_tag: type_tag.into(),
            payload: serde_json::to_value(payload)?,
            request_id: None,
            correlation_id: None,
        })
    }

    /// Deserialize the payload to a typed value
    ///
    /// This method deserializes directly from the JSON value without cloning.
    pub fn payload_as<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        T::deserialize(&self.payload)
    }

    /// Serialize to bytes for FFI transport
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// Response status indicating success or failure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with an error
    Error,
}

/// Response envelope wrapping a response for FFI transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    /// Response status
    pub status: ResponseStatus,

    /// Serialized response payload (on success) or null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// Error code (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<u32>,

    /// Error message (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Original request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,
}

impl ResponseEnvelope {
    /// Create a success response with payload
    pub fn success(payload: serde_json::Value) -> Self {
        Self {
            status: ResponseStatus::Success,
            payload: Some(payload),
            error_code: None,
            error_message: None,
            request_id: None,
        }
    }

    /// Create a success response from a serializable value
    pub fn success_typed<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        Ok(Self::success(serde_json::to_value(value)?))
    }

    /// Create a success response from raw bytes (already JSON-encoded)
    pub fn success_raw(data: &[u8]) -> Result<Self, serde_json::Error> {
        let payload: serde_json::Value = serde_json::from_slice(data)?;
        Ok(Self::success(payload))
    }

    /// Create an error response
    pub fn error(code: u32, message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            payload: None,
            error_code: Some(code),
            error_message: Some(message.into()),
            request_id: None,
        }
    }

    /// Create an error response from a PluginError
    pub fn from_error(err: &rustbridge_core::PluginError) -> Self {
        Self::error(err.error_code(), err.to_string())
    }

    /// Set request ID for correlation
    pub fn with_request_id(mut self, id: u64) -> Self {
        self.request_id = Some(id);
        self
    }

    /// Check if this is a success response
    pub fn is_success(&self) -> bool {
        self.status == ResponseStatus::Success
    }

    /// Get the payload if success
    ///
    /// This method deserializes directly from the JSON value without cloning.
    pub fn payload_as<T: DeserializeOwned>(&self) -> Result<Option<T>, serde_json::Error> {
        match &self.payload {
            Some(v) => Ok(Some(T::deserialize(v)?)),
            None => Ok(None),
        }
    }

    /// Serialize to bytes for FFI transport
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

impl Default for ResponseEnvelope {
    fn default() -> Self {
        Self::success(serde_json::Value::Null)
    }
}

#[cfg(test)]
#[path = "envelope/envelope_tests.rs"]
mod envelope_tests;
