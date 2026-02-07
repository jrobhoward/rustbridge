# Section 3: Rust Consumer

In this section, you'll implement synchronized plugin access in Rust using `crossbeam-channel` and `std::thread`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new sync-demo --all`
2. Add the sleep handler to `src/lib.rs`
3. Build the plugin and create the bundle

## Create a Rust Consumer Project

Unlike other language consumers (which live under `consumers/`), the Rust consumer should be created as a
**separate standalone project**. This avoids Cargo workspace conflicts that occur when placing a Rust
project inside another Rust project.

```bash
cd $RUSTBRIDGE_WORKSPACE
cargo new sync-demo-rust-consumer
cd sync-demo-rust-consumer
```

## Add Dependencies

Update `Cargo.toml`:

```toml
[package]
name = "sync-demo-consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
rustbridge-consumer = "0.9.1"
crossbeam-channel = "0.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Understanding the Pattern

The synchronized plugin wrapper:

1. **Bounded channel**: Requests queue up with a maximum capacity
2. **Blocking send**: Callers block when the channel is full (backpressure)
3. **Single worker**: One thread processes requests sequentially
4. **Oneshot response**: Each request includes a channel for its response

```
+------------------+
| Caller Threads   |     bounded channel       +----------------+
|                  |                           |                |
| Thread 1 --------+---> [R] [R] [R] [  ] ---> | Worker Thread  |
| Thread 2 --------+     (blocks if full)      | (processes     |
| Thread 3 --------+                           |  sequentially) |
|                  |                           |                |
| <--- oneshot <---+---------------------------|                |
+------------------+                           +----------------+
                                                      |
                                                      v
                                               +------------+
                                               |   Plugin   |
                                               +------------+
```

## Implement the SynchronizedPlugin

Create `src/main.rs`:

```rust
//! Synchronized plugin demo - Rust consumer

use crossbeam_channel::{bounded, Sender};
use rustbridge_consumer::{ConsumerError, ConsumerResult, NativePlugin, NativePluginLoader};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

// ============================================================================
// Message Types (matching the plugin)
// ============================================================================

#[derive(Debug, Serialize)]
struct EchoRequest {
    message: String,
}

#[derive(Debug, Deserialize)]
struct EchoResponse {
    message: String,
    length: usize,
}

#[derive(Debug, Serialize)]
struct SleepRequest {
    duration_ms: u64,
}

#[derive(Debug, Deserialize)]
struct SleepResponse {
    #[allow(dead_code)]
    slept_ms: u64,
}

// ============================================================================
// Work Item
// ============================================================================

/// A pending plugin call with its response channel.
struct WorkItem {
    type_tag: String,
    request_json: String,
    response_tx: crossbeam_channel::Sender<Result<String, String>>,
}

// ============================================================================
// SynchronizedPlugin
// ============================================================================

/// Thread-safe wrapper that serializes all plugin calls through a single worker thread.
pub struct SynchronizedPlugin {
    work_tx: Option<Sender<WorkItem>>,
    worker_handle: Option<JoinHandle<()>>,
}

impl SynchronizedPlugin {
    /// Create a new synchronized plugin wrapper.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The native plugin to wrap
    /// * `queue_size` - Maximum number of pending requests (enables backpressure)
    ///
    /// # Errors
    ///
    /// Returns an error if the worker thread cannot be spawned.
    pub fn new(plugin: NativePlugin, queue_size: usize) -> Result<Self, std::io::Error> {
        let (work_tx, work_rx) = bounded::<WorkItem>(queue_size);

        // Spawn worker thread
        let worker_handle = thread::Builder::new()
            .name("SynchronizedPlugin-Worker".into())
            .spawn(move || {
                for work_item in work_rx {
                    let result = plugin.call(&work_item.type_tag, &work_item.request_json);

                    let response = match result {
                        Ok(json) => Ok(json),
                        Err(e) => Err(e.to_string()),
                    };

                    // Ignore send errors (caller may have dropped)
                    let _ = work_item.response_tx.send(response);
                }

                // When channel closes, shutdown plugin
                let _ = plugin.shutdown();
            })?;

        Ok(Self {
            work_tx: Some(work_tx),
            worker_handle: Some(worker_handle),
        })
    }

    /// Get the number of pending requests in the queue.
    pub fn pending_count(&self) -> usize {
        self.work_tx.as_ref().map(|tx| tx.len()).unwrap_or(0)
    }

    /// Make a JSON call to the plugin.
    ///
    /// Blocks if the queue is full (backpressure).
    /// Returns when the request has been processed.
    pub fn call(&self, type_tag: &str, request_json: &str) -> Result<String, String> {
        let (response_tx, response_rx) = crossbeam_channel::bounded(1);

        let work_item = WorkItem {
            type_tag: type_tag.to_string(),
            request_json: request_json.to_string(),
            response_tx,
        };

        // This blocks if queue is full - backpressure!
        self.work_tx
            .as_ref()
            .ok_or_else(|| "Plugin shut down".to_string())?
            .send(work_item)
            .map_err(|_| "Plugin shut down".to_string())?;

        // Wait for response
        response_rx
            .recv()
            .map_err(|_| "Worker thread terminated".to_string())?
    }

    /// Make a typed call with automatic serialization.
    pub fn call_typed<Req, Res>(&self, type_tag: &str, request: &Req) -> Result<Res, String>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        let request_json =
            serde_json::to_string(request).map_err(|e| format!("Serialize error: {e}"))?;

        let response_json = self.call(type_tag, &request_json)?;

        serde_json::from_str(&response_json).map_err(|e| format!("Deserialize error: {e}"))
    }
}

impl Drop for SynchronizedPlugin {
    fn drop(&mut self) {
        // Close the channel to signal shutdown
        drop(self.work_tx.take());

        // Wait for worker to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

// SynchronizedPlugin is Send + Sync because it only holds Send types
unsafe impl Send for SynchronizedPlugin {}
unsafe impl Sync for SynchronizedPlugin {}

// ============================================================================
// Demo
// ============================================================================

fn main() -> ConsumerResult<()> {
    println!("=== Synchronized Plugin Demo (Rust) ===\n");

    // Load the plugin from bundle
    let bundle_path = "../sync-demo/sync-demo-0.1.0.rbp";
    let plugin = NativePluginLoader::load_bundle(bundle_path)?;

    // Wrap with synchronized access (queue size = 5 for demo)
    let sync_plugin = Arc::new(SynchronizedPlugin::new(plugin, 5)?);

    // Demo 1: Sequential calls
    println!("Demo 1: Sequential calls");
    for i in 0..3 {
        let response: EchoResponse = sync_plugin
            .call_typed("echo", &EchoRequest {
                message: format!("Message {i}"),
            })
            .map_err(ConsumerError::InvalidResponse)?;

        println!("  Echo: {} (len={})", response.message, response.length);
    }

    // Demo 2: Concurrent calls showing serialization
    println!("\nDemo 2: Concurrent calls (observe serialization)");
    run_concurrent_demo(&sync_plugin, 10, 100)?;

    // Demo 3: Backpressure
    println!("\nDemo 3: Backpressure (queue size = 5)");
    run_backpressure_demo(&sync_plugin, 20, 50)?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn run_concurrent_demo(
    sync_plugin: &Arc<SynchronizedPlugin>,
    thread_count: usize,
    sleep_ms: u64,
) -> ConsumerResult<()> {
    let start = Instant::now();

    let handles: Vec<_> = (0..thread_count)
        .map(|id| {
            let plugin = Arc::clone(sync_plugin);
            thread::spawn(move || -> Result<(), String> {
                println!("  [{id}] Submitting (queue: {})", plugin.pending_count());

                let _response: SleepResponse =
                    plugin.call_typed("sleep", &SleepRequest { duration_ms: sleep_ms })?;

                println!("  [{id}] Completed after {:?}", start.elapsed());
                Ok(())
            })
        })
        .collect();

    // Collect results from all threads
    let mut errors = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => errors.push(e),
            Err(_) => errors.push("Thread panicked".to_string()),
        }
    }

    if !errors.is_empty() {
        return Err(ConsumerError::InvalidResponse(errors.join("; ")));
    }

    let total = start.elapsed();
    println!("\nTotal time: {total:?}");
    println!(
        "  (Expected ~{}ms for {} x {}ms if serialized)",
        thread_count as u64 * sleep_ms,
        thread_count,
        sleep_ms
    );

    Ok(())
}

fn run_backpressure_demo(
    sync_plugin: &Arc<SynchronizedPlugin>,
    thread_count: usize,
    sleep_ms: u64,
) -> ConsumerResult<()> {
    let start = Instant::now();

    let handles: Vec<_> = (0..thread_count)
        .map(|id| {
            let plugin = Arc::clone(sync_plugin);
            thread::spawn(move || -> Result<(), String> {
                let submit_time = start.elapsed();

                let _response: SleepResponse =
                    plugin.call_typed("sleep", &SleepRequest { duration_ms: sleep_ms })?;

                let complete_time = start.elapsed();
                println!(
                    "  [{id:02}] Submit@{:?}, Complete@{:?}",
                    submit_time, complete_time
                );
                Ok(())
            })
        })
        .collect();

    // Collect results from all threads
    let mut errors = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => errors.push(e),
            Err(_) => errors.push("Thread panicked".to_string()),
        }
    }

    if !errors.is_empty() {
        return Err(ConsumerError::InvalidResponse(errors.join("; ")));
    }

    println!("\nTotal time: {:?}", start.elapsed());

    Ok(())
}
```

## Run the Demo

First, build the plugin and create the bundle if you haven't:

```bash
cd $RUSTBRIDGE_WORKSPACE/sync-demo
cargo build --release
rustbridge pack --no-sign
```

Then run the consumer:

```bash
cd $RUSTBRIDGE_WORKSPACE/sync-demo-rust-consumer
cargo run --release
```

Expected output:

```
=== Synchronized Plugin Demo (Rust) ===

Demo 1: Sequential calls
  Echo: Message 0 (len=9)
  Echo: Message 1 (len=9)
  Echo: Message 2 (len=9)

Demo 2: Concurrent calls (observe serialization)
  [0] Submitting (queue: 0)
  [1] Submitting (queue: 1)
  [2] Submitting (queue: 2)
  ...
  [0] Completed after 102.34ms
  [1] Completed after 203.12ms
  ...

Total time: 1.01s
  (Expected ~1000ms for 10 x 100ms if serialized)

Demo 3: Backpressure (queue size = 5)
  [00] Submit@1.23ms, Complete@52.45ms
  [01] Submit@1.45ms, Complete@103.67ms
  ...
  [18] Submit@650.12ms, Complete@1002.34ms
  [19] Submit@700.45ms, Complete@1052.67ms

Total time: 1.05s

=== Demo Complete ===
```

## Key Observations

### Serialization

Even though 10 requests are submitted concurrently, they complete sequentially (~100ms apart).
Total time is approximately `10 x 100ms = 1000ms`.

### Backpressure

With a queue size of 5:
- First 6 requests submit immediately (1 processing + 5 queued)
- Requests 6+ block until queue has space
- Submit times show delays as callers wait for queue capacity

## Understanding the Implementation

### Bounded Channel

```rust
let (work_tx, work_rx) = bounded::<WorkItem>(queue_size);
```

- **Bounded capacity**: Limits queue size, enabling backpressure
- **Thread-safe**: Safe for multiple senders
- **Blocking send**: `send()` blocks when channel is full

### Oneshot Response Pattern

```rust
let (response_tx, response_rx) = crossbeam_channel::bounded(1);

// Send work item with response channel
self.work_tx.send(work_item)?;

// Wait for response
response_rx.recv()?
```

- **Bridge**: Connects the caller's blocking recv to the worker's response
- **Capacity 1**: Only one response expected per request
- **Error propagation**: Errors are sent through the channel

### Graceful Shutdown

```rust
impl Drop for SynchronizedPlugin {
    fn drop(&mut self) {
        // Close the channel to signal shutdown
        drop(self.work_tx.take());

        // Wait for worker to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}
```

When the `SynchronizedPlugin` is dropped:
1. The work channel is closed
2. The worker drains remaining requests
3. The worker calls `plugin.shutdown()`
4. The main thread waits for the worker to finish

## Error Handling

Errors in plugin calls are propagated to the caller:

```rust
match sync_plugin.call("invalid.tag", "{}") {
    Ok(response) => println!("Response: {response}"),
    Err(e) => println!("Error: {e}"),
}
```

## What's Next?

You've now implemented synchronized plugin access in C#, Python, and Rust. Each implementation demonstrates:

- Bounded queues for memory control
- Backpressure through blocking sends
- Single-worker serialization
- Graceful shutdown

Continue to Chapter 8 to learn about binary transport for high-performance scenarios.

[Continue to Chapter 8: Binary Transport](../08-binary-transport/README.md)
