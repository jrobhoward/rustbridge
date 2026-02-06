//! FFI bindings for loading and calling rustbridge plugins.
//!
//! These types and function signatures match the exports from `rustbridge-ffi`.

use std::ffi::{c_char, c_void};

/// FFI buffer for passing data across the boundary.
///
/// Matches `FfiBuffer` from `rustbridge-ffi`.
#[repr(C)]
pub struct FfiBuffer {
    /// Pointer to the data.
    pub data: *mut u8,
    /// Length of valid data in bytes.
    pub len: usize,
    /// Total capacity of the allocation.
    pub capacity: usize,
    /// Error code (0 = success).
    pub error_code: u32,
}

impl FfiBuffer {
    /// Check if this buffer represents an error.
    #[inline]
    pub fn is_error(&self) -> bool {
        self.error_code != 0
    }

    /// Check if this buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_null() || self.len == 0
    }

    /// Get the data as a slice.
    ///
    /// # Safety
    ///
    /// The buffer must contain valid data and not have been freed.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.len) }
        }
    }
}

/// FFI response buffer for binary transport.
///
/// Matches `RbResponse` from `rustbridge-ffi`.
#[repr(C)]
pub struct RbResponse {
    /// Error code (0 = success).
    pub error_code: u32,
    /// Size of response data in bytes.
    pub len: u32,
    /// Allocation capacity.
    pub capacity: u32,
    /// Pointer to response data (or error message).
    pub data: *mut c_void,
}

impl RbResponse {
    /// Check if this response indicates an error.
    #[inline]
    pub fn is_error(&self) -> bool {
        self.error_code != 0
    }

    /// Get the data as a slice.
    ///
    /// # Safety
    ///
    /// The response must contain valid data and not have been freed.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data as *const u8, self.len as usize) }
        }
    }
}

/// Log callback type matching `rustbridge-ffi`.
pub type LogCallback =
    Option<unsafe extern "C" fn(level: u8, target: *const c_char, message: *const c_char)>;

/// Opaque plugin handle.
pub type FfiPluginHandle = *mut c_void;

// ============================================================================
// FFI Function Signatures (loaded dynamically via libloading)
// ============================================================================

/// Create a new plugin instance.
///
/// Returns a pointer to an opaque plugin object that must be passed to `plugin_init`.
pub type PluginCreateFn = unsafe extern "C" fn() -> *mut c_void;

/// Initialize a plugin instance.
///
/// # Parameters
/// - `plugin_ptr`: Pointer from `plugin_create`
/// - `config_json`: JSON configuration bytes (can be null)
/// - `config_len`: Length of config_json
/// - `log_callback`: Optional log callback function
///
/// # Returns
/// Handle to the initialized plugin, or null on failure.
pub type PluginInitFn = unsafe extern "C" fn(
    plugin_ptr: *mut c_void,
    config_json: *const u8,
    config_len: usize,
    log_callback: LogCallback,
) -> FfiPluginHandle;

/// Make a synchronous JSON call to the plugin.
///
/// # Parameters
/// - `handle`: Plugin handle from `plugin_init`
/// - `type_tag`: Message type identifier (null-terminated C string)
/// - `request`: Request payload bytes
/// - `request_len`: Length of request payload
///
/// # Returns
/// FfiBuffer containing the response.
pub type PluginCallFn = unsafe extern "C" fn(
    handle: FfiPluginHandle,
    type_tag: *const c_char,
    request: *const u8,
    request_len: usize,
) -> FfiBuffer;

/// Make a synchronous binary call to the plugin.
///
/// # Parameters
/// - `handle`: Plugin handle from `plugin_init`
/// - `message_id`: Numeric message identifier
/// - `request`: Pointer to request data
/// - `request_size`: Size of request data
///
/// # Returns
/// RbResponse containing the binary response.
pub type PluginCallRawFn = unsafe extern "C" fn(
    handle: FfiPluginHandle,
    message_id: u32,
    request: *const c_void,
    request_size: usize,
) -> RbResponse;

/// Shutdown a plugin instance.
///
/// # Returns
/// true on success, false on failure.
pub type PluginShutdownFn = unsafe extern "C" fn(handle: FfiPluginHandle) -> bool;

/// Get the current state of a plugin.
///
/// # Returns
/// State code (0=Installed, 1=Starting, 2=Active, 3=Stopping, 4=Stopped, 5=Failed).
/// Returns 255 if handle is invalid.
pub type PluginGetStateFn = unsafe extern "C" fn(handle: FfiPluginHandle) -> u8;

/// Get the number of requests rejected due to concurrency limits.
pub type PluginGetRejectedCountFn = unsafe extern "C" fn(handle: FfiPluginHandle) -> u64;

/// Set the log level for a plugin.
///
/// # Parameters
/// - `handle`: Plugin handle
/// - `level`: Log level (0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error, 5=Off)
pub type PluginSetLogLevelFn = unsafe extern "C" fn(handle: FfiPluginHandle, level: u8);

/// Free a buffer returned by `plugin_call`.
pub type PluginFreeBufferFn = unsafe extern "C" fn(buffer: *mut FfiBuffer);

/// Free a response returned by `plugin_call_raw`.
pub type RbResponseFreeFn = unsafe extern "C" fn(response: *mut RbResponse);

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn FfiBuffer___is_error___returns_true_for_nonzero_code() {
        let buf = FfiBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            error_code: 1,
        };

        assert!(buf.is_error());
    }

    #[test]
    fn FfiBuffer___is_error___returns_false_for_zero_code() {
        let buf = FfiBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            error_code: 0,
        };

        assert!(!buf.is_error());
    }

    #[test]
    fn FfiBuffer___is_empty___returns_true_for_null_data() {
        let buf = FfiBuffer {
            data: std::ptr::null_mut(),
            len: 10,
            capacity: 10,
            error_code: 0,
        };

        assert!(buf.is_empty());
    }

    #[test]
    fn FfiBuffer___is_empty___returns_true_for_zero_len() {
        let mut data = [0u8; 10];
        let buf = FfiBuffer {
            data: data.as_mut_ptr(),
            len: 0,
            capacity: 10,
            error_code: 0,
        };

        assert!(buf.is_empty());
    }

    #[test]
    fn RbResponse___is_error___returns_true_for_nonzero_code() {
        let resp = RbResponse {
            error_code: 5,
            len: 0,
            capacity: 0,
            data: std::ptr::null_mut(),
        };

        assert!(resp.is_error());
    }

    #[test]
    fn RbResponse___is_error___returns_false_for_zero_code() {
        let resp = RbResponse {
            error_code: 0,
            len: 0,
            capacity: 0,
            data: std::ptr::null_mut(),
        };

        assert!(!resp.is_error());
    }
}
