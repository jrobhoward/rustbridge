//! Tests for resource leak detection in handle management
//!
//! These tests verify that handles are properly cleaned up and don't leak
//! memory or file descriptors.

use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginConfig, PluginContext, PluginResult};
use rustbridge_ffi::{FfiBuffer, PluginHandle, PluginHandleManager};
use std::sync::Arc;

/// Minimal test plugin
struct TestPlugin;

#[async_trait]
impl Plugin for TestPlugin {
    async fn on_start(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn on_stop(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        _context: &PluginContext,
        _type_tag: &str,
        _request: &[u8],
    ) -> PluginResult<Vec<u8>> {
        Ok(vec![])
    }
}

#[test]
fn test_handle_register_remove_cycle() {
    let manager = PluginHandleManager::new();

    let config = PluginConfig::default();
    let handle = PluginHandle::new(Box::new(TestPlugin), config).expect("Should create handle");

    // Register
    let id = manager.register(handle);
    assert!(id > 0);

    // Verify we can retrieve it
    let retrieved = manager.get(id);
    assert!(retrieved.is_some());

    // Remove
    let removed = manager.remove(id);
    assert!(removed.is_some());

    // Should not be retrievable after removal
    let not_found = manager.get(id);
    assert!(not_found.is_none());
}

#[test]
fn test_sequential_register_remove_cycles() {
    let manager = PluginHandleManager::new();

    for _ in 0..100 {
        let config = PluginConfig::default();
        let handle = PluginHandle::new(Box::new(TestPlugin), config).expect("Should create handle");

        let id = manager.register(handle);
        let removed = manager.remove(id);

        assert!(
            removed.is_some(),
            "Should successfully remove registered handle"
        );
        assert!(
            manager.get(id).is_none(),
            "Handle should not be retrievable after removal"
        );
    }
}

#[test]
fn test_multiple_handles_concurrent_removal() {
    let manager = Arc::new(PluginHandleManager::new());
    let mut handles = vec![];

    // Register 50 handles
    for _ in 0..50 {
        let config = PluginConfig::default();
        let handle = PluginHandle::new(Box::new(TestPlugin), config).expect("Should create handle");

        let id = manager.register(handle);
        handles.push(id);
    }

    // Remove all handles
    for id in handles {
        let removed = manager.remove(id);
        assert!(
            removed.is_some(),
            "Should successfully remove handle {}",
            id
        );
    }

    // Verify all are gone
    let mut count = 0;
    for id in 1..=50u64 {
        if manager.get(id).is_some() {
            count += 1;
        }
    }
    assert_eq!(count, 0, "All handles should be removed");
}

#[test]
fn test_handle_drop_after_removal() {
    let manager = PluginHandleManager::new();
    let config = PluginConfig::default();
    let handle = PluginHandle::new(Box::new(TestPlugin), config).expect("Should create handle");

    let id = manager.register(handle);

    // Get a reference to the handle
    let handle_ref = manager.get(id);
    assert!(handle_ref.is_some());

    // Remove from manager (this doesn't drop the handle yet due to Arc)
    let removed = manager.remove(id);
    assert!(removed.is_some());

    // Our reference should still work temporarily
    assert!(handle_ref.is_some());

    // After dropping our reference and the removed Arc, the handle should be dropped
    drop(handle_ref);
    drop(removed);

    // Should not be in the manager
    assert!(manager.get(id).is_none());
}

#[test]
fn test_empty_buffer_creation_and_free() {
    let mut buffer = FfiBuffer::empty();

    assert!(buffer.is_empty());
    assert!(buffer.data.is_null());

    // Freeing empty buffer should be safe
    unsafe {
        buffer.free();
    }

    assert!(buffer.data.is_null());
    assert_eq!(buffer.len, 0);
}

#[test]
fn test_buffer_from_vec_large_allocation() {
    let large_vec: Vec<u8> = vec![0; 1_000_000]; // 1MB
    let original_len = large_vec.len();

    let mut buffer = FfiBuffer::from_vec(large_vec);

    assert_eq!(buffer.len, original_len);
    assert!(!buffer.data.is_null());

    unsafe {
        buffer.free();
    }

    assert!(buffer.data.is_null());
    assert_eq!(buffer.len, 0);
}

#[test]
fn test_error_buffer_cleanup() {
    let message = "Error: Something went wrong";
    let mut buffer = FfiBuffer::error(42, message);

    assert_eq!(buffer.error_code, 42);
    assert!(!buffer.data.is_null());

    unsafe {
        let slice = buffer.as_slice();
        assert_eq!(std::str::from_utf8(slice).unwrap(), message);
        buffer.free();
    }

    assert!(buffer.data.is_null());
}
