//! hello-plugin - Example rustbridge plugin
//!
//! This plugin demonstrates the basic structure of a rustbridge plugin,
//! including message handling, lifecycle management, and FFI integration.

use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginContext, PluginError, PluginMetadata, PluginResult};
use rustbridge_macros::{Message, rustbridge_entry};
use serde::{Deserialize, Serialize};

pub mod binary_messages;

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

/// Request to sleep for testing concurrency limits
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "test.sleep")]
pub struct SleepRequest {
    /// Duration to sleep in milliseconds
    pub duration_ms: u64,
}

/// Response from sleep request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepResponse {
    /// Actual duration slept in milliseconds
    pub slept_ms: u64,
}

// ============================================================================
// Benchmark Message Types
// ============================================================================

/// Small benchmark request (~100 bytes JSON)
/// Simulates: config lookup, feature flag check
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "bench.small")]
pub struct SmallRequest {
    pub key: String,
    pub flags: u32,
}

/// Small benchmark response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallResponse {
    pub value: String,
    pub ttl_seconds: u32,
    pub cache_hit: bool,
}

/// Medium benchmark request (~1KB JSON)
/// Simulates: user record lookup with field selection
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "bench.medium")]
pub struct MediumRequest {
    pub user_id: u64,
    pub include_fields: Vec<String>,
    pub options: MediumOptions,
}

/// Options for medium request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumOptions {
    pub include_metadata: bool,
    pub include_permissions: bool,
    pub max_results: u32,
}

/// Medium benchmark response (~1KB JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumResponse {
    pub user_id: u64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub metadata: Vec<KeyValue>,
    pub permissions: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Key-value pair for metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

/// Large benchmark request (~1KB JSON with filters)
/// Simulates: batch query, data export
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "bench.large")]
pub struct LargeRequest {
    pub query_id: u64,
    pub filters: Vec<Filter>,
    pub page_size: u32,
    pub page_token: Option<String>,
}

/// Filter for large queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: String,
    pub value: String,
}

/// Large benchmark response (~100KB JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeResponse {
    pub query_id: u64,
    pub results: Vec<Record>,
    pub total_count: u64,
    pub next_page_token: Option<String>,
}

/// Record in large response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub score: f64,
    pub metadata: Vec<KeyValue>,
    pub created_at: i64,
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

    /// Handle sleep request - useful for testing concurrency limits
    async fn handle_sleep(&self, req: SleepRequest) -> PluginResult<SleepResponse> {
        tracing::debug!("Sleeping for {}ms", req.duration_ms);
        tokio::time::sleep(tokio::time::Duration::from_millis(req.duration_ms)).await;
        Ok(SleepResponse {
            slept_ms: req.duration_ms,
        })
    }

    // ========================================================================
    // Benchmark Handlers
    // ========================================================================

    /// Handle small benchmark request
    fn handle_bench_small(&self, req: SmallRequest) -> PluginResult<SmallResponse> {
        // Simulate a simple lookup - response ~100 bytes
        Ok(SmallResponse {
            value: format!("value_for_{}", req.key),
            ttl_seconds: 3600,
            cache_hit: req.flags & 1 != 0,
        })
    }

    /// Handle medium benchmark request
    fn handle_bench_medium(&self, req: MediumRequest) -> PluginResult<MediumResponse> {
        // Simulate user lookup - response ~1KB
        let metadata = if req.options.include_metadata {
            (0..10)
                .map(|i| KeyValue {
                    key: format!("meta_key_{}", i),
                    value: format!("meta_value_{}_for_user_{}", i, req.user_id),
                })
                .collect()
        } else {
            Vec::new()
        };

        let permissions = if req.options.include_permissions {
            vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "admin".to_string(),
                "users.read".to_string(),
                "users.write".to_string(),
                "reports.read".to_string(),
                "reports.write".to_string(),
                "settings.read".to_string(),
                "settings.write".to_string(),
                "audit.read".to_string(),
                "billing.read".to_string(),
                "billing.write".to_string(),
                "api.access".to_string(),
                "webhooks.manage".to_string(),
            ]
        } else {
            Vec::new()
        };

        Ok(MediumResponse {
            user_id: req.user_id,
            username: format!("user_{}", req.user_id),
            email: format!("user_{}@example.com", req.user_id),
            display_name: format!("User Number {}", req.user_id),
            metadata,
            permissions,
            created_at: 1700000000,
            updated_at: 1705000000,
        })
    }

    /// Handle large benchmark request
    fn handle_bench_large(&self, req: LargeRequest) -> PluginResult<LargeResponse> {
        // Simulate batch query - response ~100KB (1000 records)
        let record_count = req.page_size.min(1000) as usize;

        let results: Vec<Record> = (0..record_count)
            .map(|i| {
                let id = req.query_id * 10000 + i as u64;
                Record {
                    id,
                    name: format!("Record {} for query {}", i, req.query_id),
                    description: format!(
                        "This is a detailed description for record {}. \
                         It contains enough text to make the payload realistic. \
                         Query ID: {}, Filters: {}",
                        i,
                        req.query_id,
                        req.filters.len()
                    ),
                    category: format!("category_{}", i % 10),
                    tags: vec![
                        format!("tag_a_{}", i % 5),
                        format!("tag_b_{}", i % 7),
                        format!("tag_c_{}", i % 3),
                    ],
                    score: (i as f64) * 0.1 + 0.5,
                    metadata: vec![
                        KeyValue {
                            key: "source".to_string(),
                            value: "benchmark".to_string(),
                        },
                        KeyValue {
                            key: "version".to_string(),
                            value: "1.0".to_string(),
                        },
                    ],
                    created_at: 1700000000 + (i as i64 * 1000),
                }
            })
            .collect();

        let total_count = record_count as u64 * 10; // Simulate more results available

        Ok(LargeResponse {
            query_id: req.query_id,
            results,
            total_count,
            next_page_token: Some(format!("token_{}_{}", req.query_id, record_count)),
        })
    }
}

#[async_trait]
impl Plugin for HelloPlugin {
    async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("HelloPlugin starting...");
        tracing::info!("Log level: {}", ctx.config.log_level);

        // Register binary message handlers for benchmarking
        binary_messages::register_benchmark_handlers();

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
            "test.sleep" => {
                let req: SleepRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_sleep(req).await?;
                Ok(serde_json::to_vec(&resp)?)
            }
            // Benchmark handlers
            "bench.small" => {
                let req: SmallRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_bench_small(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "bench.medium" => {
                let req: MediumRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_bench_medium(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "bench.large" => {
                let req: LargeRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_bench_large(req)?;
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
        Some(PluginMetadata::new("hello-plugin", "0.5.0"))
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec![
            "echo",
            "greet",
            "user.create",
            "math.add",
            "test.sleep",
            "bench.small",
            "bench.medium",
            "bench.large",
        ]
    }
}

// Generate the FFI entry point
rustbridge_entry!(HelloPlugin::new);

// Re-export FFI functions from rustbridge-ffi
pub use rustbridge_ffi::{
    plugin_call, plugin_call_async, plugin_cancel_async, plugin_free_buffer,
    plugin_get_rejected_count, plugin_get_state, plugin_init, plugin_set_log_level,
    plugin_shutdown,
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
