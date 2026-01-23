//! Request and response context types

use serde::{Deserialize, Serialize};

/// Context for an incoming request
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID
    pub request_id: u64,
    /// Message type tag
    pub type_tag: String,
    /// Optional correlation ID for tracking across systems
    pub correlation_id: Option<String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(request_id: u64, type_tag: impl Into<String>) -> Self {
        Self {
            request_id,
            type_tag: type_tag.into(),
            correlation_id: None,
        }
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Builder for constructing responses
#[derive(Debug)]
pub struct ResponseBuilder {
    data: Option<Vec<u8>>,
    error: Option<ResponseError>,
}

/// Error information in a response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    /// Error code
    pub code: u32,
    /// Error message
    pub message: String,
    /// Optional detailed information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ResponseBuilder {
    /// Create a new response builder
    pub fn new() -> Self {
        Self {
            data: None,
            error: None,
        }
    }

    /// Set successful response data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set response data from a serializable value
    pub fn json<T: Serialize>(mut self, value: &T) -> Result<Self, serde_json::Error> {
        self.data = Some(serde_json::to_vec(value)?);
        Ok(self)
    }

    /// Set error response
    pub fn error(mut self, code: u32, message: impl Into<String>) -> Self {
        self.error = Some(ResponseError {
            code,
            message: message.into(),
            details: None,
        });
        self
    }

    /// Set error with details
    pub fn error_with_details(
        mut self,
        code: u32,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        self.error = Some(ResponseError {
            code,
            message: message.into(),
            details: Some(details),
        });
        self
    }

    /// Build the response
    pub fn build(self) -> ResponseResult {
        if let Some(error) = self.error {
            ResponseResult::Error(error)
        } else {
            ResponseResult::Success(self.data.unwrap_or_default())
        }
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of building a response
#[derive(Debug)]
pub enum ResponseResult {
    /// Successful response with data
    Success(Vec<u8>),
    /// Error response
    Error(ResponseError),
}

impl ResponseResult {
    /// Check if this is a success response
    pub fn is_success(&self) -> bool {
        matches!(self, ResponseResult::Success(_))
    }

    /// Get the data if this is a success response
    pub fn data(&self) -> Option<&[u8]> {
        match self {
            ResponseResult::Success(data) => Some(data),
            ResponseResult::Error(_) => None,
        }
    }

    /// Get the error if this is an error response
    pub fn error(&self) -> Option<&ResponseError> {
        match self {
            ResponseResult::Success(_) => None,
            ResponseResult::Error(err) => Some(err),
        }
    }
}

#[cfg(test)]
#[path = "request/request_tests.rs"]
mod request_tests;
