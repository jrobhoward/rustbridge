//! Native plugin wrapper for calling plugins loaded via FFI.

use crate::error::{ConsumerError, ConsumerResult};
use crate::ffi_bindings::{
    FfiBuffer, FfiPluginHandle, PluginCallFn, PluginCallRawFn, PluginFreeBufferFn,
    PluginGetRejectedCountFn, PluginGetStateFn, PluginSetLogLevelFn, PluginShutdownFn, RbResponse,
    RbResponseFreeFn,
};
use libloading::Library;
use rustbridge_core::{LifecycleState, LogLevel, PluginError};
use rustbridge_transport::ResponseEnvelope;
use serde::{Serialize, de::DeserializeOwned};
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

/// A loaded native plugin that can be called via FFI.
///
/// The plugin is automatically shut down when dropped.
pub struct NativePlugin {
    /// The loaded shared library (must be kept alive).
    #[allow(dead_code)]
    library: Library,

    /// Handle to the initialized plugin.
    handle: FfiPluginHandle,

    /// Whether the plugin has been shut down.
    shutdown: AtomicBool,

    // Cached function pointers
    call_fn: PluginCallFn,
    call_raw_fn: Option<PluginCallRawFn>,
    shutdown_fn: PluginShutdownFn,
    get_state_fn: PluginGetStateFn,
    get_rejected_count_fn: PluginGetRejectedCountFn,
    set_log_level_fn: PluginSetLogLevelFn,
    free_buffer_fn: PluginFreeBufferFn,
    rb_response_free_fn: Option<RbResponseFreeFn>,
}

impl NativePlugin {
    /// Create a new NativePlugin from loaded library and handle.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `library` contains valid FFI exports
    /// - `handle` is a valid plugin handle from `plugin_init`
    /// - The function pointers are valid for the library's lifetime
    #[allow(clippy::too_many_arguments)]
    pub(crate) unsafe fn new(
        library: Library,
        handle: FfiPluginHandle,
        call_fn: PluginCallFn,
        call_raw_fn: Option<PluginCallRawFn>,
        shutdown_fn: PluginShutdownFn,
        get_state_fn: PluginGetStateFn,
        get_rejected_count_fn: PluginGetRejectedCountFn,
        set_log_level_fn: PluginSetLogLevelFn,
        free_buffer_fn: PluginFreeBufferFn,
        rb_response_free_fn: Option<RbResponseFreeFn>,
    ) -> Self {
        Self {
            library,
            handle,
            shutdown: AtomicBool::new(false),
            call_fn,
            call_raw_fn,
            shutdown_fn,
            get_state_fn,
            get_rejected_count_fn,
            set_log_level_fn,
            free_buffer_fn,
            rb_response_free_fn,
        }
    }

    /// Make a JSON call to the plugin.
    ///
    /// # Arguments
    ///
    /// * `type_tag` - Message type identifier (e.g., "echo", "user.create")
    /// * `request` - JSON request payload as a string
    ///
    /// # Returns
    ///
    /// The JSON response payload as a string, or an error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = plugin.call("echo", r#"{"message": "Hello"}"#)?;
    /// ```
    pub fn call(&self, type_tag: &str, request: &str) -> ConsumerResult<String> {
        self.ensure_active()?;

        let type_tag_cstr =
            CString::new(type_tag).map_err(|e| ConsumerError::InvalidResponse(e.to_string()))?;

        let request_bytes = request.as_bytes();

        // SAFETY: We validated that the plugin is active and the handle is valid
        let buffer: FfiBuffer = unsafe {
            (self.call_fn)(
                self.handle,
                type_tag_cstr.as_ptr(),
                request_bytes.as_ptr(),
                request_bytes.len(),
            )
        };

        // Always free the buffer when we're done
        let result = self.process_buffer(&buffer);

        // SAFETY: buffer was returned from plugin_call and hasn't been freed
        unsafe {
            let mut buffer = buffer;
            (self.free_buffer_fn)(&mut buffer);
        }

        result
    }

    /// Make a typed call to the plugin with automatic serialization.
    ///
    /// # Arguments
    ///
    /// * `type_tag` - Message type identifier
    /// * `request` - Request value to serialize to JSON
    ///
    /// # Returns
    ///
    /// The deserialized response, or an error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(Serialize)]
    /// struct EchoRequest { message: String }
    ///
    /// #[derive(Deserialize)]
    /// struct EchoResponse { message: String, length: usize }
    ///
    /// let response: EchoResponse = plugin.call_typed("echo", &EchoRequest {
    ///     message: "Hello".to_string(),
    /// })?;
    /// ```
    pub fn call_typed<Req, Res>(&self, type_tag: &str, request: &Req) -> ConsumerResult<Res>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        let request_json = serde_json::to_string(request)?;
        let response_json = self.call(type_tag, &request_json)?;
        let response: Res = serde_json::from_str(&response_json)?;
        Ok(response)
    }

    /// Make a binary call to the plugin.
    ///
    /// This is used for high-performance binary transport.
    ///
    /// # Arguments
    ///
    /// * `message_id` - Numeric message identifier
    /// * `request` - Raw request bytes
    ///
    /// # Returns
    ///
    /// The raw response bytes, or an error.
    ///
    /// # Errors
    ///
    /// Returns `ConsumerError::MissingSymbol` if binary transport is not available.
    pub fn call_raw(&self, message_id: u32, request: &[u8]) -> ConsumerResult<Vec<u8>> {
        self.ensure_active()?;

        let call_raw_fn = self.call_raw_fn.ok_or_else(|| {
            ConsumerError::MissingSymbol("plugin_call_raw (binary transport not available)".into())
        })?;

        let rb_response_free_fn = self.rb_response_free_fn.ok_or_else(|| {
            ConsumerError::MissingSymbol("rb_response_free (binary transport not available)".into())
        })?;

        // SAFETY: We validated that the plugin is active and the handle is valid
        let response: RbResponse = unsafe {
            call_raw_fn(
                self.handle,
                message_id,
                request.as_ptr() as *const std::ffi::c_void,
                request.len(),
            )
        };

        // Extract data before freeing
        let result = if response.is_error() {
            // SAFETY: error response data is a null-terminated string
            let error_msg = if response.data.is_null() {
                "Unknown error".to_string()
            } else {
                let slice = unsafe { response.as_slice() };
                String::from_utf8_lossy(slice).into_owned()
            };
            Err(ConsumerError::CallFailed(PluginError::from_code(
                response.error_code,
                error_msg,
            )))
        } else {
            // SAFETY: success response data is valid for len bytes
            let data = unsafe { response.as_slice().to_vec() };
            Ok(data)
        };

        // SAFETY: response was returned from plugin_call_raw and hasn't been freed
        unsafe {
            let mut response = response;
            rb_response_free_fn(&mut response);
        }

        result
    }

    /// Get the current lifecycle state of the plugin.
    pub fn state(&self) -> LifecycleState {
        // SAFETY: handle is valid
        let state_code = unsafe { (self.get_state_fn)(self.handle) };
        state_from_u8(state_code)
    }

    /// Get the number of requests rejected due to concurrency limits.
    pub fn rejected_request_count(&self) -> u64 {
        // SAFETY: handle is valid
        unsafe { (self.get_rejected_count_fn)(self.handle) }
    }

    /// Check if binary transport is available.
    pub fn has_binary_transport(&self) -> bool {
        self.call_raw_fn.is_some() && self.rb_response_free_fn.is_some()
    }

    /// Set the log level for the plugin.
    pub fn set_log_level(&self, level: LogLevel) {
        // SAFETY: handle is valid
        unsafe { (self.set_log_level_fn)(self.handle, level as u8) }
    }

    /// Shutdown the plugin gracefully.
    ///
    /// This is called automatically when the plugin is dropped, but can be
    /// called explicitly to handle shutdown errors.
    pub fn shutdown(&self) -> ConsumerResult<()> {
        // Only shutdown once
        if self
            .shutdown
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(());
        }

        // SAFETY: handle is valid and we're only calling shutdown once
        let success = unsafe { (self.shutdown_fn)(self.handle) };

        if success {
            Ok(())
        } else {
            Err(ConsumerError::InitFailed(
                "plugin shutdown returned false".to_string(),
            ))
        }
    }

    /// Ensure the plugin is in Active state.
    fn ensure_active(&self) -> ConsumerResult<()> {
        let state = self.state();
        if state.can_handle_requests() {
            Ok(())
        } else {
            Err(ConsumerError::NotActive(state))
        }
    }

    /// Process a buffer response from the plugin.
    fn process_buffer(&self, buffer: &FfiBuffer) -> ConsumerResult<String> {
        if buffer.is_error() {
            // SAFETY: error buffer data is valid
            let error_msg = if buffer.is_empty() {
                "Unknown error".to_string()
            } else {
                let slice = unsafe { buffer.as_slice() };
                String::from_utf8_lossy(slice).into_owned()
            };
            return Err(ConsumerError::CallFailed(PluginError::from_code(
                buffer.error_code,
                error_msg,
            )));
        }

        // SAFETY: success buffer data is valid JSON
        let data = unsafe { buffer.as_slice() };

        // Parse the response envelope
        let envelope: ResponseEnvelope = serde_json::from_slice(data)
            .map_err(|e| ConsumerError::InvalidResponse(e.to_string()))?;

        if envelope.is_success() {
            // Extract payload as JSON string
            match envelope.payload {
                Some(payload) => Ok(serde_json::to_string(&payload)?),
                None => Ok("null".to_string()),
            }
        } else {
            let code = envelope.error_code.unwrap_or(11);
            let message = envelope.error_message.unwrap_or_default();
            Err(ConsumerError::CallFailed(PluginError::from_code(
                code, message,
            )))
        }
    }
}

impl Drop for NativePlugin {
    fn drop(&mut self) {
        // Ignore errors on drop
        let _ = self.shutdown();
    }
}

// NativePlugin is Send because it owns all its data and the library is thread-safe.
// The FFI functions are designed to be called from any thread.
unsafe impl Send for NativePlugin {}

// NativePlugin is Sync because the FFI calls are thread-safe.
// The plugin's internal state uses proper synchronization (Arc<RwLock>, DashMap, etc.)
// and all function pointers are constant after initialization.
unsafe impl Sync for NativePlugin {}

/// Convert a u8 state code to LifecycleState.
fn state_from_u8(code: u8) -> LifecycleState {
    match code {
        0 => LifecycleState::Installed,
        1 => LifecycleState::Starting,
        2 => LifecycleState::Active,
        3 => LifecycleState::Stopping,
        4 => LifecycleState::Stopped,
        5 => LifecycleState::Failed,
        _ => LifecycleState::Failed, // Unknown state treated as failed
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn state_from_u8___valid_codes___returns_correct_state() {
        assert_eq!(state_from_u8(0), LifecycleState::Installed);
        assert_eq!(state_from_u8(1), LifecycleState::Starting);
        assert_eq!(state_from_u8(2), LifecycleState::Active);
        assert_eq!(state_from_u8(3), LifecycleState::Stopping);
        assert_eq!(state_from_u8(4), LifecycleState::Stopped);
        assert_eq!(state_from_u8(5), LifecycleState::Failed);
    }

    #[test]
    fn state_from_u8___invalid_code___returns_failed() {
        assert_eq!(state_from_u8(255), LifecycleState::Failed);
        assert_eq!(state_from_u8(100), LifecycleState::Failed);
    }
}
