//! Local FFI type definitions
//!
//! These types must match the layout in rustbridge-ffi exactly.
//! We define them locally to avoid linking against rustbridge-ffi,
//! which would create symbol resolution conflicts with the plugin's
//! own rustbridge-ffi linkage.

use std::ptr;

/// Buffer for passing data across FFI boundary
///
/// This structure must match the layout of `rustbridge_ffi::FfiBuffer` exactly.
/// It follows the "Rust allocates, host frees" pattern.
#[repr(C)]
pub struct FfiBuffer {
    /// Pointer to the data
    pub data: *mut u8,
    /// Length of valid data in bytes
    pub len: usize,
    /// Total capacity of the allocation
    pub capacity: usize,
    /// Error code (0 = success)
    pub error_code: u32,
}

impl FfiBuffer {
    /// Check if this buffer represents an error
    pub fn is_error(&self) -> bool {
        self.error_code != 0
    }

    /// Get the data as a slice (unsafe - caller must ensure buffer is valid)
    ///
    /// # Safety
    ///
    /// The buffer must contain valid data and not have been freed.
    pub unsafe fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            // SAFETY: The caller guarantees that the buffer contains valid data
            unsafe { std::slice::from_raw_parts(self.data, self.len) }
        }
    }

    /// Free the buffer's memory
    ///
    /// # Safety
    ///
    /// This must only be called once per buffer. After calling, the buffer
    /// is invalid and must not be used.
    pub unsafe fn free(&mut self) {
        if !self.data.is_null() && self.capacity > 0 {
            // Reconstruct the Vec and let it drop
            // SAFETY: The buffer was created from a Vec with these exact parameters
            unsafe {
                let _ = Vec::from_raw_parts(self.data, self.len, self.capacity);
            }
        }
        self.data = ptr::null_mut();
        self.len = 0;
        self.capacity = 0;
    }
}

/// Log callback function type
///
/// This must match `rustbridge_ffi::LogCallback` exactly.
pub type LogCallback = unsafe extern "C" fn(
    level: u8,
    target: *const std::ffi::c_char,
    message: *const std::ffi::c_char,
);

/// Response structure for binary transport
///
/// This structure must match the layout of `rustbridge_ffi::RbResponse` exactly.
#[repr(C)]
pub struct RbResponse {
    /// Error code (0 = success)
    pub error_code: u32,
    /// Length of valid data
    pub len: u32,
    /// Total capacity of the allocation
    pub capacity: u32,
    /// Padding for alignment (8-byte aligned pointer)
    pub _padding: u32,
    /// Pointer to the data
    pub data: *mut u8,
}

impl RbResponse {
    /// Check if this response represents an error
    pub fn is_error(&self) -> bool {
        self.error_code != 0
    }

    /// Get the data as a slice (unsafe - caller must ensure buffer is valid)
    ///
    /// # Safety
    ///
    /// The response must contain valid data and not have been freed.
    pub unsafe fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            // SAFETY: The caller guarantees that the buffer contains valid data
            unsafe { std::slice::from_raw_parts(self.data, self.len as usize) }
        }
    }

    /// Free the response's memory
    ///
    /// # Safety
    ///
    /// This must only be called once per response. After calling, the response
    /// is invalid and must not be used.
    pub unsafe fn free(&mut self) {
        if !self.data.is_null() && self.capacity > 0 {
            // Reconstruct the Vec and let it drop
            // SAFETY: The buffer was created from a Vec with these exact parameters
            unsafe {
                let _ = Vec::from_raw_parts(self.data, self.len as usize, self.capacity as usize);
            }
        }
        self.data = ptr::null_mut();
        self.len = 0;
        self.capacity = 0;
    }
}
