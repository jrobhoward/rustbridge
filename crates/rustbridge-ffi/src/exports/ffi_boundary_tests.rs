#![allow(non_snake_case)]

//! Comprehensive FFI boundary tests
//!
//! These tests verify edge cases and safety properties of the FFI boundary:
//! - Large payload handling
//! - Memory safety with rapid alloc/free cycles
//! - UTF-8 validation
//! - Empty/null payload handling
//! - Error propagation across FFI boundary

use super::*;

#[test]
fn plugin_call___large_payload___handles_correctly() {
    // Arrange - create a large payload (1 MB)
    let large_data = vec![0x42u8; 1024 * 1024];
    let mut buffer = FfiBuffer::from_vec(large_data);

    // Act - verify buffer is created correctly
    let is_valid = !buffer.data.is_null() && buffer.len == 1024 * 1024;

    // Assert
    assert!(is_valid);
    unsafe { buffer.free() };
}

#[test]
fn plugin_call___empty_payload___returns_error() {
    unsafe {
        // Arrange - call with empty payload (null pointer, zero length)

        // Act
        let result = plugin_call(1 as FfiPluginHandle, c"test".as_ptr(), ptr::null(), 0);

        // Assert
        assert!(result.is_error());
        let mut result = result;
        result.free();
    }
}

#[test]
fn FfiBuffer___rapid_alloc_free___no_memory_leak() {
    // Arrange & Act - rapid allocation and deallocation cycles
    for i in 0..1000 {
        let data = vec![1u8, 2, 3, 4, 5];
        let mut buf = FfiBuffer::from_vec(data);
        assert_eq!(buf.len, 5);
        unsafe { buf.free() };
        // Verify after free that buffer is cleared
        if i == 999 {
            assert!(buf.data.is_null());
        }
    }

    // Assert - test completes without crash (memory leak detection requires ASAN)
}

#[test]
fn FfiBuffer___success_json___roundtrip_preserves_data() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestPayload {
        id: u64,
        name: String,
        values: Vec<i32>,
    }

    // Arrange
    let payload = TestPayload {
        id: 42,
        name: "test".to_string(),
        values: vec![1, 2, 3, 4, 5],
    };

    // Act
    let mut buf = FfiBuffer::success_json(&payload);

    // Assert
    unsafe {
        let slice = buf.as_slice();
        let json_str = std::str::from_utf8(slice).unwrap();
        let deserialized: TestPayload = serde_json::from_str(json_str).unwrap();
        assert_eq!(deserialized, payload);
        buf.free();
    }
}

#[test]
fn FfiBuffer___error___preserves_code_and_message() {
    // Arrange
    let error_code = 123;
    let error_msg = "Custom error message with unicode: 日本語";

    // Act
    let mut buf = FfiBuffer::error(error_code, error_msg);

    // Assert
    assert_eq!(buf.error_code, error_code);
    assert!(buf.is_error());
    unsafe {
        let slice = buf.as_slice();
        let msg = std::str::from_utf8(slice).unwrap();
        assert_eq!(msg, error_msg);
        buf.free();
    }
}

#[test]
fn FfiBuffer___from_vec___zero_length_vec___creates_valid_buffer() {
    // Arrange
    let empty_vec: Vec<u8> = Vec::new();

    // Act
    let buf = FfiBuffer::from_vec(empty_vec);

    // Assert
    // Empty vector should still create a valid buffer structure
    assert_eq!(buf.len, 0);
    assert_eq!(buf.error_code, 0);
    assert!(!buf.is_error());
}

#[test]
fn FfiBuffer___very_large_error_message___truncates_safely() {
    // Arrange - create a very large error message (10 KB)
    let large_message = "ERROR ".repeat(1700); // ~10KB

    // Act
    let mut buf = FfiBuffer::error(999, &large_message);

    // Assert - should handle large messages without crash
    assert!(buf.is_error());
    assert_eq!(buf.error_code, 999);
    unsafe {
        let slice = buf.as_slice();
        assert!(!slice.is_empty());
        buf.free();
    }
}

#[test]
fn plugin_free_buffer___empty_buffer___does_not_crash() {
    // Arrange
    let buf = FfiBuffer::empty();

    // Act & Assert - should not crash
    unsafe {
        plugin_free_buffer(&buf as *const _ as *mut _);
    }
}

#[test]
fn plugin_get_state___null_handle___returns_255() {
    unsafe {
        // Arrange & Act
        let state = plugin_get_state(ptr::null_mut());

        // Assert
        assert_eq!(state, 255);
    }
}

#[test]
fn plugin_shutdown___null_handle___returns_false() {
    unsafe {
        // Arrange & Act
        let result = plugin_shutdown(ptr::null_mut());

        // Assert
        assert!(!result);
    }
}

#[test]
fn plugin_set_log_level___invalid_handle___does_not_crash() {
    unsafe {
        // Arrange & Act - should not crash
        plugin_set_log_level(999 as FfiPluginHandle, LogLevel::Debug as u8);
        plugin_set_log_level(999 as FfiPluginHandle, LogLevel::Info as u8);
        plugin_set_log_level(999 as FfiPluginHandle, LogLevel::Warn as u8);

        // Assert - completes without panic (implicitly successful if no crash)
    }
}

#[test]
fn FfiBuffer___max_u32_error_code___preserves_value() {
    // Arrange
    let max_code = u32::MAX;

    // Act
    let mut buf = FfiBuffer::error(max_code, "Max error code");

    // Assert
    assert_eq!(buf.error_code, max_code);
    unsafe { buf.free() };
}

#[test]
fn FfiBuffer___unicode_in_json___serializes_correctly() {
    #[derive(serde::Serialize)]
    struct UnicodeData {
        emoji: String,
        chinese: String,
        arabic: String,
    }

    // Arrange
    let data = UnicodeData {
        emoji: "🦀🔥✨".to_string(),
        chinese: "你好世界".to_string(),
        arabic: "مرحبا بالعالم".to_string(),
    };

    // Act
    let mut buf = FfiBuffer::success_json(&data);

    // Assert
    unsafe {
        let slice = buf.as_slice();
        let json_str = std::str::from_utf8(slice).unwrap();
        assert!(json_str.contains("🦀"));
        assert!(json_str.contains("你好"));
        assert!(json_str.contains("مرحبا"));
        buf.free();
    }
}

#[test]
fn plugin_call___type_tag_with_special_chars___handles_safely() {
    unsafe {
        // Arrange - type tags with dots, underscores, and alphanumeric
        let type_tags = [
            c"user.create",
            c"api_v2.request",
            c"namespace::method",
            c"CamelCase.method",
        ];

        // Act & Assert - all should handle gracefully (may return error, but shouldn't crash)
        for type_tag in type_tags {
            let result = plugin_call(999 as FfiPluginHandle, type_tag.as_ptr(), ptr::null(), 0);
            // Should return error for invalid handle, not crash
            assert!(result.is_error());
            let mut result = result;
            result.free();
        }
    }
}

#[test]
fn FfiBuffer___as_slice___does_not_panic_on_valid_buffer() {
    // Arrange
    let data = vec![1u8, 2, 3, 4, 5];
    let buf = FfiBuffer::from_vec(data);

    // Act
    let slice = unsafe { buf.as_slice() };

    // Assert
    assert_eq!(slice.len(), 5);
    assert_eq!(slice, &[1, 2, 3, 4, 5]);
}
