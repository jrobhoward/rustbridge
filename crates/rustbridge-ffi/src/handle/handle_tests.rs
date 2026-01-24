#![allow(non_snake_case)]

use super::*;
use async_trait::async_trait;

struct TestPlugin;

#[async_trait]
impl Plugin for TestPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        if type_tag == "echo" {
            Ok(payload.to_vec())
        } else if type_tag == "slow" {
            // Sleep to simulate a slow request
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            Ok(payload.to_vec())
        } else {
            Err(PluginError::UnknownMessageType(type_tag.to_string()))
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }
}

// PluginHandle tests

#[test]
fn PluginHandle___new___starts_in_installed_state() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();

    assert_eq!(handle.state(), LifecycleState::Installed);
}

#[test]
fn PluginHandle___start___transitions_to_active() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();

    handle.start().unwrap();

    assert_eq!(handle.state(), LifecycleState::Active);
}

#[test]
fn PluginHandle___shutdown___transitions_to_stopped() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();
    handle.start().unwrap();

    handle.shutdown(1000).unwrap();

    assert_eq!(handle.state(), LifecycleState::Stopped);
}

#[test]
fn PluginHandle___call___echo_returns_payload() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();
    handle.start().unwrap();

    let response = handle.call("echo", b"hello").unwrap();

    assert_eq!(response, b"hello");

    handle.shutdown(1000).unwrap();
}

#[test]
fn PluginHandle___call___unknown_type_returns_error() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();
    handle.start().unwrap();

    let result = handle.call("unknown", b"test");

    assert!(result.is_err());

    handle.shutdown(1000).unwrap();
}

#[test]
fn PluginHandle___call___before_start_returns_error() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();

    let result = handle.call("echo", b"hello");

    assert!(matches!(result, Err(PluginError::InvalidState { .. })));
}

#[test]
fn PluginHandle___id___initially_none() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();

    assert!(handle.id().is_none());
}

#[test]
fn PluginHandle___set_id___sets_id() {
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();

    handle.set_id(42);

    assert_eq!(handle.id(), Some(42));
}

// PluginHandleManager tests

#[test]
fn PluginHandleManager___new___creates_empty_manager() {
    let manager = PluginHandleManager::new();

    assert!(manager.get(1).is_none());
}

#[test]
fn PluginHandleManager___register___returns_positive_id() {
    let manager = PluginHandleManager::new();
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();

    let id = manager.register(handle);

    assert!(id > 0);
}

#[test]
fn PluginHandleManager___get___retrieves_registered_handle() {
    let manager = PluginHandleManager::new();
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();
    let id = manager.register(handle);

    let retrieved = manager.get(id);

    assert!(retrieved.is_some());
}

#[test]
fn PluginHandleManager___remove___removes_handle() {
    let manager = PluginHandleManager::new();
    let handle = PluginHandle::new(Box::new(TestPlugin), PluginConfig::default()).unwrap();
    let id = manager.register(handle);

    let removed = manager.remove(id);

    assert!(removed.is_some());
    assert!(manager.get(id).is_none());
}

#[test]
fn PluginHandleManager___remove___returns_none_for_unknown_id() {
    let manager = PluginHandleManager::new();

    let removed = manager.remove(999);

    assert!(removed.is_none());
}

#[test]
fn PluginHandleManager___default___same_as_new() {
    let manager = PluginHandleManager::default();

    assert!(manager.get(1).is_none());
}

// Concurrency limiting tests

#[test]
fn concurrency_limit___exceeded___returns_error() {
    let config = PluginConfig {
        max_concurrent_ops: 2,
        ..Default::default()
    };
    let handle = Arc::new(PluginHandle::new(Box::new(TestPlugin), config).unwrap());
    handle.start().unwrap();

    // Spawn 2 concurrent blocking calls
    let h1 = handle.clone();
    let h2 = handle.clone();
    let h3 = handle.clone();

    let t1 = std::thread::spawn(move || h1.call("slow", b"1"));
    let t2 = std::thread::spawn(move || h2.call("slow", b"2"));

    // Give threads time to acquire permits
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Third call should be rejected
    let result3 = h3.call("slow", b"3");

    // Wait for threads
    let result1 = t1.join().unwrap();
    let result2 = t2.join().unwrap();

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(matches!(result3, Err(PluginError::TooManyRequests)));
    assert_eq!(handle.rejected_request_count(), 1);

    handle.shutdown(1000).unwrap();
}

#[test]
fn concurrency_limit___zero___unlimited() {
    let config = PluginConfig {
        max_concurrent_ops: 0,
        ..Default::default()
    };
    let handle = Arc::new(PluginHandle::new(Box::new(TestPlugin), config).unwrap());
    handle.start().unwrap();

    // Spawn many concurrent calls
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let h = handle.clone();
            std::thread::spawn(move || h.call("echo", format!("{}", i).as_bytes()))
        })
        .collect();

    // All should succeed
    for thread_handle in handles {
        let result = thread_handle.join().unwrap();
        assert!(result.is_ok());
    }

    assert_eq!(handle.rejected_request_count(), 0);
    handle.shutdown(1000).unwrap();
}

#[test]
fn concurrency_limit___permit_released___on_error() {
    let config = PluginConfig {
        max_concurrent_ops: 1,
        ..Default::default()
    };
    let handle = PluginHandle::new(Box::new(TestPlugin), config).unwrap();
    handle.start().unwrap();

    // First call that errors
    let result1 = handle.call("unknown", b"test");
    assert!(result1.is_err());

    // Second call should succeed (permit was released)
    let result2 = handle.call("echo", b"test");
    assert!(result2.is_ok());

    assert_eq!(handle.rejected_request_count(), 0);
    handle.shutdown(1000).unwrap();
}

#[test]
fn rejected_request_count___incremented___on_limit_exceeded() {
    let config = PluginConfig {
        max_concurrent_ops: 1,
        ..Default::default()
    };
    let handle = Arc::new(PluginHandle::new(Box::new(TestPlugin), config).unwrap());
    handle.start().unwrap();

    // Start a blocking call in another thread
    let h1 = handle.clone();
    let t1 = std::thread::spawn(move || h1.call("slow", b"1"));

    // Give thread time to acquire permit
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Try multiple concurrent calls (should be rejected)
    for _ in 0..5 {
        let result = handle.call("echo", b"test");
        assert!(matches!(result, Err(PluginError::TooManyRequests)));
    }

    // Wait for the blocking call to complete
    t1.join().unwrap().unwrap();

    assert_eq!(handle.rejected_request_count(), 5);
    handle.shutdown(1000).unwrap();
}
