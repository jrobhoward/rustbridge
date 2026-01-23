//! Plugin trait and context types

use crate::{LifecycleState, PluginConfig, PluginError, PluginResult};
use async_trait::async_trait;

/// Context provided to plugin operations
pub struct PluginContext {
    /// Plugin configuration
    pub config: PluginConfig,
    /// Current lifecycle state
    state: std::sync::atomic::AtomicU8,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            state: std::sync::atomic::AtomicU8::new(LifecycleState::Installed as u8),
        }
    }

    /// Get current lifecycle state
    pub fn state(&self) -> LifecycleState {
        let value = self.state.load(std::sync::atomic::Ordering::SeqCst);
        match value {
            0 => LifecycleState::Installed,
            1 => LifecycleState::Starting,
            2 => LifecycleState::Active,
            3 => LifecycleState::Stopping,
            4 => LifecycleState::Stopped,
            _ => LifecycleState::Failed,
        }
    }

    /// Set lifecycle state directly (bypassing transition validation)
    ///
    /// Use this for error recovery scenarios where normal transitions don't apply.
    pub fn set_state(&self, state: LifecycleState) {
        let value = match state {
            LifecycleState::Installed => 0,
            LifecycleState::Starting => 1,
            LifecycleState::Active => 2,
            LifecycleState::Stopping => 3,
            LifecycleState::Stopped => 4,
            LifecycleState::Failed => 5,
        };
        self.state.store(value, std::sync::atomic::Ordering::SeqCst);
    }

    /// Attempt to transition to a new state
    pub fn transition_to(&self, target: LifecycleState) -> PluginResult<()> {
        let current = self.state();
        if current.can_transition_to(target) {
            self.set_state(target);
            Ok(())
        } else {
            Err(PluginError::InvalidState {
                expected: format!("state that can transition to {}", target),
                actual: current.to_string(),
            })
        }
    }
}

/// Main trait for implementing rustbridge plugins
///
/// Plugins must implement this trait to define their behavior.
/// The async methods are executed on the Tokio runtime.
///
/// # Example
///
/// ```ignore
/// use rustbridge_core::prelude::*;
///
/// struct MyPlugin;
///
/// #[async_trait::async_trait]
/// impl Plugin for MyPlugin {
///     async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
///         // Initialize resources
///         Ok(())
///     }
///
///     async fn handle_request(
///         &self,
///         ctx: &PluginContext,
///         type_tag: &str,
///         payload: &[u8],
///     ) -> PluginResult<Vec<u8>> {
///         match type_tag {
///             "echo" => Ok(payload.to_vec()),
///             _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
///         }
///     }
///
///     async fn on_stop(&self, ctx: &PluginContext) -> PluginResult<()> {
///         // Cleanup resources
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Called when the plugin is starting up
    ///
    /// Use this to initialize resources, connections, etc.
    /// The plugin transitions to Active state after this returns successfully.
    async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()>;

    /// Handle an incoming request
    ///
    /// - `type_tag`: Message type identifier (e.g., "user.create")
    /// - `payload`: JSON-encoded request payload
    ///
    /// Returns JSON-encoded response payload
    async fn handle_request(
        &self,
        ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>>;

    /// Called when the plugin is shutting down
    ///
    /// Use this to cleanup resources, close connections, etc.
    /// The plugin transitions to Stopped state after this returns.
    async fn on_stop(&self, ctx: &PluginContext) -> PluginResult<()>;

    /// Get plugin metadata
    ///
    /// Override this to provide plugin information
    fn metadata(&self) -> Option<crate::PluginMetadata> {
        None
    }

    /// List supported message types
    ///
    /// Override this to provide a list of supported type tags
    fn supported_types(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

#[cfg(test)]
#[path = "plugin/plugin_tests.rs"]
mod plugin_tests;
