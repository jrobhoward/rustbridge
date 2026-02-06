//! variant-log-plugin - Example plugin that identifies its build variant.
//!
//! This plugin returns different responses and log messages depending on its
//! build profile (debug/release) and feature flags (extended-info). It is used
//! by integration tests to verify bundle variant loading.

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};

// ============================================================================
// Variant Detection
// ============================================================================

/// Return a label describing the build variant of this plugin.
fn variant_label() -> &'static str {
    match (cfg!(debug_assertions), cfg!(feature = "extended-info")) {
        (true, false) => "debug",
        (false, false) => "release",
        (true, true) => "debug+extended-info",
        (false, true) => "release+extended-info",
    }
}

// ============================================================================
// Message Types
// ============================================================================

/// Request to identify the build variant
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "identify")]
pub struct IdentifyRequest {}

/// Response containing the build variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyResponse {
    pub variant: String,
}

// ============================================================================
// Plugin Implementation
// ============================================================================

/// Plugin that reports its build variant via responses and log messages.
#[derive(Default)]
pub struct VariantLogPlugin;

impl VariantLogPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for VariantLogPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("VariantLogPlugin starting [{}]", variant_label());
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "identify" => {
                let _req: IdentifyRequest = serde_json::from_slice(payload)?;
                tracing::info!("Handling identify request [{}]", variant_label());
                let resp = IdentifyResponse {
                    variant: variant_label().to_string(),
                };
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("VariantLogPlugin stopping [{}]", variant_label());
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["identify"]
    }
}

// Generate the FFI entry point
rustbridge_entry!(VariantLogPlugin::new);

// Re-export FFI functions for the shared library
pub use rustbridge::ffi_exports::*;
