//! rustbridge-runtime - Tokio async runtime integration
//!
//! This crate provides:
//! - [`AsyncRuntime`] for managing the Tokio runtime
//! - [`AsyncBridge`] for bridging sync FFI calls to async handlers
//! - Graceful shutdown support with broadcast signals

mod bridge;
mod runtime;
mod shutdown;

pub use bridge::AsyncBridge;
pub use runtime::{AsyncRuntime, RuntimeConfig};
pub use shutdown::{ShutdownHandle, ShutdownSignal};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{AsyncBridge, AsyncRuntime, RuntimeConfig, ShutdownHandle, ShutdownSignal};
}
