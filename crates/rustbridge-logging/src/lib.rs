//! rustbridge-logging - Tracing to FFI callback bridge
//!
//! This crate provides:
//! - [`FfiLoggingLayer`] tracing layer that forwards logs to FFI callbacks
//! - [`LogCallback`] type for the FFI log callback function
//! - Dynamic log level filtering

mod callback;
mod layer;
mod reload;

pub use callback::{LogCallback, LogCallbackManager};
pub use layer::{FfiLoggingLayer, init_logging};
pub use reload::ReloadHandle;
pub use rustbridge_core::LogLevel;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{FfiLoggingLayer, LogCallback, LogCallbackManager, LogLevel, init_logging};
}
