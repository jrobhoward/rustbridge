#![allow(non_snake_case)]

use super::*;
use crate::RuntimeConfig;

fn create_test_bridge() -> AsyncBridge {
    let runtime = AsyncRuntime::new(RuntimeConfig::default()).unwrap();
    AsyncBridge::new(Arc::new(runtime))
}

// AsyncBridge tests

#[test]
fn AsyncBridge___next_request_id___increments_sequentially() {
    let bridge = create_test_bridge();

    let id1 = bridge.next_request_id();
    let id2 = bridge.next_request_id();
    let id3 = bridge.next_request_id();

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 2);
}

#[test]
fn AsyncBridge___call_sync___executes_async_future() {
    let bridge = create_test_bridge();

    let result = bridge.call_sync(async { Ok::<_, PluginError>(42) });

    assert_eq!(result.unwrap(), 42);
}

#[test]
fn AsyncBridge___call_sync___propagates_errors() {
    let bridge = create_test_bridge();

    let result: PluginResult<()> = bridge.call_sync(async {
        Err(PluginError::HandlerError("test error".to_string()))
    });

    assert!(result.is_err());
}

#[test]
fn AsyncBridge___call_sync_timeout___succeeds_within_timeout() {
    let bridge = create_test_bridge();

    let result = bridge.call_sync_timeout(
        async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            Ok::<_, PluginError>(42)
        },
        std::time::Duration::from_secs(1),
    );

    assert_eq!(result.unwrap(), 42);
}

#[test]
fn AsyncBridge___call_sync_timeout___returns_timeout_error() {
    let bridge = create_test_bridge();

    let result: PluginResult<()> = bridge.call_sync_timeout(
        async {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            Ok(())
        },
        std::time::Duration::from_millis(10),
    );

    assert!(matches!(result, Err(PluginError::Timeout)));
}

#[test]
fn AsyncBridge___spawn___creates_task() {
    let bridge = create_test_bridge();

    let handle = bridge.spawn(async { 123 });
    let result = bridge.call_sync(async {
        handle
            .await
            .map_err(|e| PluginError::RuntimeError(e.to_string()))
    });

    assert_eq!(result.unwrap(), 123);
}

#[test]
fn AsyncBridge___shutdown_signal___returns_valid_signal() {
    let bridge = create_test_bridge();

    let signal = bridge.shutdown_signal();

    assert!(!signal.is_triggered());
}

#[test]
fn AsyncBridge___is_shutting_down___initially_false() {
    let bridge = create_test_bridge();

    assert!(!bridge.is_shutting_down());
}

// PendingRequest tests

#[test]
fn PendingRequest___new___creates_request() {
    extern "C" fn dummy_callback(
        _context: *mut std::ffi::c_void,
        _request_id: u64,
        _data: *const u8,
        _len: usize,
        _error_code: u32,
    ) {
    }

    let request = PendingRequest::new(42, dummy_callback, std::ptr::null_mut());

    assert_eq!(request.request_id, 42);
    assert!(request.cancel_handle.is_none());
}
