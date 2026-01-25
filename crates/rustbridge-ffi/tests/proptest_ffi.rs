//! Property-based tests for FFI boundary safety
//!
//! Tests that any valid input can be safely allocated, passed, and freed
//! across the FFI boundary without panics or memory corruption.

use proptest::prelude::*;
use rustbridge_ffi::FfiBuffer;

// Strategy: Generate any valid byte vector (0-10KB for reasonable test speed)
fn arb_byte_vector() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..10_240)
}

proptest! {
    /// Property: Any byte vector can be converted to an FfiBuffer and freed safely
    #[test]
    fn proptest_buffer_from_vec_allocate_free(vec in arb_byte_vector()) {
        let original_len = vec.len();

        // Create buffer from vec
        let mut buffer = FfiBuffer::from_vec(vec);

        // Verify buffer has correct metadata
        prop_assert_eq!(buffer.len, original_len);
        prop_assert!(buffer.capacity >= original_len, "capacity must be >= len");
        prop_assert!(!buffer.data.is_null());
        prop_assert_eq!(buffer.error_code, 0);

        // Verify data is preserved
        unsafe {
            let slice = buffer.as_slice();
            prop_assert_eq!(slice.len(), original_len);
        }

        // Free should not panic
        unsafe {
            buffer.free();
        }

        // After free, buffer should be cleared
        prop_assert!(buffer.data.is_null());
        prop_assert_eq!(buffer.len, 0);
        prop_assert_eq!(buffer.capacity, 0);
    }

    /// Property: Any valid UTF-8 string can be used as an error message
    #[test]
    fn proptest_error_buffer_with_utf8_strings(text in ".*") {
        let mut buffer = FfiBuffer::error(42, &text);

        prop_assert_eq!(buffer.error_code, 42);

        // Verify we can read the error message back
        unsafe {
            let slice = buffer.as_slice();
            let recovered = std::str::from_utf8(slice);
            prop_assert!(recovered.is_ok());
            prop_assert_eq!(recovered.unwrap(), text);
        }

        // Free should work safely
        unsafe {
            buffer.free();
        }
    }

    /// Property: Multiple sequential alloc/free cycles don't corrupt memory
    #[test]
    fn proptest_sequential_allocate_free_cycles(
        vectors in prop::collection::vec(arb_byte_vector(), 1..20),
    ) {
        for vec in vectors {
            let original_len = vec.len();
            let mut buffer = FfiBuffer::from_vec(vec);

            prop_assert_eq!(buffer.len, original_len);
            prop_assert!(!buffer.data.is_null());

            unsafe {
                buffer.free();
            }

            prop_assert!(buffer.data.is_null());
        }
    }

    /// Property: Buffer can store any byte pattern, not just valid UTF-8
    #[test]
    fn proptest_binary_data_preservation(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let original_bytes = bytes.clone();
        let len = bytes.len();

        let mut buffer = FfiBuffer::from_vec(bytes);

        unsafe {
            let slice = buffer.as_slice();
            prop_assert_eq!(slice, &original_bytes[..len]);
        }

        unsafe {
            buffer.free();
        }
    }
}

#[test]
fn test_empty_buffer_operations() {
    let mut buffer = FfiBuffer::empty();

    assert!(buffer.is_empty());
    assert_eq!(buffer.error_code, 0);

    unsafe {
        let slice = buffer.as_slice();
        assert_eq!(slice.len(), 0);
    }

    // Freeing an empty buffer should be safe
    unsafe {
        buffer.free();
    }

    assert!(buffer.data.is_null());
}

#[test]
fn test_empty_buffer_from_vec() {
    let buffer = FfiBuffer::from_vec(vec![]);

    assert_eq!(buffer.len, 0);
    assert_eq!(buffer.capacity, 0);
    assert!(buffer.is_empty());

    unsafe {
        let slice = buffer.as_slice();
        assert_eq!(slice.len(), 0);
    }
}
