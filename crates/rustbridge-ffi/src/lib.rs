//! rustbridge-ffi - C ABI exports and FFI buffer management
//!
//! This crate provides the FFI boundary layer:
//! - [`FfiBuffer`] for passing data across FFI
//! - [`PluginHandle`] for managing plugin instances
//! - C ABI exported functions (plugin_init, plugin_call, etc.)
//!
//! # FFI Functions
//!
//! The following functions are exported with C linkage:
//!
//! - `plugin_init` - Initialize a plugin instance
//! - `plugin_call` - Make a synchronous request to the plugin
//! - `plugin_free_buffer` - Free a buffer returned by plugin_call
//! - `plugin_shutdown` - Shutdown a plugin instance
//! - `plugin_set_log_level` - Set the log level for a plugin
//! - `plugin_call_async` - Make an async request (future)
//! - `plugin_cancel_async` - Cancel an async request (future)

mod buffer;
mod exports;
mod handle;

pub use buffer::FfiBuffer;
pub use handle::{PluginHandle, PluginHandleManager};

// Re-export FFI functions for use by plugins
pub use exports::{
    plugin_call, plugin_call_async, plugin_cancel_async, plugin_free_buffer, plugin_get_state,
    plugin_init, plugin_set_log_level, plugin_shutdown,
};

// Re-export types needed for plugin implementation
pub use rustbridge_core::{LogLevel, Plugin, PluginConfig, PluginContext, PluginError};
pub use rustbridge_logging::LogCallback;
pub use rustbridge_runtime::{AsyncBridge, AsyncRuntime, RuntimeConfig};
pub use rustbridge_transport::{RequestEnvelope, ResponseEnvelope};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{FfiBuffer, PluginHandle, PluginHandleManager};
    pub use rustbridge_core::prelude::*;
    pub use rustbridge_logging::prelude::*;
    pub use rustbridge_runtime::prelude::*;
    pub use rustbridge_transport::prelude::*;
}

/// Macro to generate the FFI entry point for a plugin
///
/// This macro creates the necessary `extern "C"` functions that the host
/// will call to interact with the plugin.
///
/// # Example
///
/// ```ignore
/// use rustbridge_ffi::prelude::*;
///
/// struct MyPlugin;
///
/// // ... implement Plugin trait ...
///
/// rustbridge_ffi::plugin_entry!(MyPlugin::new);
/// ```
#[macro_export]
macro_rules! plugin_entry {
    ($factory:expr_2021) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn plugin_create() -> *mut std::ffi::c_void {
            let plugin = Box::new($factory());
            Box::into_raw(plugin) as *mut std::ffi::c_void
        }
    };
}
