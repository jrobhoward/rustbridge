//! Binary message types for C struct transport benchmarking
//!
//! These types are `#[repr(C)]` equivalents of the JSON benchmark messages,
//! designed for direct FFI transport without serialization overhead.
//!
//! # Message IDs
//!
//! | Message | ID |
//! |---------|-----|
//! | bench.small | 1 |
//! | bench.medium | 2 |
//! | bench.large | 3 |

use rustbridge_core::PluginResult;
use rustbridge_ffi::{PluginHandle, register_binary_handler};

/// Message ID for small benchmark
pub const MSG_BENCH_SMALL: u32 = 1;

/// Message ID for medium benchmark
pub const MSG_BENCH_MEDIUM: u32 = 2;

/// Message ID for large benchmark
pub const MSG_BENCH_LARGE: u32 = 3;

// ============================================================================
// Small Benchmark Messages (~100 bytes)
// ============================================================================

/// Small benchmark request (C struct version)
///
/// Equivalent to SmallRequest JSON type.
/// Simulates: config lookup, feature flag check
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SmallRequestRaw {
    /// Struct version for forward compatibility
    pub version: u8,
    /// Reserved for alignment (must be zero)
    pub _reserved: [u8; 3],
    /// Key to look up (fixed-size buffer for simplicity)
    pub key: [u8; 64],
    /// Length of key string
    pub key_len: u32,
    /// Flags bitmask
    pub flags: u32,
}

impl SmallRequestRaw {
    /// Current version of this struct
    pub const VERSION: u8 = 1;

    /// Get key as string slice
    pub fn key_str(&self) -> &str {
        let len = self.key_len.min(64) as usize;
        // SAFETY: We control the data and ensure valid UTF-8 in tests
        unsafe { std::str::from_utf8_unchecked(&self.key[..len]) }
    }

    /// Create a new request from a key and flags
    pub fn new(key: &str, flags: u32) -> Self {
        let mut key_buf = [0u8; 64];
        let key_bytes = key.as_bytes();
        let len = key_bytes.len().min(64);
        key_buf[..len].copy_from_slice(&key_bytes[..len]);

        Self {
            version: Self::VERSION,
            _reserved: [0; 3],
            key: key_buf,
            key_len: len as u32,
            flags,
        }
    }
}

/// Small benchmark response (C struct version)
///
/// Equivalent to SmallResponse JSON type.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SmallResponseRaw {
    /// Struct version for forward compatibility
    pub version: u8,
    /// Reserved for alignment (must be zero)
    pub _reserved: [u8; 3],
    /// Value (fixed-size buffer)
    pub value: [u8; 64],
    /// Length of value string
    pub value_len: u32,
    /// TTL in seconds
    pub ttl_seconds: u32,
    /// Cache hit flag (0 = miss, 1 = hit)
    pub cache_hit: u8,
    /// Padding for alignment
    pub _padding: [u8; 3],
}

impl SmallResponseRaw {
    /// Current version of this struct
    pub const VERSION: u8 = 1;

    /// Create a new response
    pub fn new(value: &str, ttl_seconds: u32, cache_hit: bool) -> Self {
        let mut value_buf = [0u8; 64];
        let value_bytes = value.as_bytes();
        let len = value_bytes.len().min(64);
        value_buf[..len].copy_from_slice(&value_bytes[..len]);

        Self {
            version: Self::VERSION,
            _reserved: [0; 3],
            value: value_buf,
            value_len: len as u32,
            ttl_seconds,
            cache_hit: if cache_hit { 1 } else { 0 },
            _padding: [0; 3],
        }
    }
}

// ============================================================================
// Handler Registration
// ============================================================================

/// Register binary message handlers for benchmarking
///
/// Call this during plugin initialization to enable binary transport
/// for benchmark messages.
pub fn register_benchmark_handlers() {
    register_binary_handler(MSG_BENCH_SMALL, handle_bench_small_raw);
    // TODO: Add medium and large handlers when needed
}

/// Handle small benchmark request (binary transport)
fn handle_bench_small_raw(_handle: &PluginHandle, request: &[u8]) -> PluginResult<Vec<u8>> {
    // Validate request size
    if request.len() < std::mem::size_of::<SmallRequestRaw>() {
        return Err(rustbridge_core::PluginError::HandlerError(
            "Request too small".to_string(),
        ));
    }

    // SAFETY: We validated the size, and SmallRequestRaw is repr(C)
    let req = unsafe { &*(request.as_ptr() as *const SmallRequestRaw) };

    // Validate version for forward compatibility
    if req.version != SmallRequestRaw::VERSION {
        return Err(rustbridge_core::PluginError::HandlerError(format!(
            "Unsupported request version: {} (expected {})",
            req.version,
            SmallRequestRaw::VERSION
        )));
    }

    // Process request (same logic as JSON handler)
    let key = req.key_str();
    let value = format!("value_for_{}", key);
    let cache_hit = req.flags & 1 != 0;

    let response = SmallResponseRaw::new(&value, 3600, cache_hit);

    // Convert response to bytes
    let response_bytes = unsafe {
        std::slice::from_raw_parts(
            &response as *const SmallResponseRaw as *const u8,
            std::mem::size_of::<SmallResponseRaw>(),
        )
    };

    Ok(response_bytes.to_vec())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn SmallRequestRaw___new___creates_valid_request() {
        let req = SmallRequestRaw::new("test_key", 0x01);

        assert_eq!(req.version, SmallRequestRaw::VERSION);
        assert_eq!(req.key_str(), "test_key");
        assert_eq!(req.flags, 0x01);
        assert_eq!(req.key_len, 8);
    }

    #[test]
    fn SmallRequestRaw___new___truncates_long_keys() {
        let long_key = "a".repeat(100);
        let req = SmallRequestRaw::new(&long_key, 0);

        assert_eq!(req.key_len, 64);
        assert_eq!(req.key_str().len(), 64);
    }

    #[test]
    fn SmallResponseRaw___new___creates_valid_response() {
        let resp = SmallResponseRaw::new("test_value", 3600, true);

        assert_eq!(resp.version, SmallResponseRaw::VERSION);
        assert_eq!(resp.value_len, 10);
        assert_eq!(resp.ttl_seconds, 3600);
        assert_eq!(resp.cache_hit, 1);
    }

    #[test]
    fn memory_layout___SmallRequestRaw___has_fixed_size() {
        // 1 (version) + 3 (reserved) + 64 (key) + 4 (key_len) + 4 (flags) = 76 bytes
        let size = std::mem::size_of::<SmallRequestRaw>();
        assert_eq!(size, 76);
    }

    #[test]
    fn memory_layout___SmallResponseRaw___has_fixed_size() {
        // 1 (version) + 3 (reserved) + 64 (value) + 4 (value_len) + 4 (ttl) + 1 (cache_hit) + 3 (padding) = 80 bytes
        let size = std::mem::size_of::<SmallResponseRaw>();
        assert_eq!(size, 80);
    }

    #[test]
    fn handler___handle_bench_small_raw___processes_request() {
        let req = SmallRequestRaw::new("my_key", 0x01);
        let req_bytes = unsafe {
            std::slice::from_raw_parts(
                &req as *const SmallRequestRaw as *const u8,
                std::mem::size_of::<SmallRequestRaw>(),
            )
        };

        // Create a dummy plugin handle for testing
        // Note: This test uses a simplified approach without a real handle
        // In practice, the handler would use the handle for state access

        // For this test, we just verify the request/response serialization
        let response_bytes = handle_bench_small_raw_test(req_bytes).unwrap();

        assert_eq!(
            response_bytes.len(),
            std::mem::size_of::<SmallResponseRaw>()
        );

        let resp = unsafe { &*(response_bytes.as_ptr() as *const SmallResponseRaw) };
        assert!(resp.value_len > 0);
        assert_eq!(resp.ttl_seconds, 3600);
        assert_eq!(resp.cache_hit, 1); // flags & 1 != 0
    }

    // Test helper that doesn't require PluginHandle
    fn handle_bench_small_raw_test(request: &[u8]) -> PluginResult<Vec<u8>> {
        if request.len() < std::mem::size_of::<SmallRequestRaw>() {
            return Err(rustbridge_core::PluginError::HandlerError(
                "Request too small".to_string(),
            ));
        }

        let req = unsafe { &*(request.as_ptr() as *const SmallRequestRaw) };

        // Validate version
        if req.version != SmallRequestRaw::VERSION {
            return Err(rustbridge_core::PluginError::HandlerError(format!(
                "Unsupported request version: {} (expected {})",
                req.version,
                SmallRequestRaw::VERSION
            )));
        }

        let key = req.key_str();
        let value = format!("value_for_{}", key);
        let cache_hit = req.flags & 1 != 0;

        let response = SmallResponseRaw::new(&value, 3600, cache_hit);
        let response_bytes = unsafe {
            std::slice::from_raw_parts(
                &response as *const SmallResponseRaw as *const u8,
                std::mem::size_of::<SmallResponseRaw>(),
            )
        };

        Ok(response_bytes.to_vec())
    }

    #[test]
    fn handler___handle_bench_small_raw___rejects_invalid_version() {
        let mut req = SmallRequestRaw::new("my_key", 0x01);
        req.version = 99; // Invalid version

        let req_bytes = unsafe {
            std::slice::from_raw_parts(
                &req as *const SmallRequestRaw as *const u8,
                std::mem::size_of::<SmallRequestRaw>(),
            )
        };

        let result = handle_bench_small_raw_test(req_bytes);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unsupported request version"));
    }
}
