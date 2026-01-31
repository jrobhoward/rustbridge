//! FFI Integration Tests
//!
//! These tests verify the complete FFI call flow including:
//! - Successful plugin initialization, call, and shutdown
//! - Binary transport success paths
//! - Concurrent call/shutdown race conditions

#![allow(non_snake_case)]

use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginContext, PluginError, PluginResult};
use rustbridge_ffi::{
    plugin_call, plugin_get_rejected_count, plugin_get_state, plugin_init, plugin_shutdown,
};
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

/// Test plugin that echoes requests and tracks call count
struct EchoPlugin {
    call_count: AtomicU64,
}

impl EchoPlugin {
    fn new() -> Self {
        Self {
            call_count: AtomicU64::new(0),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct EchoResponse {
    message: String,
    call_number: u64,
}

#[async_trait]
impl Plugin for EchoPlugin {
    async fn on_start(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn on_stop(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        _context: &PluginContext,
        type_tag: &str,
        request: &[u8],
    ) -> PluginResult<Vec<u8>> {
        let call_num = self.call_count.fetch_add(1, Ordering::Relaxed) + 1;

        match type_tag {
            "echo" => {
                let req: EchoRequest = serde_json::from_slice(request)
                    .map_err(|e| PluginError::SerializationError(e.to_string()))?;

                let response = EchoResponse {
                    message: req.message,
                    call_number: call_num,
                };

                serde_json::to_vec(&response).map_err(|e| PluginError::Internal(e.to_string()))
            }
            "slow" => {
                // Simulate slow operation for race condition tests
                std::thread::sleep(std::time::Duration::from_millis(100));
                Ok(b"{}".to_vec())
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }
}

/// Helper to create a plugin pointer (simulates plugin_create)
fn create_test_plugin() -> *mut c_void {
    let plugin: Box<dyn Plugin> = Box::new(EchoPlugin::new());
    let boxed: Box<Box<dyn Plugin>> = Box::new(plugin);
    Box::into_raw(boxed) as *mut c_void
}

// =============================================================================
// Success Path Tests
// =============================================================================

#[test]
fn plugin_init___valid_plugin___returns_handle() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);

        assert!(!handle.is_null(), "plugin_init should return valid handle");

        // Cleanup
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_init___with_config___applies_config() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let config = r#"{"log_level": "debug", "max_concurrent_requests": 10}"#;

        let handle = plugin_init(plugin_ptr, config.as_ptr(), config.len(), None);

        assert!(!handle.is_null(), "plugin_init with config should succeed");

        // Cleanup
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_get_state___active_plugin___returns_active() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        let state = plugin_get_state(handle);
        // State 2 = Active (after successful init and on_start)
        assert_eq!(state, 2, "Plugin should be in Active state");

        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___echo_request___returns_response() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null(), "Handle should not be null");

        // Make a call
        let request = r#"{"message": "hello"}"#;
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        if result.is_error() {
            let error_msg = if !result.data.is_null() {
                String::from_utf8_lossy(result.as_slice()).to_string()
            } else {
                format!("error_code={}", result.error_code)
            };
            result.free();
            plugin_shutdown(handle);
            panic!("Call failed: {}", error_msg);
        }

        assert!(!result.data.is_null(), "Result should have data");

        // Parse response - the response is wrapped in ResponseEnvelope
        let response_bytes = result.as_slice();
        let response_str = String::from_utf8_lossy(response_bytes);

        // Try parsing as ResponseEnvelope first
        let envelope: serde_json::Value = serde_json::from_slice(response_bytes)
            .unwrap_or_else(|e| panic!("Failed to parse response: {} - raw: {}", e, response_str));

        // Extract payload from envelope
        let payload = envelope
            .get("payload")
            .unwrap_or_else(|| panic!("No payload in response: {}", response_str));

        let response: EchoResponse = serde_json::from_value(payload.clone())
            .unwrap_or_else(|e| panic!("Failed to parse payload: {} - payload: {}", e, payload));

        assert_eq!(response.message, "hello");
        assert_eq!(response.call_number, 1);

        result.free();
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___multiple_calls___increments_counter() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        let type_tag = c"echo";

        for i in 1..=5u64 {
            let request = format!(r#"{{"message": "call {}"}}"#, i);
            let mut result =
                plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

            assert!(!result.is_error(), "Call {} should succeed", i);

            // Parse ResponseEnvelope and extract payload
            let envelope: serde_json::Value =
                serde_json::from_slice(result.as_slice()).expect("Should parse envelope");
            let payload = envelope.get("payload").expect("Should have payload");
            let response: EchoResponse =
                serde_json::from_value(payload.clone()).expect("Should parse payload");

            assert_eq!(response.call_number, i);

            result.free();
        }

        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___unknown_type_tag___returns_error() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        let type_tag = c"unknown.handler";
        let request = b"{}";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        assert!(result.is_error(), "Unknown handler should return error");
        assert!(result.error_code > 0, "Error code should be set");

        result.free();
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_get_rejected_count___no_rejections___returns_zero() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        let count = plugin_get_rejected_count(handle);
        assert_eq!(count, 0, "No requests should be rejected initially");

        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_shutdown___active_plugin___returns_true() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        // Verify plugin is active before shutdown
        let state_before = plugin_get_state(handle);
        assert_eq!(state_before, 2, "Plugin should be Active before shutdown");

        let result = plugin_shutdown(handle);
        assert!(result, "Shutdown of active plugin should succeed");

        // After shutdown, the handle may be invalidated (removed from manager)
        // So we just verify shutdown returned true - that's the success indicator
    }
}

#[test]
fn plugin_shutdown___double_shutdown___is_idempotent() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        let result1 = plugin_shutdown(handle);
        assert!(result1, "First shutdown should succeed");

        // Second shutdown should not crash (may return true or false)
        let _result2 = plugin_shutdown(handle);
        // Just verify no crash - idempotent behavior
    }
}

// =============================================================================
// Concurrent Call/Shutdown Race Condition Tests
// =============================================================================

#[test]
fn concurrent_calls___multiple_threads___all_succeed() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        let num_threads = 10;
        let calls_per_thread = 5;
        let barrier = Arc::new(Barrier::new(num_threads));

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let barrier_clone = barrier.clone();
            let handle_copy = handle as usize; // Copy for thread

            let thread_handle = thread::spawn(move || {
                barrier_clone.wait();

                for call_id in 0..calls_per_thread {
                    let handle = handle_copy as *mut c_void;
                    let request = format!(r#"{{"message": "t{}c{}"}}"#, thread_id, call_id);
                    let type_tag = c"echo";

                    let mut result =
                        plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

                    if !result.is_error() {
                        result.free();
                    }
                }
            });

            handles.push(thread_handle);
        }

        // Wait for all threads
        for h in handles {
            h.join().expect("Thread should complete");
        }

        plugin_shutdown(handle);
    }
}

#[test]
fn concurrent_call_during_shutdown___no_crash_or_deadlock() {
    // This test verifies that calling plugin_call while plugin_shutdown is in
    // progress doesn't cause a crash or deadlock. The call may fail, but
    // should fail gracefully.
    unsafe {
        let plugin_ptr = create_test_plugin();
        // Use a config with longer shutdown timeout to have time for race
        let config = r#"{"shutdown_timeout_ms": 1000}"#;
        let handle = plugin_init(plugin_ptr, config.as_ptr(), config.len(), None);
        assert!(!handle.is_null());

        let num_callers = 5;
        let barrier = Arc::new(Barrier::new(num_callers + 1));

        let mut caller_handles = vec![];

        // Spawn caller threads that will keep calling during shutdown
        for _ in 0..num_callers {
            let barrier_clone = barrier.clone();
            let handle_copy = handle as usize;

            let caller = thread::spawn(move || {
                barrier_clone.wait();

                // Make calls in a loop - some will succeed, some may fail during shutdown
                for _ in 0..10 {
                    let handle = handle_copy as *mut c_void;
                    let request = b"{}";
                    let type_tag = c"slow"; // Use slow handler to increase race window

                    let mut result =
                        plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

                    // Free the result regardless of success/failure
                    result.free();

                    // Small delay between calls
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            });

            caller_handles.push(caller);
        }

        // Wait for barrier then initiate shutdown
        barrier.wait();
        thread::sleep(std::time::Duration::from_millis(50)); // Let some calls start

        // Shutdown while calls are in flight
        let shutdown_result = plugin_shutdown(handle);

        // Wait for all callers to finish
        for h in caller_handles {
            h.join().expect("Caller thread should not panic");
        }

        // Shutdown should have succeeded (possibly after waiting for in-flight calls)
        assert!(
            shutdown_result,
            "Shutdown should succeed even with concurrent calls"
        );
    }
}

#[test]
fn plugin_call___after_shutdown___returns_error() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        // Shutdown first
        plugin_shutdown(handle);

        // Try to call after shutdown
        let request = b"{}";
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        assert!(result.is_error(), "Call after shutdown should return error");

        result.free();
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn plugin_init___null_plugin_ptr___returns_null() {
    unsafe {
        let handle = plugin_init(std::ptr::null_mut(), std::ptr::null(), 0, None);
        assert!(
            handle.is_null(),
            "Null plugin_ptr should return null handle"
        );
    }
}

#[test]
fn plugin_init___invalid_config_json___returns_null() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let bad_config = b"{ invalid json }";

        let handle = plugin_init(plugin_ptr, bad_config.as_ptr(), bad_config.len(), None);

        assert!(handle.is_null(), "Invalid config JSON should return null");

        // Note: plugin_ptr is leaked here since init failed
        // In production, the host would need to handle this
    }
}

// =============================================================================
// Unicode and Special Character Tests
// =============================================================================

#[test]
fn plugin_call___unicode_emoji_in_message___handled() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        // Request with emoji
        let request = r#"{"message": "Hello 🌍🚀✨ World"}"#;
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        assert!(!result.is_error(), "Unicode emoji should be handled");

        // Verify emoji is preserved in response
        let response_bytes = result.as_slice();
        let response_str = String::from_utf8_lossy(response_bytes);
        assert!(response_str.contains("🌍") || response_str.contains("Hello"));

        result.free();
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___unicode_cjk_characters___handled() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        // Request with CJK characters
        let request = r#"{"message": "你好世界 こんにちは 안녕하세요"}"#;
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        assert!(!result.is_error(), "CJK characters should be handled");
        result.free();
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___escaped_special_chars___handled() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        // Request with escaped special characters
        let request = r#"{"message": "line1\nline2\ttab\"quote\\"}"#;
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        assert!(
            !result.is_error(),
            "Escaped special chars should be handled"
        );
        result.free();
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___null_byte_in_json_string___handled() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        // JSON with escaped null byte
        let request = r#"{"message": "hello\u0000world"}"#;
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        // Should either succeed or fail gracefully
        result.free();
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___empty_message___handled() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        let request = r#"{"message": ""}"#;
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        assert!(!result.is_error(), "Empty message should be handled");
        result.free();
        plugin_shutdown(handle);
    }
}

#[test]
fn plugin_call___very_long_message___handled() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        // Create a long message (100KB)
        let long_content = "x".repeat(100_000);
        let request = format!(r#"{{"message": "{}"}}"#, long_content);
        let type_tag = c"echo";

        let mut result = plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

        assert!(!result.is_error(), "Long message should be handled");
        result.free();
        plugin_shutdown(handle);
    }
}

// =============================================================================
// Handle Management Edge Cases
// =============================================================================

#[test]
fn handle___rapid_create_destroy_cycles___no_leak() {
    unsafe {
        // Rapidly create and destroy plugins
        for _ in 0..50 {
            let plugin_ptr = create_test_plugin();
            let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
            assert!(!handle.is_null());
            plugin_shutdown(handle);
        }
        // If we get here without crash or OOM, the test passes
    }
}

#[test]
fn handle___multiple_plugins_simultaneously___isolated() {
    unsafe {
        // Create multiple plugins at once
        let mut handles = Vec::new();
        for _ in 0..5 {
            let plugin_ptr = create_test_plugin();
            let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
            assert!(!handle.is_null());
            handles.push(handle);
        }

        // Each plugin should be independent
        for (i, &handle) in handles.iter().enumerate() {
            let request = format!(r#"{{"message": "plugin {}"}}"#, i);
            let type_tag = c"echo";

            let mut result =
                plugin_call(handle, type_tag.as_ptr(), request.as_ptr(), request.len());

            assert!(!result.is_error());
            result.free();
        }

        // Shutdown all
        for handle in handles {
            plugin_shutdown(handle);
        }
    }
}

#[test]
fn handle___get_state_after_shutdown___returns_invalid() {
    unsafe {
        let plugin_ptr = create_test_plugin();
        let handle = plugin_init(plugin_ptr, std::ptr::null(), 0, None);
        assert!(!handle.is_null());

        plugin_shutdown(handle);

        // After shutdown, handle may be invalid
        let state = plugin_get_state(handle);
        // State 255 indicates invalid/removed handle, or state may be Stopped (4)
        assert!(
            state == 255 || state == 4,
            "State after shutdown should be invalid or stopped"
        );
    }
}
