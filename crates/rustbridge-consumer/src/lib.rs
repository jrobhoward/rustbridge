//! Rust consumer for rustbridge plugins.
//!
//! This crate enables Rust applications to dynamically load and invoke rustbridge
//! plugin bundles (`.rbp` files) or shared libraries at runtime. It provides the
//! same functionality available to Java, Kotlin, C#, and Python consumers.
//!
//! # Features
//!
//! - **Dynamic Loading**: Load plugins at runtime without compile-time linking
//! - **Bundle Support**: Load from `.rbp` bundles with automatic platform detection
//! - **Signature Verification**: Verify minisign signatures on bundles
//! - **JSON Transport**: Make calls using JSON serialization
//! - **Binary Transport**: High-performance binary struct transport (7x faster)
//! - **Lifecycle Management**: Full OSGI-inspired lifecycle state machine
//! - **Logging Integration**: Route plugin logs through host callbacks
//!
//! # Quick Start
//!
//! ```ignore
//! use rustbridge_consumer::{NativePluginLoader, ConsumerResult};
//!
//! fn main() -> ConsumerResult<()> {
//!     // Load a plugin
//!     let plugin = NativePluginLoader::load("target/release/libmy_plugin.so")?;
//!
//!     // Make a call
//!     let response = plugin.call("echo", r#"{"message": "Hello"}"#)?;
//!     println!("Response: {response}");
//!
//!     Ok(())
//! }
//! ```
//!
//! # Loading from Bundles
//!
//! ```ignore
//! use rustbridge_consumer::{NativePluginLoader, PluginConfig};
//!
//! let config = PluginConfig::default();
//! let plugin = NativePluginLoader::load_bundle_with_config(
//!     "my-plugin-1.0.0.rbp",
//!     &config,
//!     None,
//! )?;
//! ```
//!
//! # Loading with Signature Verification
//!
//! ```ignore
//! use rustbridge_consumer::{NativePluginLoader, PluginConfig};
//!
//! // Load with signature verification (recommended for production)
//! let plugin = NativePluginLoader::load_bundle_with_verification(
//!     "my-plugin-1.0.0.rbp",
//!     &PluginConfig::default(),
//!     None,   // no log callback
//!     true,   // verify signatures
//!     None,   // use manifest's public key
//! )?;
//! ```
//!
//! # Typed Calls
//!
//! ```ignore
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize)]
//! struct EchoRequest { message: String }
//!
//! #[derive(Deserialize)]
//! struct EchoResponse { message: String, length: usize }
//!
//! let response: EchoResponse = plugin.call_typed("echo", &EchoRequest {
//!     message: "Hello".to_string(),
//! })?;
//! ```

mod error;
mod ffi_bindings;
mod loader;
mod plugin;

pub use error::{ConsumerError, ConsumerResult};
pub use loader::{LogCallbackFn, NativePluginLoader};
pub use plugin::NativePlugin;

// Re-export commonly used types from dependencies
pub use rustbridge_core::{LifecycleState, LogLevel, PluginConfig, PluginError};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        ConsumerError, ConsumerResult, LifecycleState, LogCallbackFn, LogLevel, NativePlugin,
        NativePluginLoader, PluginConfig,
    };
}
