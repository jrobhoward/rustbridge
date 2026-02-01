# Chapter 7: Backpressure Queues

In this chapter, you'll implement bounded queues with backpressure for plugin calls. This pattern lets you control memory usage and throttle producers when the plugin can't keep up.

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

1. **Bounded Queue**: Requests are enqueued with a maximum capacity
2. **Blocking Submit**: Callers block when the queue is full
3. **Single Worker**: One thread drains the queue sequentially
4. **Future/Promise**: Callers receive a future that completes when processed

## Project Setup

Scaffold a new project with all consumer types:

```powershell
cd $env:USERPROFILE\rustbridge-workspace

rustbridge new sync-demo --all
cd sync-demo
```

## Adding a Sleep Handler

Replace `src\lib.rs` with a plugin that has a configurable sleep handler for testing backpressure. See the [Linux tutorial](../tutorials/07-backpressure-queues/README.md#adding-a-sleep-handler) for the complete Rust code.

## Build the Plugin and Bundle

```powershell
# Build the plugin
cargo build --release

# Create a bundle
rustbridge bundle create `
  --name sync-demo `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\sync_demo.dll `
  --output sync-demo-0.1.0.rbp

# Copy to each consumer directory
Copy-Item sync-demo-0.1.0.rbp consumers\csharp\
Copy-Item sync-demo-0.1.0.rbp consumers\java-jni\
Copy-Item sync-demo-0.1.0.rbp consumers\python\
```

> **Note**: The generated consumer code may reference a different bundle version (e.g., `0.1.0`). Update the bundle path in each consumer's source code to match the version you created, or create the bundle with the version the consumers expect.

## Sections

Implement the synchronized wrapper in each language:

### [01: C# Consumer](./01-csharp-consumer.md)

Implement synchronized access in C# using `BlockingCollection<T>` and `Task`.

### [02: Java/JNI Consumer](./02-java-jni-consumer.md)

Implement synchronized access in Java using `BlockingQueue` and `CompletableFuture`.

### [03: Python Consumer](./03-python-consumer.md)

Implement synchronized access in Python using `queue.Queue` and `concurrent.futures`.

## Key Concepts

### Backpressure

When the queue is full, `submit()` blocks the caller:

```
Producer Rate: 100 req/sec
Plugin Rate:   10 req/sec
Queue Size:    5

Result: After 5 requests queue, producers block until plugin catches up.
        Effective rate becomes 10 req/sec (plugin's rate).
```

### Graceful Shutdown

1. Stop accepting new requests
2. Drain the queue
3. Complete all pending futures
4. Shutdown the plugin

## What's Next?

Continue to [Chapter 8: Binary Transport](../08-binary-transport/README.md) to learn about high-performance binary data handling.
