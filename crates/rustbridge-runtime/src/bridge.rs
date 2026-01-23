//! Bridge between sync FFI calls and async handlers

use crate::{AsyncRuntime, ShutdownSignal};
use rustbridge_core::{PluginError, PluginResult};
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Bridge for executing async operations from sync FFI context
pub struct AsyncBridge {
    runtime: Arc<AsyncRuntime>,
    request_counter: AtomicU64,
}

impl AsyncBridge {
    /// Create a new async bridge
    pub fn new(runtime: Arc<AsyncRuntime>) -> Self {
        Self {
            runtime,
            request_counter: AtomicU64::new(0),
        }
    }

    /// Get the next request ID
    pub fn next_request_id(&self) -> u64 {
        self.request_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Execute an async operation synchronously (blocking)
    ///
    /// This is the primary method for handling sync FFI calls.
    pub fn call_sync<F, T>(&self, future: F) -> PluginResult<T>
    where
        F: Future<Output = PluginResult<T>>,
    {
        if self.runtime.is_shutting_down() {
            return Err(PluginError::RuntimeError(
                "Runtime is shutting down".to_string(),
            ));
        }
        self.runtime.block_on(future)
    }

    /// Execute an async operation with timeout
    pub fn call_sync_timeout<F, T>(
        &self,
        future: F,
        timeout: std::time::Duration,
    ) -> PluginResult<T>
    where
        F: Future<Output = PluginResult<T>>,
    {
        self.runtime.block_on(async move {
            match tokio::time::timeout(timeout, future).await {
                Ok(result) => result,
                Err(_) => Err(PluginError::Timeout),
            }
        })
    }

    /// Spawn an async task and return a handle
    pub fn spawn<F, T>(&self, future: F) -> tokio::task::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Get a shutdown signal
    pub fn shutdown_signal(&self) -> ShutdownSignal {
        self.runtime.shutdown_signal()
    }

    /// Check if the runtime is shutting down
    pub fn is_shutting_down(&self) -> bool {
        self.runtime.is_shutting_down()
    }
}

/// Type alias for async completion callback used in FFI
#[allow(dead_code)] // Reserved for future async API
pub type CompletionCallback = extern "C" fn(
    context: *mut std::ffi::c_void,
    request_id: u64,
    data: *const u8,
    len: usize,
    error_code: u32,
);

/// Pending async request tracker
#[allow(dead_code)] // Reserved for future async API
pub struct PendingRequest {
    pub request_id: u64,
    pub callback: CompletionCallback,
    pub context: *mut std::ffi::c_void,
    pub cancel_handle: Option<tokio::task::JoinHandle<()>>,
}

// Safety: The context pointer is provided by the host and assumed to be thread-safe
unsafe impl Send for PendingRequest {}
unsafe impl Sync for PendingRequest {}

#[allow(dead_code)] // Reserved for future async API
impl PendingRequest {
    /// Create a new pending request
    pub fn new(
        request_id: u64,
        callback: CompletionCallback,
        context: *mut std::ffi::c_void,
    ) -> Self {
        Self {
            request_id,
            callback,
            context,
            cancel_handle: None,
        }
    }

    /// Complete the request with success
    ///
    /// # Safety
    /// The callback and context must be valid for the duration of this call.
    pub unsafe fn complete_success(&self, data: &[u8]) {
        (self.callback)(self.context, self.request_id, data.as_ptr(), data.len(), 0);
    }

    /// Complete the request with error
    ///
    /// # Safety
    /// The callback and context must be valid for the duration of this call.
    pub unsafe fn complete_error(&self, error_code: u32, message: &str) {
        let msg_bytes = message.as_bytes();
        (self.callback)(
            self.context,
            self.request_id,
            msg_bytes.as_ptr(),
            msg_bytes.len(),
            error_code,
        );
    }
}

#[cfg(test)]
#[path = "bridge/bridge_tests.rs"]
mod bridge_tests;
