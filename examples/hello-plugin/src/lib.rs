//! hello-plugin - Example rustbridge plugin
//!
//! This plugin demonstrates the basic structure of a rustbridge plugin,
//! including message handling, lifecycle management, and FFI integration.

use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginContext, PluginError, PluginMetadata, PluginResult};
use rustbridge_macros::{rustbridge_entry, Message};
use serde::{Deserialize, Serialize};

// ============================================================================
// Message Types
// ============================================================================

/// Request to echo a message back
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {
    pub message: String,
}

/// Response from echo request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
    pub length: usize,
}

/// Request to greet a user
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "greet")]
pub struct GreetRequest {
    pub name: String,
}

/// Response from greet request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetResponse {
    pub greeting: String,
}

/// Request to create a user
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "user.create")]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

/// Response from user creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub user_id: String,
    pub created_at: String,
}

/// Request to add numbers
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "math.add")]
pub struct AddRequest {
    pub a: i64,
    pub b: i64,
}

/// Response from add operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddResponse {
    pub result: i64,
}

// ============================================================================
// Plugin Implementation
// ============================================================================

/// Hello Plugin - demonstrates rustbridge functionality
#[derive(Default)]
pub struct HelloPlugin {
    /// Counter for generated user IDs
    user_counter: std::sync::atomic::AtomicU64,
}

impl HelloPlugin {
    /// Create a new HelloPlugin instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle echo request
    fn handle_echo(&self, req: EchoRequest) -> PluginResult<EchoResponse> {
        tracing::debug!("Handling echo request: {:?}", req);
        Ok(EchoResponse {
            length: req.message.len(),
            message: req.message,
        })
    }

    /// Handle greet request
    fn handle_greet(&self, req: GreetRequest) -> PluginResult<GreetResponse> {
        tracing::debug!("Handling greet request: {:?}", req);
        Ok(GreetResponse {
            greeting: format!("Hello, {}! Welcome to rustbridge.", req.name),
        })
    }

    /// Handle user creation
    fn handle_create_user(&self, req: CreateUserRequest) -> PluginResult<CreateUserResponse> {
        tracing::info!("Creating user: {} ({})", req.username, req.email);

        // Generate a simple user ID
        let id = self
            .user_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Ok(CreateUserResponse {
            user_id: format!("user-{:08x}", id),
            created_at: chrono_lite_now(),
        })
    }

    /// Handle math addition
    fn handle_add(&self, req: AddRequest) -> PluginResult<AddResponse> {
        tracing::debug!("Adding {} + {}", req.a, req.b);
        Ok(AddResponse {
            result: req.a + req.b,
        })
    }
}

#[async_trait]
impl Plugin for HelloPlugin {
    async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("HelloPlugin starting...");
        tracing::info!("Log level: {}", ctx.config.log_level);
        tracing::info!("HelloPlugin started successfully");
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
                let resp = self.handle_echo(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "greet" => {
                let req: GreetRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_greet(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "user.create" => {
                let req: CreateUserRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_create_user(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "math.add" => {
                let req: AddRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_add(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("HelloPlugin stopping...");
        tracing::info!("HelloPlugin stopped");
        Ok(())
    }

    fn metadata(&self) -> Option<PluginMetadata> {
        Some(PluginMetadata::new("hello-plugin", "0.1.0"))
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["echo", "greet", "user.create", "math.add"]
    }
}

// Generate the FFI entry point
rustbridge_entry!(HelloPlugin::new);

// Re-export FFI functions from rustbridge-ffi
pub use rustbridge_ffi::{
    plugin_call, plugin_call_async, plugin_cancel_async, plugin_free_buffer, plugin_get_state,
    plugin_init, plugin_set_log_level, plugin_shutdown,
};

// ============================================================================
// Utilities
// ============================================================================

/// Simple timestamp function (avoiding chrono dependency for the example)
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("{}Z", duration.as_secs())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
