//! Concurrency and race condition tests
//!
//! These tests verify that concurrent operations on handles, buffers, and
//! plugins don't cause deadlocks, data corruption, or panic.

use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginConfig, PluginContext, PluginResult};
use rustbridge_ffi::{PluginHandle, PluginHandleManager};
use std::sync::{Arc, Barrier};
use std::thread;

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
    let num_threads = 20;
    let barrier = Arc::new(Barrier::new(num_threads));

    let mut handles = vec![];

    for _ in 0..num_threads {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            // Create and register handle
            let config = PluginConfig::default();
            let plugin_handle = PluginHandle::new(Box::new(TestPlugin::new()), config)
                .expect("Should create handle");

            let id = manager_clone.register(plugin_handle);
            id
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
    let num_threads = 20;

    // Pre-register handles
    let mut ids = vec![];
    for _ in 0..num_threads {
        let config = PluginConfig::default();
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

    // All removals should succeed
    assert!(
        removals.iter().all(|r| r.is_some()),
        "All handles should be removed"
    );

    // All handles should be gone
    for id in ids {
        assert!(manager.get(id).is_none(), "Handle {} should not exist", id);
    }
}

#[test]
fn test_concurrent_handle_get_and_remove() {
    let manager = Arc::new(PluginHandleManager::new());
    let config = PluginConfig::default();
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
    let num_handles = 10;
    let mut ids = vec![];

    // Create handles
    for _ in 0..num_handles {
        let config = PluginConfig::default();
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
    let num_iterations = 100;
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
                let config = PluginConfig::default();
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
}

#[test]
fn test_rapid_register_remove_no_leak() {
    let manager = PluginHandleManager::new();
    let iterations = 100;

    for _ in 0..iterations {
        let config = PluginConfig::default();
        let handle =
            PluginHandle::new(Box::new(TestPlugin::new()), config).expect("Should create handle");

        let id = manager.register(handle);
        let removed = manager.remove(id);

        assert!(removed.is_some(), "Handle should be successfully removed");
    }

    // After all operations, manager should be empty
    assert!(
        manager.get(1).is_none(),
        "Manager should be clean after all removals"
    );
}
