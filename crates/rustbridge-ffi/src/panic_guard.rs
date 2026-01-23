//! Panic handling for FFI boundaries
//!
//! This module provides utilities to safely catch panics at FFI boundaries,
//! preventing them from unwinding into host language runtimes.

use crate::buffer::FfiBuffer;
use crate::handle::PluginHandleManager;
use std::any::Any;
use std::panic;

/// Catch panics from FFI calls and convert them to error buffers
///
/// This function wraps FFI operations to ensure panics don't cross the FFI boundary.
/// When a panic occurs:
/// 1. The panic is caught and logged via tracing
/// 2. The plugin is marked as Failed if a valid handle is provided
/// 3. An error buffer is returned with error code 11 (InternalError)
///
/// # Parameters
/// - `handle_id`: Optional plugin handle ID (0 if no handle available)
/// - `f`: The function to execute, which may panic
///
/// # Returns
/// - `Ok(R)` if the function executed successfully
/// - `Err(FfiBuffer)` if a panic was caught
///
/// # Example
///
/// ```ignore
/// #[unsafe(no_mangle)]
/// pub unsafe extern "C" fn plugin_call(...) -> FfiBuffer {
///     match catch_panic(handle as u64, || unsafe {
///         plugin_call_impl(handle, type_tag, request, request_len)
///     }) {
///         Ok(result) => result,
///         Err(error_buffer) => error_buffer,
///     }
/// }
/// ```
pub fn catch_panic<F, R>(handle_id: u64, f: F) -> Result<R, FfiBuffer>
where
    F: FnOnce() -> R + panic::UnwindSafe,
{
    panic::catch_unwind(f).map_err(|panic_info| {
        let panic_msg = panic_to_string(&panic_info);

        // Log the panic
        tracing::error!("FFI panic caught: {}", panic_msg);

        // Mark plugin as failed if we have a valid handle
        if handle_id != 0 {
            if let Some(h) = PluginHandleManager::global().get(handle_id) {
                h.mark_failed();
                tracing::warn!("Plugin handle {} marked as failed due to panic", handle_id);
            }
        }

        // Return error buffer with InternalError code (11)
        FfiBuffer::error(11, &panic_msg)
    })
}

/// Convert a panic payload to a human-readable string
///
/// Handles common panic payload types (&str, String) and provides
/// a fallback for unknown types.
fn panic_to_string(panic_info: &Box<dyn Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        format!("Plugin panicked: {}", s)
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        format!("Plugin panicked: {}", s)
    } else {
        "Plugin panicked with unknown payload".to_string()
    }
}

/// Install a custom panic hook that logs to the FFI callback
///
/// This should be called during plugin initialization to ensure panics
/// are properly logged through the host language's logging system.
///
/// The hook will:
/// - Extract file, line, and column information
/// - Log the panic message via tracing (which goes through FFI callback)
/// - Provide detailed context for debugging
///
/// # Safety
/// This function is safe to call, but note that:
/// - It replaces any existing panic hook
/// - The hook will be global for the entire process
/// - Multiple plugins in the same process will share the same hook
pub fn install_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(location) = panic_info.location() {
            let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Box<dyn Any>".to_string()
            };

            format!(
                "Panic at {}:{}:{}: {}",
                location.file(),
                location.line(),
                location.column(),
                payload
            )
        } else {
            format!(
                "Panic at unknown location: {:?}",
                panic_info
                    .payload()
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| panic_info
                        .payload()
                        .downcast_ref::<String>()
                        .map(|s| s.as_str()))
                    .unwrap_or("unknown")
            )
        };

        // Log via FFI callback through tracing
        tracing::error!("PANIC: {}", msg);
    }));
}

#[cfg(test)]
#[path = "panic_guard/panic_guard_tests.rs"]
mod panic_guard_tests;
