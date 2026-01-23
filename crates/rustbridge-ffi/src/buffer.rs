//! FFI buffer for passing data across the boundary

use std::ptr;

/// Buffer for passing data across FFI boundary
///
/// This structure follows the "Rust allocates, host frees" pattern:
/// - Rust creates the buffer and populates it with data
/// - Host copies the data to its managed heap
/// - Host calls `plugin_free_buffer` to release the memory
///
/// # Memory Safety
///
/// The buffer owns its memory. When `plugin_free_buffer` is called, the
/// memory is deallocated. The host must not use the buffer after freeing it.
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
    /// Create an empty buffer
    pub fn empty() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
            capacity: 0,
            error_code: 0,
        }
    }

    /// Create a buffer from a Vec
    ///
    /// This transfers ownership of the Vec's memory to the buffer.
    pub fn from_vec(mut vec: Vec<u8>) -> Self {
        let data = vec.as_mut_ptr();
        let len = vec.len();
        let capacity = vec.capacity();

        // Prevent Vec from deallocating the memory
        std::mem::forget(vec);

        Self {
            data,
            len,
            capacity,
            error_code: 0,
        }
    }

    /// Create an error buffer
    ///
    /// The error message is stored in the buffer data.
    pub fn error(code: u32, message: &str) -> Self {
        let mut buffer = Self::from_vec(message.as_bytes().to_vec());
        buffer.error_code = code;
        buffer
    }

    /// Create a success buffer with JSON data
    pub fn success_json<T: serde::Serialize>(value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(data) => Self::from_vec(data),
            Err(e) => Self::error(5, &format!("Serialization error: {}", e)),
        }
    }

    /// Check if this buffer represents an error
    pub fn is_error(&self) -> bool {
        self.error_code != 0
    }

    /// Check if this buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_null() || self.len == 0
    }

    /// Get the data as a slice (unsafe - caller must ensure buffer is valid)
    ///
    /// # Safety
    ///
    /// The buffer must contain valid data and not have been freed.
    pub unsafe fn as_slice(&self) -> &[u8] {
        unsafe {
            if self.data.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(self.data, self.len)
            }
        }
    }

    /// Free the buffer's memory
    ///
    /// # Safety
    ///
    /// This must only be called once per buffer. After calling, the buffer
    /// is invalid and must not be used.
    pub unsafe fn free(&mut self) {
        unsafe {
            if !self.data.is_null() && self.capacity > 0 {
                // Reconstruct the Vec and let it drop
                let _ = Vec::from_raw_parts(self.data, self.len, self.capacity);
            }
            self.data = ptr::null_mut();
            self.len = 0;
            self.capacity = 0;
        }
    }
}

impl Default for FfiBuffer {
    fn default() -> Self {
        Self::empty()
    }
}

// FfiBuffer is not Clone because it owns memory
// FfiBuffer is Send because it owns its data
unsafe impl Send for FfiBuffer {}

#[cfg(test)]
#[path = "buffer/buffer_tests.rs"]
mod buffer_tests;
