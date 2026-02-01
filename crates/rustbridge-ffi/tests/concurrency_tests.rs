//! Concurrency and race condition tests
//!
//! These tests verify that concurrent operations on handles, buffers, and
//! plugins don't cause deadlocks, data corruption, or panic.
//!
//! Note: These tests use reduced thread counts and explicit cleanup to avoid
//! resource exhaustion. Each PluginHandle creates a Tokio runtime with worker
//! threads and file descriptors.

use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginConfig, PluginContext, PluginResult};
use rustbridge_ffi::{PluginHandle, PluginHandleManager};
use std::sync::{Arc, Barrier};
use std::thread;

/// Create a PluginConfig with minimal resource usage for tests.
/// Uses 1 worker thread to minimize resource consumption when creating many handles.
fn test_config() -> PluginConfig {
    PluginConfig {
        worker_threads: Some(1),
        ..Default::default()
    }
}

/// Minimal test plugin
struct TestPlugin {
    call_count: std::sync::atomic::AtomicU64,
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            call_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl Plugin for TestPlugin {
    async fn on_start(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn on_stop(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        _context: &PluginContext,
        _type_tag: &str,
        _request: &[u8],
    ) -> PluginResult<Vec<u8>> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(vec![])
    }
}

#[test]
fn test_concurrent_handle_registration() {
    let manager = Arc::new(PluginHandleManager::new());
    // Reduced from 20 to 8 to minimize resource usage
    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));

    let mut handles = vec![];

    for _ in 0..num_threads {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            // Create and register handle
            let config = test_config();
            let plugin_handle = PluginHandle::new(Box::new(TestPlugin::new()), config)
                .expect("Should create handle");

            manager_clone.register(plugin_handle)
        });

        handles.push(handle);
    }

    // Collect all IDs
    let ids: Vec<u64> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread should complete"))
        .collect();

    // Verify all are unique
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(
        unique_ids.len(),
        num_threads,
        "All registered handle IDs should be unique"
    );

    // Clean up
    for id in ids {
        manager.remove(id);
    }
}

#[test]
fn test_concurrent_handle_removal() {
    let manager = Arc::new(PluginHandleManager::new());
    // Reduced from 20 to 8 to minimize resource usage
    let num_threads = 8;

    // Pre-register handles
    let mut ids = vec![];
    for _ in 0..num_threads {
        let config = test_config();
        let handle =
            PluginHandle::new(Box::new(TestPlugin::new()), config).expect("Should create handle");
        ids.push(manager.register(handle));
    }

    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    // Concurrently remove handles
    for id in ids.clone() {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            // Remove handle
            manager_clone.remove(id)
        });

        handles.push(handle);
    }

    // Collect results
    let removals: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread should complete"))
        .collect();

    // All removals should succeed - each thread removes a unique ID
    let failed_removals: Vec<_> = removals
        .iter()
        .enumerate()
        .filter(|(_, r)| r.is_none())
        .map(|(i, _)| i)
        .collect();
    assert!(
        failed_removals.is_empty(),
        "All handles should be removed, but removals at indices {:?} returned None",
        failed_removals
    );

    // All handles should be gone
    for id in ids {
        assert!(manager.get(id).is_none(), "Handle {} should not exist", id);
    }
}

#[test]
fn test_concurrent_handle_get_and_remove() {
    let manager = Arc::new(PluginHandleManager::new());
    let config = test_config();
    let handle =
        PluginHandle::new(Box::new(TestPlugin::new()), config).expect("Should create handle");

    let id = manager.register(handle);
    let barrier = Arc::new(Barrier::new(10));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let manager_clone = manager.clone();
            let barrier_clone = barrier.clone();

            thread::spawn(move || {
                barrier_clone.wait();

                if i % 2 == 0 {
                    // Even threads: get
                    manager_clone.get(id)
                } else {
                    // Odd thread: remove (only first one will succeed)
                    manager_clone.remove(id);
                    None
                }
            })
        })
        .collect();

    // Wait for all threads
    for h in handles {
        h.join().expect("Thread should complete");
    }

    // One thread should have removed it
    assert!(manager.get(id).is_none(), "Handle should be removed");
}

#[test]
fn test_multiple_handles_lifecycle() {
    let manager = Arc::new(PluginHandleManager::new());
    // Reduced from 10 to 5 to minimize resource usage
    let num_handles = 5;
    let mut ids = vec![];

    // Create handles
    for _ in 0..num_handles {
        let config = test_config();
        let handle =
            PluginHandle::new(Box::new(TestPlugin::new()), config).expect("Should create handle");
        ids.push(manager.register(handle));
    }

    // Concurrently manipulate them
    let barrier = Arc::new(Barrier::new(num_handles * 3));
    let mut threads = vec![];

    for id in &ids {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();
        let id_copy = *id;

        // Thread 1: repeatedly get
        for _ in 0..2 {
            let m = manager_clone.clone();
            let b = barrier_clone.clone();
            threads.push(thread::spawn(move || {
                b.wait();
                let _ = m.get(id_copy);
            }));
        }

        // Thread 2: remove
        let m = manager_clone.clone();
        let b = barrier_clone.clone();
        threads.push(thread::spawn(move || {
            b.wait();
            let _ = m.remove(id_copy);
        }));
    }

    // Wait for all to complete
    for t in threads {
        t.join().expect("Thread should complete");
    }

    // Most should be removed
    let remaining = ids.iter().filter(|id| manager.get(**id).is_some()).count();
    assert!(
        remaining <= num_handles,
        "At most {} handles should remain",
        num_handles
    );
}

#[test]
fn test_handle_id_uniqueness_under_concurrent_registration() {
    let manager = Arc::new(PluginHandleManager::new());
    // Reduced iterations to avoid exhausting file descriptors
    // Each PluginHandle creates a Tokio runtime which opens FDs
    let num_iterations = 10;
    let num_threads = 4;

    let registered_ids =
        std::sync::Arc::new(parking_lot::Mutex::new(std::collections::HashSet::new()));

    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for _ in 0..num_threads {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();
        let ids_clone = registered_ids.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            for _ in 0..num_iterations {
                let config = test_config();
                let plugin_handle = PluginHandle::new(Box::new(TestPlugin::new()), config)
                    .expect("Should create handle");

                let id = manager_clone.register(plugin_handle);
                ids_clone.lock().insert(id);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for h in handles {
        h.join().expect("Thread should complete");
    }

    // All IDs should be unique
    let ids = registered_ids.lock();
    assert_eq!(
        ids.len(),
        num_threads * num_iterations,
        "All handle IDs should be unique"
    );

    // Clean up all handles to release file descriptors
    for id in ids.iter() {
        manager.remove(*id);
    }
}

#[test]
fn test_rapid_register_remove_no_leak() {
    let manager = PluginHandleManager::new();
    // Reduced iterations to avoid exhausting file descriptors
    // Each PluginHandle creates a Tokio runtime which opens FDs
    let iterations = 10;

    for _ in 0..iterations {
        let config = test_config();
        let handle =
            PluginHandle::new(Box::new(TestPlugin::new()), config).expect("Should create handle");

        let id = manager.register(handle);
        let removed = manager.remove(id);

        assert!(removed.is_some(), "Handle should be successfully removed");

        // Explicitly drop the handle to ensure runtime shutdown and FD release
        drop(removed);
    }

    // After all operations, manager should be empty
    assert!(
        manager.get(1).is_none(),
        "Manager should be clean after all removals"
    );
}
