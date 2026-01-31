# Chapter 7: Backpressure Queues

In this chapter, you'll implement bounded queues with backpressure for plugin calls. This pattern lets you control
memory usage and throttle producers when the plugin can't keep up—callers block when the queue is full rather than
consuming unbounded memory.

## What You'll Build

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Backpressure Queue Pattern                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Caller Threads                     Synchronized Wrapper                    │
│  ──────────────                     ────────────────────                    │
│                                                                             │
│  Thread 1 ──┐                      ┌─────────────────────┐                  │
│             │    submit()          │  Bounded Queue      │                  │
│  Thread 2 ──┼───────────────────►  │  ┌───┬───┬───┬───┐  │                  │
│             │    (blocks if full)  │  │ R │ R │ R │   │  │                  │
│  Thread 3 ──┘                      │  └───┴───┴───┴───┘  │                  │
│                                    │         │          │                  │
│                                    │         ▼          │                  │
│                                    │  ┌─────────────┐   │                  │
│                                    │  │   Worker    │   │                  │
│  ◄─────────────────────────────────│  │   Thread    │   │                  │
│     Future/Task completes          │  └──────┬──────┘   │                  │
│     with response                  │         │          │                  │
│                                    └─────────┼──────────┘                  │
│                                              ▼                             │
│                                    ┌─────────────────┐                     │
│                                    │   sync-demo     │                     │
│                                    │   plugin        │                     │
│                                    └─────────────────┘                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## The Pattern

This tutorial implements a **bounded queue with backpressure**:

1. **Bounded Queue**: Requests are enqueued with a maximum capacity (e.g., 5, 100, 1000)
2. **Blocking Submit**: Callers block when the queue is full—this is the backpressure
3. **Single Worker**: One thread drains the queue and processes requests sequentially
4. **Future/Promise**: Callers receive a future that completes when their request is processed

### When to Use This Pattern

This pattern is ideal when:

- **Memory control**: You want predictable memory usage regardless of producer rate
- **Flow control**: Producers should slow down when the consumer can't keep up
- **Resource constraints**: The plugin wraps a single connection (database, hardware, network)
- **Simplicity**: You want to avoid reasoning about concurrent plugin state

### When NOT to Use This Pattern

Skip this pattern when:

- **High throughput needed**: The plugin is thread-safe and can handle concurrent calls
- **Independent requests**: Requests don't share state and can run in parallel
- **Low latency critical**: The serialization overhead is unacceptable

## Project Setup

First, scaffold a new project with all consumer types:

```bash
cd ~/rustbridge-workspace

rustbridge new sync-demo --all
cd sync-demo
```

This creates:

```
sync-demo/
├── Cargo.toml                      # Rust plugin
├── src/
│   └── lib.rs                      # Plugin implementation
└── consumers/
    ├── kotlin/                     # Kotlin/FFM consumer
    ├── java-ffm/                   # Java FFM consumer
    ├── java-jni/                   # Java JNI consumer
    ├── csharp/                     # C# consumer
    └── python/                     # Python consumer
```

## Adding a Sleep Handler

To demonstrate backpressure, we need a message handler that takes a configurable amount of time.
Replace the contents of `src/lib.rs` with:

```rust
//! sync-demo - A plugin demonstrating synchronized access patterns

use rustbridge::prelude::*;
use rustbridge::{serde_json, tokio, tracing};

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

/// Request to sleep for a specified duration (for testing backpressure)
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "sleep")]
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
// Plugin Implementation
// ============================================================================

#[derive(Default)]
pub struct SyncDemoPlugin;

impl SyncDemoPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_echo(&self, req: EchoRequest) -> PluginResult<EchoResponse> {
        tracing::debug!("Handling echo: {:?}", req);
        Ok(EchoResponse {
            length: req.message.len(),
            message: req.message,
        })
    }

    async fn handle_sleep(&self, req: SleepRequest) -> PluginResult<SleepResponse> {
        tracing::debug!("Sleeping for {}ms", req.duration_ms);
        tokio::time::sleep(tokio::time::Duration::from_millis(req.duration_ms)).await;
        Ok(SleepResponse {
            slept_ms: req.duration_ms,
        })
    }
}

#[async_trait]
impl Plugin for SyncDemoPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("sync-demo plugin started");
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
            "sleep" => {
                let req: SleepRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_sleep(req).await?;
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("sync-demo plugin stopped");
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["echo", "sleep"]
    }
}

// Generate the FFI entry point
rustbridge_entry!(SyncDemoPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Build the Plugin and Bundle

```bash
# Build the plugin
cargo build --release

# Create a bundle
rustbridge bundle create \
  --name sync-demo \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libsync_demo.so \
  --output sync-demo-1.0.0.rbp

# Copy to each consumer directory
cp sync-demo-1.0.0.rbp consumers/csharp/
cp sync-demo-1.0.0.rbp consumers/java-jni/
cp sync-demo-1.0.0.rbp consumers/python/
```

> **Note**: Adjust the `--lib` platform identifier for your OS:
> - Linux: `linux-x86_64:target/release/libsync_demo.so`
> - macOS (Intel): `darwin-x86_64:target/release/libsync_demo.dylib`
> - macOS (Apple Silicon): `darwin-aarch64:target/release/libsync_demo.dylib`
> - Windows: `windows-x86_64:target/release/sync_demo.dll`

## Sections

Now implement the synchronized wrapper in each language:

### [01: C# Consumer](./01-csharp-consumer.md)

Implement synchronized access in C# using `BlockingCollection<T>` and `Task`.

### [02: Java/JNI Consumer](./02-java-jni-consumer.md)

Implement synchronized access in Java using `BlockingQueue` and `CompletableFuture`.

### [03: Python Consumer](./03-python-consumer.md)

Implement synchronized access in Python using `queue.Queue` and `concurrent.futures`.

## Prerequisites

Before starting this chapter:

- **Completed Chapter 1** (understanding plugin structure and message types)
- **Language-specific setup**:
  - C#: .NET 8.0+
  - Java: JDK 17+ (for JNI)
  - Python: Python 3.10+

## Key Concepts

### Backpressure

When the queue is full, `submit()` blocks the caller. This naturally throttles producers:

```
Producer Rate: 100 req/sec
Plugin Rate:   10 req/sec
Queue Size:    5

Result: After 5 requests queue, producers block until plugin catches up.
        Effective rate becomes 10 req/sec (plugin's rate).
```

### Graceful Shutdown

The wrapper must handle shutdown cleanly:

1. Stop accepting new requests
2. Drain the queue (process remaining requests)
3. Complete all pending futures
4. Shutdown the plugin

### Error Handling

When a request fails:

1. The error is captured
2. The corresponding future completes with the error
3. Other queued requests are unaffected
4. The worker continues processing

## What You'll Learn

By completing this chapter, you'll understand:

- How to bootstrap a multi-language rustbridge project
- Serializing access to a shared resource
- Implementing backpressure with bounded queues
- Using futures/promises for async result delivery
- Graceful shutdown patterns for worker threads
- Testing concurrent access patterns

## Next Steps

After completing this chapter, you'll have production-ready patterns for serialized plugin access in three languages.
These patterns can be adapted for other use cases like connection pooling or rate limiting.

Continue to Chapter 8 to learn about binary transport for high-performance scenarios with large payloads.

[Continue to Chapter 8: Binary Transport](../08-binary-transport/README.md)
