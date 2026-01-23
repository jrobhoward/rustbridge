//! Graceful shutdown support

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Handle for triggering shutdown
#[derive(Clone)]
pub struct ShutdownHandle {
    triggered: Arc<AtomicBool>,
    sender: broadcast::Sender<()>,
}

impl ShutdownHandle {
    /// Create a new shutdown handle
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self {
            triggered: Arc::new(AtomicBool::new(false)),
            sender,
        }
    }

    /// Trigger shutdown
    pub fn trigger(&self) {
        if !self.triggered.swap(true, Ordering::SeqCst) {
            // Only send if this is the first trigger
            let _ = self.sender.send(());
        }
    }

    /// Check if shutdown has been triggered
    pub fn is_triggered(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    /// Get a signal that can be used to detect shutdown
    pub fn signal(&self) -> ShutdownSignal {
        ShutdownSignal {
            triggered: self.triggered.clone(),
            receiver: self.sender.subscribe(),
        }
    }
}

impl Default for ShutdownHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Signal for detecting shutdown (cloneable, can be passed to tasks)
pub struct ShutdownSignal {
    triggered: Arc<AtomicBool>,
    receiver: broadcast::Receiver<()>,
}

impl ShutdownSignal {
    /// Check if shutdown has been triggered (non-blocking)
    pub fn is_triggered(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    /// Wait for shutdown to be triggered
    ///
    /// Returns immediately if already triggered.
    pub async fn wait(&mut self) {
        if self.is_triggered() {
            return;
        }
        let _ = self.receiver.recv().await;
    }

    /// Create a future that completes when shutdown is triggered
    ///
    /// This is useful for select! statements.
    pub fn notified(&mut self) -> impl std::future::Future<Output = ()> + '_ {
        async move {
            self.wait().await;
        }
    }
}

impl Clone for ShutdownSignal {
    fn clone(&self) -> Self {
        Self {
            triggered: self.triggered.clone(),
            receiver: self.receiver.resubscribe(),
        }
    }
}

#[cfg(test)]
#[path = "shutdown/shutdown_tests.rs"]
mod shutdown_tests;
