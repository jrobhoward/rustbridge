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
