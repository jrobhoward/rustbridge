#![allow(non_snake_case)]

use super::*;

// Test helper plugin
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

// PluginContext tests

#[test]
fn PluginContext___new___starts_in_installed_state() {
    let ctx = PluginContext::new(PluginConfig::default());

    assert_eq!(ctx.state(), LifecycleState::Installed);
}

#[test]
fn PluginContext___transition_to___valid_transition_succeeds() {
    let ctx = PluginContext::new(PluginConfig::default());

    ctx.transition_to(LifecycleState::Starting).unwrap();

    assert_eq!(ctx.state(), LifecycleState::Starting);
}

#[test]
fn PluginContext___transition_to___chain_through_lifecycle() {
    let ctx = PluginContext::new(PluginConfig::default());

    ctx.transition_to(LifecycleState::Starting).unwrap();
    ctx.transition_to(LifecycleState::Active).unwrap();

    assert_eq!(ctx.state(), LifecycleState::Active);
}

#[test]
fn PluginContext___transition_to___invalid_transition_fails() {
    let ctx = PluginContext::new(PluginConfig::default());

    let result = ctx.transition_to(LifecycleState::Active);

    assert!(result.is_err());
}

#[test]
fn PluginContext___set_state___bypasses_validation() {
    let ctx = PluginContext::new(PluginConfig::default());

    ctx.set_state(LifecycleState::Failed);

    assert_eq!(ctx.state(), LifecycleState::Failed);
}

// Plugin trait tests

#[tokio::test]
async fn Plugin___handle_request___echo_returns_payload() {
    let plugin = TestPlugin;
    let ctx = PluginContext::new(PluginConfig::default());

    let response = plugin.handle_request(&ctx, "echo", b"hello").await.unwrap();

    assert_eq!(response, b"hello");
}

#[tokio::test]
async fn Plugin___handle_request___unknown_type_returns_error() {
    let plugin = TestPlugin;
    let ctx = PluginContext::new(PluginConfig::default());

    let result = plugin.handle_request(&ctx, "unknown", b"test").await;

    assert!(matches!(result, Err(PluginError::UnknownMessageType(_))));
}

#[tokio::test]
async fn Plugin___on_start___succeeds_for_test_plugin() {
    let plugin = TestPlugin;
    let ctx = PluginContext::new(PluginConfig::default());

    let result = plugin.on_start(&ctx).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn Plugin___on_stop___succeeds_for_test_plugin() {
    let plugin = TestPlugin;
    let ctx = PluginContext::new(PluginConfig::default());

    let result = plugin.on_stop(&ctx).await;

    assert!(result.is_ok());
}

#[test]
fn Plugin___metadata___returns_none_by_default() {
    let plugin = TestPlugin;

    let metadata = plugin.metadata();

    assert!(metadata.is_none());
}

#[test]
fn Plugin___supported_types___returns_empty_by_default() {
    let plugin = TestPlugin;

    let types = plugin.supported_types();

    assert!(types.is_empty());
}
