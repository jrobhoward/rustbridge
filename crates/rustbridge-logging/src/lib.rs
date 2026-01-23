//! rustbridge-logging - Tracing to FFI callback bridge
//!
//! This crate provides:
//! - [`FfiLoggingLayer`] tracing layer that forwards logs to FFI callbacks
//! - [`LogCallback`] type for the FFI log callback function
//! - Dynamic log level filtering

mod callback;
mod layer;

pub use callback::{LogCallback, LogCallbackManager};
pub use layer::{init_logging, FfiLoggingLayer};
pub use rustbridge_core::LogLevel;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{init_logging, FfiLoggingLayer, LogCallback, LogCallbackManager, LogLevel};
}
