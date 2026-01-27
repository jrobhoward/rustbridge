//! my-plugin - A rustbridge plugin template
//!
//! This template implements a simple "echo" message type.
//! Modify it to add your own message types and business logic.

use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginConfig, PluginContext, PluginError, PluginResult};
use rustbridge_macros::{rustbridge_entry, Message};
use serde::{Deserialize, Serialize};

// ============================================================================
// Message Types
// ============================================================================

/// Echo request message
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {
    pub message: String,
}

/// Echo response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
    pub length: usize,
}

// ============================================================================
// Plugin Implementation
// ============================================================================

/// Plugin implementation
#[derive(Default)]
pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("my-plugin started");
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "echo" => {
                let req: EchoRequest = serde_json::from_slice(payload)?;
                let response = EchoResponse {
                    length: req.message.len(),
                    message: req.message,
                };
                Ok(serde_json::to_vec(&response)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("my-plugin stopped");
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["echo"]
    }
}

// ============================================================================
// FFI Entry Point
// ============================================================================

// Generate FFI entry point
rustbridge_entry!(MyPlugin::default);

// Re-export FFI functions for the compiled library
pub use rustbridge_ffi::{
    plugin_call,
    plugin_free_buffer,
    plugin_get_rejected_count,
    plugin_get_state,
    plugin_init,
    plugin_set_log_level,
    plugin_shutdown,
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo() {
        let plugin = MyPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&EchoRequest {
            message: "Hello, World!".to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "echo", &request).await.unwrap();
        let echo_response: EchoResponse = serde_json::from_slice(&response).unwrap();

        assert_eq!(echo_response.message, "Hello, World!");
        assert_eq!(echo_response.length, 13);
    }
}
