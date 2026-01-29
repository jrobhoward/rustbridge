//! # rustbridge
//!
//! A framework for developing Rust shared libraries callable from other languages.
//!
//! rustbridge uses C ABI under the hood but abstracts the complexity, providing:
//! - OSGI-like lifecycle management
//! - Mandatory async (Tokio) runtime
//! - Logging callbacks to host language
//! - JSON-based data transport with optional binary transport
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//!
//! [dependencies]
//! rustbridge = "0.6"
//! ```
//!
//! ## Creating a Plugin
//!
//! ```ignore
//! use rustbridge::prelude::*;
//!
//! #[derive(Debug, Serialize, Deserialize, Message)]
//! #[message(tag = "echo")]
//! pub struct EchoRequest {
//!     pub message: String,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! pub struct EchoResponse {
//!     pub message: String,
//! }
//!
//! #[derive(Default)]
//! pub struct MyPlugin;
//!
//! #[async_trait]
//! impl Plugin for MyPlugin {
//!     async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
//!         tracing::info!("Plugin started");
//!         Ok(())
//!     }
//!
//!     async fn handle_request(
//!         &self,
//!         _ctx: &PluginContext,
//!         type_tag: &str,
//!         payload: &[u8],
//!     ) -> PluginResult<Vec<u8>> {
//!         match type_tag {
//!             "echo" => {
//!                 let req: EchoRequest = serde_json::from_slice(payload)?;
//!                 Ok(serde_json::to_vec(&EchoResponse { message: req.message })?)
//!             }
//!             _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
//!         }
//!     }
//!
//!     async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
//!         tracing::info!("Plugin stopped");
//!         Ok(())
//!     }
//! }
//!
//! // Generate FFI entry point
//! rustbridge_entry!(MyPlugin::default);
//!
//! // Re-export FFI functions for the shared library
//! pub use rustbridge::ffi_exports::*;
//! ```
//!
//! ## Crate Structure
//!
//! This is a facade crate that re-exports from:
//! - [`rustbridge_core`] - Core traits, types, and lifecycle
//! - [`rustbridge_macros`] - Procedural macros (`Message`, `rustbridge_entry!`)
//! - [`rustbridge_ffi`] - FFI exports and buffer management

// Re-export core types
pub use rustbridge_core::{
    LifecycleState, LogLevel, Plugin, PluginConfig, PluginContext, PluginError, PluginFactory,
    PluginMetadata, PluginResult, RequestContext, ResponseBuilder,
};

// Re-export macros
pub use rustbridge_macros::{
    Message, impl_plugin, rustbridge_entry, rustbridge_handler, rustbridge_plugin,
};

// Re-export FFI types
pub use rustbridge_ffi::{FfiBuffer, PluginHandle, PluginHandleManager, register_binary_handler};

// Re-export common dependencies that plugin authors need
pub use async_trait::async_trait;
pub use serde;
pub use serde_json;
pub use tokio;
pub use tracing;

/// FFI function exports that plugins must re-export.
///
/// Add `pub use rustbridge::ffi_exports::*;` at the end of your plugin's lib.rs
/// to expose the required FFI functions for the shared library.
pub mod ffi_exports {
    pub use rustbridge_ffi::{
        plugin_call, plugin_call_async, plugin_call_raw, plugin_cancel_async, plugin_free_buffer,
        plugin_get_rejected_count, plugin_get_state, plugin_init, plugin_set_log_level,
        plugin_shutdown, rb_response_free,
    };
}

/// Prelude module for convenient imports.
///
/// Use `use rustbridge::prelude::*;` to import commonly used types.
///
/// This includes:
/// - Core traits: `Plugin`, `PluginFactory`
/// - Core types: `PluginConfig`, `PluginContext`, `PluginError`, `PluginResult`
/// - Lifecycle: `LifecycleState`, `LogLevel`
/// - Macros: `Message`, `rustbridge_entry!`, `rustbridge_plugin`, `rustbridge_handler`
/// - Common deps: `async_trait`, `Serialize`, `Deserialize`
pub mod prelude {
    // Core traits and types
    pub use crate::{
        LifecycleState, LogLevel, Plugin, PluginConfig, PluginContext, PluginError, PluginFactory,
        PluginResult, async_trait,
    };

    // Macros
    pub use rustbridge_macros::{Message, rustbridge_entry, rustbridge_handler, rustbridge_plugin};

    // Serde derives (commonly needed for message types)
    pub use serde::{Deserialize, Serialize};
}
