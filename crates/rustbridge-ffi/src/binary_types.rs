//! FFI-safe binary types for C struct transport
//!
//! This module provides `#[repr(C)]` types that can be safely passed across the FFI
//! boundary for binary transport. These types are designed to be used as an alternative
//! to JSON serialization for performance-critical use cases.
//!
//! # Memory Ownership
//!
//! - **RbString**: Borrowed reference, caller owns the memory
//! - **RbBytes**: Borrowed reference, caller owns the memory
//! - **RbStringOwned**: Rust-owned string, must be freed via `rb_string_free`
//! - **RbBytesOwned**: Rust-owned bytes, must be freed via `rb_bytes_free`
//!
//! # Safety
//!
//! All types use explicit `#[repr(C)]` layout for predictable memory representation.
//! Pointer validity must be ensured by the caller for borrowed types.

use std::ffi::c_void;
use std::slice;

// ============================================================================
// Borrowed Types (caller-owned memory)
// ============================================================================

/// FFI-safe borrowed string reference
///
/// This is a view into a UTF-8 string that the caller owns. The string data
/// must remain valid for the duration of the FFI call.
///
/// # Memory Layout
///
/// ```text
/// +--------+--------+
/// |  len   |  data  |
/// | (u32)  | (*u8)  |
/// +--------+--------+
/// ```
///
/// # Invariants
///
/// - If `data` is non-null, it must point to valid UTF-8 bytes
/// - If `data` is non-null, it must be null-terminated (for C compatibility)
/// - `len` is the byte length, NOT including the null terminator
/// - If `len == 0` and `data == null`, the string is considered "not present" (None)
/// - If `len == 0` and `data != null`, the string is an empty string
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RbString {
    /// Length in bytes (excluding null terminator)
    pub len: u32,
    /// Pointer to null-terminated UTF-8 data
    pub data: *const u8,
}

impl RbString {
    /// Create an empty/absent string (represents None)
    #[inline]
    pub const fn none() -> Self {
        Self {
            len: 0,
            data: std::ptr::null(),
        }
    }

    /// Create a string from a static str
    ///
    /// # Safety
    ///
    /// The string must be null-terminated. Use this only with string literals
    /// or strings known to have a null terminator.
    #[inline]
    pub const fn from_static(s: &'static str) -> Self {
        Self {
            len: s.len() as u32,
            data: s.as_ptr(),
        }
    }

    /// Check if this string is present (not None)
    #[inline]
    pub fn is_present(&self) -> bool {
        !self.data.is_null()
    }

    /// Check if this string is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Convert to a Rust string slice
    ///
    /// # Safety
    ///
    /// - `data` must be valid for reads of `len` bytes
    /// - `data` must point to valid UTF-8
    /// - The memory must not be modified during the lifetime of the returned slice
    #[inline]
    pub unsafe fn as_str(&self) -> Option<&str> {
        if self.data.is_null() {
            return None;
        }
        let bytes = unsafe { slice::from_raw_parts(self.data, self.len as usize) };
        // SAFETY: Caller guarantees valid UTF-8
        Some(unsafe { std::str::from_utf8_unchecked(bytes) })
    }

    /// Convert to a Rust String (copies data)
    ///
    /// # Safety
    ///
    /// Same requirements as `as_str`
    #[inline]
    pub unsafe fn to_string(&self) -> Option<String> {
        unsafe { self.as_str().map(String::from) }
    }
}

// SAFETY: RbString is just a pointer + length, safe to send across threads
// The actual data it points to may not be, but that's the caller's responsibility
unsafe impl Send for RbString {}
unsafe impl Sync for RbString {}

impl Default for RbString {
    fn default() -> Self {
        Self::none()
    }
}

/// FFI-safe borrowed byte slice reference
///
/// This is a view into binary data that the caller owns. The data
/// must remain valid for the duration of the FFI call.
///
/// # Memory Layout
///
/// ```text
/// +--------+--------+
/// |  len   |  data  |
/// | (u32)  | (*u8)  |
/// +--------+--------+
/// ```
///
/// # Invariants
///
/// - If `data` is non-null, it must point to `len` valid bytes
/// - If `len == 0` and `data == null`, represents "not present" (None)
/// - Maximum size is 4GB (u32::MAX bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RbBytes {
    /// Length in bytes
    pub len: u32,
    /// Pointer to binary data
    pub data: *const u8,
}

impl RbBytes {
    /// Create an empty/absent byte slice (represents None)
    #[inline]
    pub const fn none() -> Self {
        Self {
            len: 0,
            data: std::ptr::null(),
        }
    }

    /// Create from a static byte slice
    #[inline]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Self {
            len: bytes.len() as u32,
            data: bytes.as_ptr(),
        }
    }

    /// Check if this byte slice is present (not None)
    #[inline]
    pub fn is_present(&self) -> bool {
        !self.data.is_null()
    }

    /// Check if this byte slice is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Convert to a Rust byte slice
    ///
    /// # Safety
    ///
    /// - `data` must be valid for reads of `len` bytes
    /// - The memory must not be modified during the lifetime of the returned slice
    #[inline]
    pub unsafe fn as_slice(&self) -> Option<&[u8]> {
        if self.data.is_null() {
            return None;
        }
        Some(unsafe { slice::from_raw_parts(self.data, self.len as usize) })
    }

    /// Convert to a Vec<u8> (copies data)
    ///
    /// # Safety
    ///
    /// Same requirements as `as_slice`
    #[inline]
    pub unsafe fn to_vec(&self) -> Option<Vec<u8>> {
        unsafe { self.as_slice().map(Vec::from) }
    }
}

// SAFETY: RbBytes is just a pointer + length, safe to send across threads
unsafe impl Send for RbBytes {}
unsafe impl Sync for RbBytes {}

impl Default for RbBytes {
    fn default() -> Self {
        Self::none()
    }
}

// ============================================================================
// Owned Types (Rust-owned memory, must be freed)
// ============================================================================

/// FFI-safe owned string
///
/// Unlike `RbString`, this type owns its memory and must be freed by calling
/// `rb_string_free`. This is used for strings returned from Rust to the host.
///
/// # Memory Ownership
///
/// - Memory is allocated by Rust
/// - Must be freed by calling `rb_string_free`
/// - Do NOT free with host language's free()
#[repr(C)]
#[derive(Debug)]
pub struct RbStringOwned {
    /// Length in bytes (excluding null terminator)
    pub len: u32,
    /// Pointer to null-terminated UTF-8 data (Rust-owned)
    pub data: *mut u8,
    /// Allocation capacity (for proper deallocation)
    pub capacity: u32,
}

impl RbStringOwned {
    /// Create an empty owned string
    #[inline]
    pub fn empty() -> Self {
        Self {
            len: 0,
            data: std::ptr::null_mut(),
            capacity: 0,
        }
    }

    /// Create from a Rust String (takes ownership)
    ///
    /// The string will be null-terminated for C compatibility.
    pub fn from_string(mut s: String) -> Self {
        // Ensure null termination
        s.push('\0');
        let len = s.len() - 1; // Exclude null terminator from len
        let capacity = s.capacity();
        let data = s.as_mut_ptr();
        std::mem::forget(s); // Prevent String destructor

        Self {
            len: len as u32,
            data,
            capacity: capacity as u32,
        }
    }

    /// Create from a string slice (copies data)
    pub fn from_slice(s: &str) -> Self {
        Self::from_string(s.to_string())
    }

    /// Convert to borrowed RbString
    #[inline]
    pub fn as_borrowed(&self) -> RbString {
        RbString {
            len: self.len,
            data: self.data,
        }
    }

    /// Free the owned string
    ///
    /// # Safety
    ///
    /// - Must only be called once
    /// - Must only be called on strings created by Rust
    pub unsafe fn free(&mut self) {
        if !self.data.is_null() && self.capacity > 0 {
            // Reconstruct the String and let it drop
            let _ = unsafe {
                String::from_raw_parts(self.data, (self.len + 1) as usize, self.capacity as usize)
            };
            self.data = std::ptr::null_mut();
            self.len = 0;
            self.capacity = 0;
        }
    }
}

// SAFETY: RbStringOwned owns its memory and can be sent across threads
unsafe impl Send for RbStringOwned {}

impl Default for RbStringOwned {
    fn default() -> Self {
        Self::empty()
    }
}

/// FFI-safe owned byte buffer
///
/// Unlike `RbBytes`, this type owns its memory and must be freed by calling
/// `rb_bytes_free`. This is used for binary data returned from Rust to the host.
///
/// # Memory Ownership
///
/// - Memory is allocated by Rust
/// - Must be freed by calling `rb_bytes_free`
/// - Do NOT free with host language's free()
#[repr(C)]
#[derive(Debug)]
pub struct RbBytesOwned {
    /// Length in bytes
    pub len: u32,
    /// Pointer to binary data (Rust-owned)
    pub data: *mut u8,
    /// Allocation capacity (for proper deallocation)
    pub capacity: u32,
}

impl RbBytesOwned {
    /// Create an empty owned byte buffer
    #[inline]
    pub fn empty() -> Self {
        Self {
            len: 0,
            data: std::ptr::null_mut(),
            capacity: 0,
        }
    }

    /// Create from a Vec<u8> (takes ownership)
    pub fn from_vec(mut v: Vec<u8>) -> Self {
        let len = v.len();
        let capacity = v.capacity();
        let data = v.as_mut_ptr();
        std::mem::forget(v); // Prevent Vec destructor

        Self {
            len: len as u32,
            data,
            capacity: capacity as u32,
        }
    }

    /// Create from a byte slice (copies data)
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self::from_vec(bytes.to_vec())
    }

    /// Convert to borrowed RbBytes
    #[inline]
    pub fn as_borrowed(&self) -> RbBytes {
        RbBytes {
            len: self.len,
            data: self.data,
        }
    }

    /// Free the owned bytes
    ///
    /// # Safety
    ///
    /// - Must only be called once
    /// - Must only be called on buffers created by Rust
    pub unsafe fn free(&mut self) {
        if !self.data.is_null() && self.capacity > 0 {
            // Reconstruct the Vec and let it drop
            let _ = unsafe {
                Vec::from_raw_parts(self.data, self.len as usize, self.capacity as usize)
            };
            self.data = std::ptr::null_mut();
            self.len = 0;
            self.capacity = 0;
        }
    }
}

// SAFETY: RbBytesOwned owns its memory and can be sent across threads
unsafe impl Send for RbBytesOwned {}

impl Default for RbBytesOwned {
    fn default() -> Self {
        Self::empty()
    }
}

// ============================================================================
// FFI Response Buffer for Binary Transport
// ============================================================================

/// FFI buffer for binary transport responses
///
/// Similar to `FfiBuffer` but designed specifically for binary struct responses.
/// The response data is a raw C struct that can be cast directly by the host.
///
/// # Memory Layout
///
/// ```text
/// +------------+--------+----------+------------+
/// | error_code |  len   | capacity |    data    |
/// |   (u32)    | (u32)  |  (u32)   | (*mut u8)  |
/// +------------+--------+----------+------------+
/// ```
///
/// # Usage
///
/// - `error_code == 0`: Success, `data` points to response struct
/// - `error_code != 0`: Error, `data` may point to error message
#[repr(C)]
#[derive(Debug)]
pub struct RbResponse {
    /// Error code (0 = success)
    pub error_code: u32,
    /// Size of response data in bytes
    pub len: u32,
    /// Allocation capacity
    pub capacity: u32,
    /// Pointer to response data (or error message)
    pub data: *mut c_void,
}

impl RbResponse {
    /// Create a successful response with struct data
    ///
    /// # Safety
    ///
    /// The type T must be `#[repr(C)]` and safe to transmit across FFI.
    pub fn success<T: Sized>(value: T) -> Self {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Allocate aligned memory
        #[allow(clippy::expect_used)] // Safe: Layout is valid for any Sized type from std::mem
        let layout = std::alloc::Layout::from_size_align(size, align)
            .expect("Invalid layout for response type");

        // SAFETY: We just created a valid layout
        let data = unsafe { std::alloc::alloc(layout) as *mut c_void };

        if data.is_null() {
            return Self::error(11, "Failed to allocate response buffer");
        }

        // Copy the value into the allocated memory
        // SAFETY: data is valid, aligned, and sized for T
        unsafe {
            std::ptr::write(data as *mut T, value);
        }

        Self {
            error_code: 0,
            len: size as u32,
            capacity: size as u32,
            data,
        }
    }

    /// Create an error response
    pub fn error(code: u32, message: &str) -> Self {
        let mut msg = message.as_bytes().to_vec();
        msg.push(0); // Null terminate

        let len = msg.len();
        let capacity = msg.capacity();
        let data = msg.as_mut_ptr();
        std::mem::forget(msg);

        Self {
            error_code: code,
            len: len as u32,
            capacity: capacity as u32,
            data: data as *mut c_void,
        }
    }

    /// Create an empty response (for invalid calls)
    pub fn empty() -> Self {
        Self {
            error_code: 0,
            len: 0,
            capacity: 0,
            data: std::ptr::null_mut(),
        }
    }

    /// Check if this response indicates an error
    #[inline]
    pub fn is_error(&self) -> bool {
        self.error_code != 0
    }

    /// Free the response buffer
    ///
    /// # Safety
    ///
    /// - Must only be called once
    /// - Must only be called on responses created by Rust
    /// - For success responses, the original type's size must match `len`
    pub unsafe fn free(&mut self) {
        if !self.data.is_null() && self.capacity > 0 {
            if self.error_code != 0 {
                // Error message is a Vec<u8>
                let _ = unsafe {
                    Vec::from_raw_parts(
                        self.data as *mut u8,
                        self.len as usize,
                        self.capacity as usize,
                    )
                };
            } else {
                // Success data was allocated with alloc
                // SAFETY: capacity was set from a valid layout during allocation
                let layout = unsafe {
                    std::alloc::Layout::from_size_align_unchecked(
                        self.capacity as usize,
                        std::mem::align_of::<usize>(), // Conservative alignment
                    )
                };
                unsafe {
                    std::alloc::dealloc(self.data as *mut u8, layout);
                }
            }
            self.data = std::ptr::null_mut();
            self.len = 0;
            self.capacity = 0;
        }
    }

    /// Get the response data as a typed reference
    ///
    /// # Safety
    ///
    /// - Response must be successful (error_code == 0)
    /// - Type T must match the type used to create the response
    /// - T must be `#[repr(C)]`
    #[inline]
    pub unsafe fn as_ref<T: Sized>(&self) -> Option<&T> {
        if self.is_error() || self.data.is_null() {
            return None;
        }
        Some(unsafe { &*(self.data as *const T) })
    }
}

// SAFETY: RbResponse owns its data and can be sent across threads
unsafe impl Send for RbResponse {}

impl Default for RbResponse {
    fn default() -> Self {
        Self::empty()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "binary_types/binary_types_tests.rs"]
mod binary_types_tests;
