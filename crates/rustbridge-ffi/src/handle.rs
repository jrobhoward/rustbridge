//! Plugin handle management

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rustbridge_core::{
    LifecycleState, Plugin, PluginConfig, PluginContext, PluginError, PluginResult,
};
use rustbridge_logging::LogCallbackManager;
use rustbridge_runtime::{AsyncBridge, AsyncRuntime, RuntimeConfig};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global handle manager
static HANDLE_MANAGER: OnceCell<PluginHandleManager> = OnceCell::new();

/// Manages plugin handles
pub struct PluginHandleManager {
    handles: DashMap<u64, Arc<PluginHandle>>,
    next_id: AtomicU64,
}

impl PluginHandleManager {
    /// Create a new handle manager
    pub fn new() -> Self {
        Self {
            handles: DashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// Get the global handle manager
    pub fn global() -> &'static PluginHandleManager {
        HANDLE_MANAGER.get_or_init(PluginHandleManager::new)
    }

    /// Register a new handle
    pub fn register(&self, handle: PluginHandle) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.insert(id, Arc::new(handle));
        id
    }

    /// Get a handle by ID
    pub fn get(&self, id: u64) -> Option<Arc<PluginHandle>> {
        self.handles.get(&id).map(|r| r.clone())
    }

    /// Remove a handle
    pub fn remove(&self, id: u64) -> Option<Arc<PluginHandle>> {
        self.handles.remove(&id).map(|(_, v)| v)
    }
}

impl Default for PluginHandleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to an initialized plugin instance
pub struct PluginHandle {
    /// The plugin implementation
    plugin: Box<dyn Plugin>,
    /// Plugin context (state, config)
    context: PluginContext,
    /// Async runtime
    runtime: Arc<AsyncRuntime>,
    /// Async bridge for FFI calls
    bridge: AsyncBridge,
    /// Handle ID (set after registration)
    id: RwLock<Option<u64>>,
}

impl PluginHandle {
    /// Create a new plugin handle
    pub fn new(plugin: Box<dyn Plugin>, config: PluginConfig) -> PluginResult<Self> {
        // Create runtime configuration from plugin config
        let runtime_config = RuntimeConfig {
            worker_threads: config.worker_threads,
            ..Default::default()
        };

        // Create the async runtime
        let runtime = Arc::new(AsyncRuntime::new(runtime_config)?);

        // Create the async bridge
        let bridge = AsyncBridge::new(runtime.clone());

        // Create plugin context
        let context = PluginContext::new(config);

        Ok(Self {
            plugin,
            context,
            runtime,
            bridge,
            id: RwLock::new(None),
        })
    }

    /// Get the handle ID
    pub fn id(&self) -> Option<u64> {
        *self.id.read()
    }

    /// Set the handle ID (called after registration)
    pub(crate) fn set_id(&self, id: u64) {
        *self.id.write() = Some(id);
    }

    /// Get the current lifecycle state
    pub fn state(&self) -> LifecycleState {
        self.context.state()
    }

    /// Start the plugin
    pub fn start(&self) -> PluginResult<()> {
        // Transition to Starting state
        self.context.transition_to(LifecycleState::Starting)?;

        // Call plugin's on_start
        let result = self.bridge.call_sync(self.plugin.on_start(&self.context));

        match result {
            Ok(()) => {
                // Transition to Active
                self.context.transition_to(LifecycleState::Active)?;
                tracing::info!("Plugin started successfully");
                Ok(())
            }
            Err(e) => {
                // Transition to Failed
                self.context.set_state(LifecycleState::Failed);
                tracing::error!("Plugin failed to start: {}", e);
                Err(e)
            }
        }
    }

    /// Handle a request
    pub fn call(&self, type_tag: &str, request: &[u8]) -> PluginResult<Vec<u8>> {
        // Check state
        if !self.context.state().can_handle_requests() {
            return Err(PluginError::InvalidState {
                expected: "Active".to_string(),
                actual: self.context.state().to_string(),
            });
        }

        // Call the plugin handler
        self.bridge
            .call_sync(self.plugin.handle_request(&self.context, type_tag, request))
    }

    /// Shutdown the plugin
    pub fn shutdown(&self, timeout_ms: u64) -> PluginResult<()> {
        let current_state = self.context.state();

        // Can only shutdown from Active state
        if current_state != LifecycleState::Active {
            if current_state.is_terminal() {
                return Ok(()); // Already stopped/failed
            }
            return Err(PluginError::InvalidState {
                expected: "Active".to_string(),
                actual: current_state.to_string(),
            });
        }

        // Transition to Stopping
        self.context.transition_to(LifecycleState::Stopping)?;

        // Call plugin's on_stop with timeout
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let result = self
            .bridge
            .call_sync_timeout(self.plugin.on_stop(&self.context), timeout);

        // Shutdown runtime
        let runtime_timeout = std::time::Duration::from_millis(timeout_ms / 2);
        let _ = self.runtime.shutdown(runtime_timeout);

        match result {
            Ok(()) => {
                self.context.transition_to(LifecycleState::Stopped)?;
                tracing::info!("Plugin shutdown complete");
                Ok(())
            }
            Err(PluginError::Timeout) => {
                self.context.set_state(LifecycleState::Stopped);
                tracing::warn!("Plugin shutdown timed out");
                Ok(()) // Consider timeout as successful shutdown
            }
            Err(e) => {
                self.context.set_state(LifecycleState::Failed);
                tracing::error!("Plugin shutdown failed: {}", e);
                Err(e)
            }
        }
    }

    /// Set the log level
    pub fn set_log_level(&self, level: rustbridge_core::LogLevel) {
        // Update the log callback manager's level
        LogCallbackManager::global().set_level(level);

        // Reload the tracing subscriber's filter
        if let Err(e) = rustbridge_logging::ReloadHandle::global().reload_level(level) {
            tracing::warn!("Failed to reload tracing filter: {}", e);
        }
    }

    /// Mark the plugin as failed
    ///
    /// This is called when a panic is caught at the FFI boundary or when
    /// an unrecoverable error occurs. It immediately transitions the plugin
    /// to the Failed state.
    ///
    /// After calling this method, the plugin will reject all further requests.
    /// The host should call plugin_shutdown to clean up resources.
    pub fn mark_failed(&self) {
        tracing::error!("Marking plugin as failed due to panic or unrecoverable error");
        self.context.set_state(LifecycleState::Failed);
    }
}

// Plugin handles are thread-safe
unsafe impl Send for PluginHandle {}
unsafe impl Sync for PluginHandle {}

#[cfg(test)]
#[path = "handle/handle_tests.rs"]
mod handle_tests;
