//! C ABI exported functions
//!
//! These functions are the FFI entry points called by host languages.

use crate::binary_types::RbResponse;
use crate::buffer::FfiBuffer;
use crate::handle::{PluginHandle, PluginHandleManager};
use crate::panic_guard::catch_panic;
use rustbridge_core::{LogLevel, PluginConfig};
use rustbridge_logging::{LogCallback, LogCallbackManager};
use rustbridge_transport::ResponseEnvelope;
use std::ffi::c_void;
use std::panic::AssertUnwindSafe;
use std::ptr;

/// Opaque handle type for FFI
pub type FfiPluginHandle = *mut c_void;

/// Initialize a plugin instance
///
/// # Parameters
/// - `plugin_ptr`: Pointer to the plugin instance (from plugin_create)
/// - `config_json`: JSON configuration bytes (can be null for defaults)
/// - `config_len`: Length of config_json
/// - `log_callback`: Optional log callback function
///
/// # Returns
/// Handle to the initialized plugin, or null on failure
///
/// # Safety
/// - `plugin_ptr` must be a valid pointer from `plugin_create`
/// - `config_json` must be valid for `config_len` bytes if not null
/// - The log callback must remain valid for the lifetime of the plugin
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_init(
    plugin_ptr: *mut c_void,
    config_json: *const u8,
    config_len: usize,
    log_callback: Option<LogCallback>,
) -> FfiPluginHandle {
    // Wrap in panic handler (handle_id = 0 since no handle exists yet)
    match catch_panic(
        0,
        AssertUnwindSafe(|| unsafe {
            plugin_init_impl(plugin_ptr, config_json, config_len, log_callback)
        }),
    ) {
        Ok(handle) => handle,
        Err(_error_buffer) => {
            // For plugin_init, return null on panic instead of FfiBuffer
            // since we're returning a handle, not a buffer
            ptr::null_mut()
        }
    }
}

/// Internal implementation of plugin_init (wrapped by panic handler)
unsafe fn plugin_init_impl(
    plugin_ptr: *mut c_void,
    config_json: *const u8,
    config_len: usize,
    log_callback: Option<LogCallback>,
) -> FfiPluginHandle {
    // Validate plugin pointer
    if plugin_ptr.is_null() {
        return ptr::null_mut();
    }

    // Register plugin with callback manager (increments ref count)
    LogCallbackManager::global().register_plugin(log_callback);

    // Initialize logging
    rustbridge_logging::init_logging();

    // Install panic hook to log panics via FFI callback
    crate::panic_guard::install_panic_hook();

    // Parse configuration
    let config = if config_json.is_null() || config_len == 0 {
        PluginConfig::default()
    } else {
        // SAFETY: caller guarantees config_json is valid for config_len bytes
        let config_slice = unsafe { std::slice::from_raw_parts(config_json, config_len) };
        match PluginConfig::from_json(config_slice) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to parse config: {}", e);
                return ptr::null_mut();
            }
        }
    };

    // Take ownership of the plugin
    // SAFETY: caller guarantees plugin_ptr is from plugin_create
    let plugin: Box<Box<dyn rustbridge_core::Plugin>> =
        unsafe { Box::from_raw(plugin_ptr as *mut Box<dyn rustbridge_core::Plugin>) };

    // Create the handle
    let handle = match PluginHandle::new(*plugin, config) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Failed to create handle: {}", e);
            return ptr::null_mut();
        }
    };

    // Start the plugin
    if let Err(e) = handle.start() {
        tracing::error!("Failed to start plugin: {}", e);
        return ptr::null_mut();
    }

    // Register and return handle
    let id = PluginHandleManager::global().register(handle);

    // Store ID in the handle
    if let Some(h) = PluginHandleManager::global().get(id) {
        h.set_id(id);
    }

    id as FfiPluginHandle
}

/// Make a synchronous call to the plugin
///
/// # Parameters
/// - `handle`: Plugin handle from plugin_init
/// - `type_tag`: Message type identifier (null-terminated C string)
/// - `request`: Request payload bytes
/// - `request_len`: Length of request payload
///
/// # Returns
/// FfiBuffer containing the response (must be freed with plugin_free_buffer)
///
/// # Safety
/// - `handle` must be a valid handle from plugin_init
/// - `type_tag` must be a valid null-terminated C string
/// - `request` must be valid for `request_len` bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_call(
    handle: FfiPluginHandle,
    type_tag: *const std::ffi::c_char,
    request: *const u8,
    request_len: usize,
) -> FfiBuffer {
    // Wrap in panic handler
    let handle_id = handle as u64;
    match catch_panic(
        handle_id,
        AssertUnwindSafe(|| unsafe { plugin_call_impl(handle, type_tag, request, request_len) }),
    ) {
        Ok(result) => result,
        Err(error_buffer) => error_buffer,
    }
}

/// Internal implementation of plugin_call (wrapped by panic handler)
unsafe fn plugin_call_impl(
    handle: FfiPluginHandle,
    type_tag: *const std::ffi::c_char,
    request: *const u8,
    request_len: usize,
) -> FfiBuffer {
    // Validate handle
    let id = handle as u64;
    let plugin_handle = match PluginHandleManager::global().get(id) {
        Some(h) => h,
        None => return FfiBuffer::error(1, "Invalid handle"),
    };

    // Parse type tag
    let type_tag_str = if type_tag.is_null() {
        return FfiBuffer::error(4, "Type tag is null");
    } else {
        // SAFETY: caller guarantees type_tag is a valid null-terminated C string
        match unsafe { std::ffi::CStr::from_ptr(type_tag) }.to_str() {
            Ok(s) => s,
            Err(_) => return FfiBuffer::error(4, "Invalid type tag encoding"),
        }
    };

    // Get request data
    let request_data = if request.is_null() || request_len == 0 {
        &[]
    } else {
        // SAFETY: caller guarantees request is valid for request_len bytes
        unsafe { std::slice::from_raw_parts(request, request_len) }
    };

    // Make the call
    match plugin_handle.call(type_tag_str, request_data) {
        Ok(response_data) => {
            // Wrap in response envelope
            match ResponseEnvelope::success_raw(&response_data) {
                Ok(envelope) => match envelope.to_bytes() {
                    Ok(bytes) => FfiBuffer::from_vec(bytes),
                    Err(e) => FfiBuffer::error(5, &format!("Serialization error: {}", e)),
                },
                Err(e) => FfiBuffer::error(5, &format!("Serialization error: {}", e)),
            }
        }
        Err(e) => {
            let envelope = ResponseEnvelope::from_error(&e);
            match envelope.to_bytes() {
                Ok(bytes) => {
                    let mut buf = FfiBuffer::from_vec(bytes);
                    buf.error_code = e.error_code();
                    buf
                }
                Err(se) => FfiBuffer::error(e.error_code(), &format!("{}: {}", e, se)),
            }
        }
    }
}

/// Free a buffer returned by plugin_call
///
/// # Safety
/// - `buffer` must be a valid FfiBuffer from plugin_call
/// - Must only be called once per buffer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_free_buffer(buffer: *mut FfiBuffer) {
    unsafe {
        if !buffer.is_null() {
            (*buffer).free();
        }
    }
}

/// Shutdown a plugin instance
///
/// # Parameters
/// - `handle`: Plugin handle from plugin_init
///
/// # Returns
/// true on success, false on failure
///
/// # Safety
/// - `handle` must be a valid handle from plugin_init
/// - After this call, the handle is no longer valid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_shutdown(handle: FfiPluginHandle) -> bool {
    // Wrap in panic handler
    let handle_id = handle as u64;
    catch_panic(handle_id, AssertUnwindSafe(|| plugin_shutdown_impl(handle))).unwrap_or_default() // Returns false on panic
}

/// Internal implementation of plugin_shutdown (wrapped by panic handler)
fn plugin_shutdown_impl(handle: FfiPluginHandle) -> bool {
    let id = handle as u64;

    // Remove from manager
    let plugin_handle = match PluginHandleManager::global().remove(id) {
        Some(h) => h,
        None => return false,
    };

    // Shutdown with default timeout
    let result = match plugin_handle.shutdown(5000) {
        Ok(()) => true,
        Err(e) => {
            tracing::error!("Shutdown error: {}", e);
            false
        }
    };

    // Unregister plugin from callback manager (decrements ref count)
    // The callback will only be cleared when the last plugin shuts down.
    // This allows multiple plugins to coexist and share the same callback.
    LogCallbackManager::global().unregister_plugin();

    result
}

/// Set the log level for a plugin
///
/// # Parameters
/// - `handle`: Plugin handle from plugin_init
/// - `level`: Log level (0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error, 5=Off)
///
/// # Safety
/// - `handle` must be a valid handle from plugin_init
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_set_log_level(handle: FfiPluginHandle, level: u8) {
    let id = handle as u64;

    if let Some(plugin_handle) = PluginHandleManager::global().get(id) {
        plugin_handle.set_log_level(LogLevel::from_u8(level));
    }
}

/// Get the current state of a plugin
///
/// # Parameters
/// - `handle`: Plugin handle from plugin_init
///
/// # Returns
/// State code (0=Installed, 1=Starting, 2=Active, 3=Stopping, 4=Stopped, 5=Failed)
/// Returns 255 if handle is invalid
///
/// # Safety
/// - `handle` must be a valid handle from plugin_init
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_get_state(handle: FfiPluginHandle) -> u8 {
    let id = handle as u64;

    match PluginHandleManager::global().get(id) {
        Some(h) => match h.state() {
            rustbridge_core::LifecycleState::Installed => 0,
            rustbridge_core::LifecycleState::Starting => 1,
            rustbridge_core::LifecycleState::Active => 2,
            rustbridge_core::LifecycleState::Stopping => 3,
            rustbridge_core::LifecycleState::Stopped => 4,
            rustbridge_core::LifecycleState::Failed => 5,
        },
        None => 255,
    }
}

/// Get the number of requests rejected due to concurrency limits
///
/// # Parameters
/// - `handle`: Plugin handle from plugin_init
///
/// # Returns
/// Number of rejected requests since plugin initialization. Returns 0 if handle is invalid.
///
/// # Safety
/// - `handle` must be a valid handle from plugin_init
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_get_rejected_count(handle: FfiPluginHandle) -> u64 {
    let id = handle as u64;
    match PluginHandleManager::global().get(id) {
        Some(h) => h.rejected_request_count(),
        None => 0,
    }
}

// ============================================================================
// Binary Transport Functions
// ============================================================================

/// Handler function type for binary message processing
///
/// Plugins that want to support binary transport must register handlers
/// using this signature. The handler receives the request struct as raw bytes
/// and must return response bytes (which will be wrapped in RbResponse).
pub type BinaryMessageHandler =
    fn(handle: &PluginHandle, request: &[u8]) -> Result<Vec<u8>, rustbridge_core::PluginError>;

// Registry for binary message handlers
//
// This is a simple thread-local registry mapping message_id to handlers.
// Plugins can register handlers during initialization.
//
// Note: For the initial benchmark, we use a simple static approach.
// A more sophisticated implementation would use the Plugin trait.
thread_local! {
    static BINARY_HANDLERS: std::cell::RefCell<std::collections::HashMap<u32, BinaryMessageHandler>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Register a binary message handler
///
/// Call this during plugin initialization to register handlers for
/// binary transport message types.
pub fn register_binary_handler(message_id: u32, handler: BinaryMessageHandler) {
    BINARY_HANDLERS.with(|handlers| {
        handlers.borrow_mut().insert(message_id, handler);
    });
}

/// Make a synchronous binary call to the plugin
///
/// # Parameters
/// - `handle`: Plugin handle from plugin_init
/// - `message_id`: Numeric message identifier
/// - `request`: Pointer to request struct
/// - `request_size`: Size of request struct (for validation)
///
/// # Returns
/// RbResponse containing the binary response (must be freed with rb_response_free)
///
/// # Safety
/// - `handle` must be a valid handle from plugin_init
/// - `request` must be valid for `request_size` bytes
/// - The request struct must match the expected type for `message_id`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_call_raw(
    handle: FfiPluginHandle,
    message_id: u32,
    request: *const c_void,
    request_size: usize,
) -> RbResponse {
    // Wrap in panic handler
    let handle_id = handle as u64;
    match catch_panic(
        handle_id,
        AssertUnwindSafe(|| unsafe {
            plugin_call_raw_impl(handle, message_id, request, request_size)
        }),
    ) {
        Ok(result) => result,
        Err(error_buffer) => {
            // Convert FfiBuffer error to RbResponse error
            let msg = if error_buffer.is_error() && !error_buffer.data.is_null() {
                // SAFETY: error buffer contains a valid string
                let slice =
                    unsafe { std::slice::from_raw_parts(error_buffer.data, error_buffer.len) };
                String::from_utf8_lossy(slice).into_owned()
            } else {
                "Internal error (panic)".to_string()
            };
            // Free the original error buffer
            let mut buf = error_buffer;
            // SAFETY: buf is a valid FfiBuffer from catch_panic
            unsafe { buf.free() };
            RbResponse::error(11, &msg)
        }
    }
}

/// Internal implementation of plugin_call_raw (wrapped by panic handler)
unsafe fn plugin_call_raw_impl(
    handle: FfiPluginHandle,
    message_id: u32,
    request: *const c_void,
    request_size: usize,
) -> RbResponse {
    // Validate handle
    let id = handle as u64;
    let plugin_handle = match PluginHandleManager::global().get(id) {
        Some(h) => h,
        None => return RbResponse::error(1, "Invalid handle"),
    };

    // Check plugin state
    if !plugin_handle.state().can_handle_requests() {
        return RbResponse::error(1, "Plugin not in Active state");
    }

    // Get request data
    let request_data = if request.is_null() || request_size == 0 {
        &[]
    } else {
        // SAFETY: caller guarantees request is valid for request_size bytes
        unsafe { std::slice::from_raw_parts(request as *const u8, request_size) }
    };

    // Look up handler
    let handler = BINARY_HANDLERS.with(|handlers| handlers.borrow().get(&message_id).copied());

    match handler {
        Some(h) => {
            // Call the handler
            match h(&plugin_handle, request_data) {
                Ok(response_bytes) => {
                    // Return raw bytes as response
                    // The caller is responsible for interpreting the bytes as the correct struct
                    let mut response = RbResponse::empty();
                    let len = response_bytes.len();
                    let capacity = response_bytes.capacity();
                    let data = response_bytes.leak().as_mut_ptr();

                    response.error_code = 0;
                    response.len = len as u32;
                    response.capacity = capacity as u32;
                    response.data = data as *mut c_void;

                    response
                }
                Err(e) => RbResponse::error(e.error_code(), &e.to_string()),
            }
        }
        None => RbResponse::error(6, &format!("Unknown message ID: {}", message_id)),
    }
}

/// Free an RbResponse returned by plugin_call_raw
///
/// # Safety
/// - `response` must be a valid pointer to an RbResponse from plugin_call_raw
/// - Must only be called once per response
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rb_response_free(response: *mut RbResponse) {
    unsafe {
        if !response.is_null() {
            (*response).free();
        }
    }
}

// ============================================================================
// Async API (future implementation)
// ============================================================================

// Type definitions for async API (for future implementation)

/// Completion callback for async requests
pub type CompletionCallbackFn = extern "C" fn(
    context: *mut c_void,
    request_id: u64,
    data: *const u8,
    len: usize,
    error_code: u32,
);

/// Make an async call to the plugin (placeholder for future implementation)
///
/// # Safety
/// - `handle` must be a valid handle from `plugin_init`, or null
/// - `type_tag` must be a valid null-terminated C string, or null
/// - `request` must be valid for `request_len` bytes, or null if `request_len` is 0
/// - `callback` will be invoked when the request completes
/// - `context` is passed through to the callback
///
/// # Returns
/// Request ID that can be used with plugin_cancel_async, or 0 if not implemented
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_call_async(
    _handle: FfiPluginHandle,
    _type_tag: *const std::ffi::c_char,
    _request: *const u8,
    _request_len: usize,
    _callback: CompletionCallbackFn,
    _context: *mut c_void,
) -> u64 {
    // TODO: Implement async call support
    0 // Return 0 to indicate not implemented
}

/// Cancel an async request (placeholder for future implementation)
///
/// # Safety
/// - `handle` must be a valid handle from `plugin_init`, or null
/// - `request_id` must be a valid request ID from `plugin_call_async`
///
/// # Returns
/// `true` if cancellation was successful, `false` otherwise
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_cancel_async(_handle: FfiPluginHandle, _request_id: u64) -> bool {
    // TODO: Implement async cancellation
    false
}

#[cfg(test)]
#[path = "exports/exports_tests.rs"]
mod exports_tests;

#[cfg(test)]
#[path = "exports/ffi_boundary_tests.rs"]
mod ffi_boundary_tests;
