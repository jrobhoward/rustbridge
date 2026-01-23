//! Tokio runtime management

use crate::shutdown::{ShutdownHandle, ShutdownSignal};
use parking_lot::Mutex;
use rustbridge_core::{PluginError, PluginResult};
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};

/// Configuration for the async runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Number of worker threads (None = number of CPU cores)
    pub worker_threads: Option<usize>,
    /// Name prefix for worker threads
    pub thread_name: String,
    /// Enable I/O driver
    pub enable_io: bool,
    /// Enable time driver
    pub enable_time: bool,
    /// Maximum blocking threads
    pub max_blocking_threads: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None,
            thread_name: "rustbridge-worker".to_string(),
            enable_io: true,
            enable_time: true,
            max_blocking_threads: 512,
        }
    }
}

impl RuntimeConfig {
    /// Create a new runtime configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of worker threads
    pub fn with_worker_threads(mut self, threads: usize) -> Self {
        self.worker_threads = Some(threads);
        self
    }

    /// Set the thread name prefix
    pub fn with_thread_name(mut self, name: impl Into<String>) -> Self {
        self.thread_name = name.into();
        self
    }
}

/// Manages the Tokio async runtime for a plugin
pub struct AsyncRuntime {
    runtime: Arc<Runtime>,
    shutdown_handle: ShutdownHandle,
    config: RuntimeConfig,
}

impl AsyncRuntime {
    /// Create a new async runtime with the given configuration
    pub fn new(config: RuntimeConfig) -> PluginResult<Self> {
        let mut builder = Builder::new_multi_thread();

        if let Some(threads) = config.worker_threads {
            builder.worker_threads(threads);
        }

        builder
            .thread_name(&config.thread_name)
            .max_blocking_threads(config.max_blocking_threads);

        if config.enable_io {
            builder.enable_io();
        }

        if config.enable_time {
            builder.enable_time();
        }

        let runtime = builder
            .build()
            .map_err(|e| PluginError::RuntimeError(format!("Failed to create runtime: {}", e)))?;

        Ok(Self {
            runtime: Arc::new(runtime),
            shutdown_handle: ShutdownHandle::new(),
            config,
        })
    }

    /// Create a runtime with default configuration
    pub fn with_defaults() -> PluginResult<Self> {
        Self::new(RuntimeConfig::default())
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get a handle to the underlying Tokio runtime
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }

    /// Get a shutdown signal that can be used to detect shutdown
    pub fn shutdown_signal(&self) -> ShutdownSignal {
        self.shutdown_handle.signal()
    }

    /// Block on a future from a sync context
    ///
    /// This is the primary method for bridging sync FFI calls to async handlers.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.runtime.block_on(future)
    }

    /// Spawn a task on the runtime
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Spawn a blocking task
    pub fn spawn_blocking<F, R>(&self, func: F) -> tokio::task::JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.runtime.spawn_blocking(func)
    }

    /// Initiate graceful shutdown
    ///
    /// This signals all tasks to stop and waits for completion up to the timeout.
    pub fn shutdown(&self, timeout: std::time::Duration) -> PluginResult<()> {
        tracing::info!("Initiating runtime shutdown with timeout {:?}", timeout);

        // Signal shutdown to all tasks
        self.shutdown_handle.trigger();

        // Give tasks time to complete gracefully
        self.runtime.block_on(async {
            tokio::time::sleep(timeout).await;
        });

        tracing::info!("Runtime shutdown complete");
        Ok(())
    }

    /// Check if shutdown has been triggered
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_handle.is_triggered()
    }
}

impl Drop for AsyncRuntime {
    fn drop(&mut self) {
        // Ensure shutdown is triggered when runtime is dropped
        self.shutdown_handle.trigger();
    }
}

/// Thread-safe wrapper for optional runtime
pub struct RuntimeHolder {
    runtime: Mutex<Option<AsyncRuntime>>,
}

impl RuntimeHolder {
    /// Create a new empty runtime holder
    pub fn new() -> Self {
        Self {
            runtime: Mutex::new(None),
        }
    }

    /// Initialize the runtime
    pub fn init(&self, config: RuntimeConfig) -> PluginResult<()> {
        let mut guard = self.runtime.lock();
        if guard.is_some() {
            return Err(PluginError::RuntimeError(
                "Runtime already initialized".to_string(),
            ));
        }
        *guard = Some(AsyncRuntime::new(config)?);
        Ok(())
    }

    /// Execute a closure with the runtime
    pub fn with<F, R>(&self, f: F) -> PluginResult<R>
    where
        F: FnOnce(&AsyncRuntime) -> R,
    {
        let guard = self.runtime.lock();
        match guard.as_ref() {
            Some(rt) => Ok(f(rt)),
            None => Err(PluginError::RuntimeError("Runtime not initialized".to_string())),
        }
    }

    /// Shutdown and remove the runtime
    pub fn shutdown(&self, timeout: std::time::Duration) -> PluginResult<()> {
        let mut guard = self.runtime.lock();
        if let Some(rt) = guard.take() {
            rt.shutdown(timeout)?;
        }
        Ok(())
    }
}

impl Default for RuntimeHolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "runtime/runtime_tests.rs"]
mod runtime_tests;
