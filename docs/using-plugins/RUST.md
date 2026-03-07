# Getting Started: Rust

This guide walks you through using rustbridge plugins from Rust using the `rustbridge-consumer` crate.

## Prerequisites

- **Rust 1.90 or later** - Edition 2024
  ```bash
  rustc --version  # Should be >= 1.90.0
  ```
- **A rustbridge plugin** - Either a `.rbp` bundle or native library

## Important: Separate Projects

Unlike other rustbridge consumers (Java, C#, Python) which are scaffolded by the CLI, **Rust consumers must be created as separate projects** using `cargo new`. This avoids Cargo workspace conflicts between the consumer and plugin.

```bash
# Create a new consumer project
cargo new my-rust-consumer
cd my-rust-consumer
```

## Add Dependencies

Add `rustbridge-consumer` to your `Cargo.toml`:

```toml
[dependencies]
rustbridge-consumer = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

For building against rustbridge source instead of the published crate, see the [Development Guide](../DEVELOPMENT.md#rust).

## Loading a Plugin

### From Bundle (Recommended)

```rust
use rustbridge_consumer::{NativePluginLoader, ConsumerResult};

fn main() -> ConsumerResult<()> {
    // Load plugin from bundle (auto-extracts for current platform)
    let plugin = NativePluginLoader::load_bundle("my-plugin-1.0.0.rbp")?;

    let response = plugin.call("echo", r#"{"message": "Hello"}"#)?;
    println!("{response}");

    Ok(())
}
```

### From Raw Library

```rust
use rustbridge_consumer::NativePluginLoader;

fn main() -> rustbridge_consumer::ConsumerResult<()> {
    // Platform-specific path
    #[cfg(target_os = "linux")]
    let plugin_path = "target/release/libmyplugin.so";
    #[cfg(target_os = "macos")]
    let plugin_path = "target/release/libmyplugin.dylib";
    #[cfg(windows)]
    let plugin_path = "target/release/myplugin.dll";

    let plugin = NativePluginLoader::load(plugin_path)?;

    let response = plugin.call("echo", r#"{"message": "Hello"}"#)?;
    println!("{response}");

    Ok(())
}
```

### By Name (Auto-Search)

```rust
use rustbridge_consumer::NativePluginLoader;

// Searches ./target/release, ./target/debug, and system paths
// for libmyplugin.so (Linux), libmyplugin.dylib (macOS), myplugin.dll (Windows)
let plugin = NativePluginLoader::load_by_name("myplugin")?;
```

## Making JSON Calls

```rust
use rustbridge_consumer::NativePluginLoader;

let plugin = NativePluginLoader::load(plugin_path)?;

// Simple call
let response = plugin.call("echo", r#"{"message": "Hello, World!"}"#)?;
println!("{response}");

// With serde_json for dynamic JSON
let request = serde_json::json!({
    "message": "Hello"
});
let response = plugin.call("echo", &serde_json::to_string(&request)?)?;
let result: serde_json::Value = serde_json::from_str(&response)?;

println!("Message: {}", result["message"]);
println!("Length: {}", result["length"]);
```

## Type-Safe Calls with Serde

```rust
use rustbridge_consumer::NativePluginLoader;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct EchoRequest {
    message: String,
}

#[derive(Deserialize)]
struct EchoResponse {
    message: String,
    length: usize,
}

let plugin = NativePluginLoader::load(plugin_path)?;

// Type-safe call with automatic serialization/deserialization
let response: EchoResponse = plugin.call_typed("echo", &EchoRequest {
    message: "Hello, Rust!".to_string(),
})?;

println!("Message: {}", response.message);
println!("Length: {}", response.length);
```

## Configuration

```rust
use rustbridge_consumer::{NativePluginLoader, PluginConfig, LogLevel};

let config = PluginConfig {
    log_level: LogLevel::Debug,
    worker_threads: Some(4),
    max_concurrent_ops: Some(100),
    shutdown_timeout_ms: Some(5000),
    ..PluginConfig::default()
};

let plugin = NativePluginLoader::load_with_config(plugin_path, &config, None)?;
```

## Logging

```rust
use rustbridge_consumer::{NativePluginLoader, LogCallbackFn, LogLevel, PluginConfig};
use std::sync::Arc;

// Create a log callback
let log_callback: LogCallbackFn = Arc::new(|level, target, message| {
    println!("[{:?}] {}: {}", level, target, message);
});

let config = PluginConfig {
    log_level: LogLevel::Debug,
    ..PluginConfig::default()
};

let plugin = NativePluginLoader::load_with_config(
    plugin_path,
    &config,
    Some(log_callback),
)?;

// Change log level dynamically
plugin.set_log_level(LogLevel::Warn);
```

## Binary Transport (Advanced)

For performance-critical paths, use binary transport with `#[repr(C)]` structs.

### Define Structs

```rust
const MSG_ECHO: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct EchoRequestRaw {
    version: u8,
    _reserved: [u8; 3],
    message: [u8; 256],
    message_len: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EchoResponseRaw {
    version: u8,
    _reserved: [u8; 3],
    message: [u8; 256],
    message_len: u32,
    length: u32,
}
```

### Make Binary Calls

```rust
// Build request
let mut request = EchoRequestRaw {
    version: 1,
    _reserved: [0; 3],
    message: [0; 256],
    message_len: 5,
};
request.message[..5].copy_from_slice(b"Hello");

// Serialize to bytes
let request_bytes: &[u8] = unsafe {
    std::slice::from_raw_parts(
        &request as *const _ as *const u8,
        std::mem::size_of::<EchoRequestRaw>(),
    )
};

// Check if binary transport is available
if plugin.has_binary_transport() {
    // Call binary transport
    let response_bytes = plugin.call_raw(MSG_ECHO, request_bytes)?;

    // Parse response
    let result: &EchoResponseRaw = unsafe {
        &*(response_bytes.as_ptr() as *const EchoResponseRaw)
    };
    println!("Length: {}", result.length);
}
```

## Error Handling

```rust
use rustbridge_consumer::{ConsumerError, NativePluginLoader};

let plugin = NativePluginLoader::load(plugin_path)?;

match plugin.call("invalid.type", "{}") {
    Ok(response) => println!("Response: {response}"),
    Err(ConsumerError::CallFailed(plugin_error)) => {
        println!("Plugin error: {}", plugin_error);
        // Access error code via plugin_error.code()
    }
    Err(ConsumerError::NotActive(state)) => {
        println!("Plugin not active, state: {:?}", state);
    }
    Err(e) => {
        println!("Other error: {e}");
    }
}
```

### Error Types

| Error | Meaning |
|-------|---------|
| `ConsumerError::LibraryLoad` | Failed to load shared library |
| `ConsumerError::MissingSymbol` | Required FFI symbol not found |
| `ConsumerError::NullHandle` | Plugin initialization returned null |
| `ConsumerError::InitFailed` | Plugin failed to initialize |
| `ConsumerError::CallFailed` | Plugin call returned an error |
| `ConsumerError::NotActive` | Plugin not in Active state |
| `ConsumerError::InvalidResponse` | Response parsing failed |
| `ConsumerError::Bundle` | Bundle loading/extraction error |

## Async Usage with Tokio

```rust
use rustbridge_consumer::NativePluginLoader;
use std::sync::Arc;

#[tokio::main]
async fn main() -> rustbridge_consumer::ConsumerResult<()> {
    let plugin = Arc::new(NativePluginLoader::load(plugin_path)?);

    // Spawn blocking task for FFI call
    let plugin_clone = plugin.clone();
    let response = tokio::task::spawn_blocking(move || {
        plugin_clone.call("echo", r#"{"message": "Hello"}"#)
    })
    .await
    .expect("Task panicked")?;

    println!("{response}");
    Ok(())
}
```

### Concurrent Calls

```rust
use rustbridge_consumer::NativePluginLoader;
use std::sync::Arc;

#[tokio::main]
async fn main() -> rustbridge_consumer::ConsumerResult<()> {
    let plugin = Arc::new(NativePluginLoader::load(plugin_path)?);

    let tasks: Vec<_> = (0..100)
        .map(|i| {
            let plugin = plugin.clone();
            tokio::task::spawn_blocking(move || {
                plugin.call("echo", &format!(r#"{{"message": "Message {}"}}"#, i))
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(tasks).await;
    println!("Completed {} calls", results.len());

    Ok(())
}
```

## Monitoring

```rust
use rustbridge_consumer::{NativePluginLoader, LifecycleState};

let plugin = NativePluginLoader::load(plugin_path)?;

// Check plugin state
let state = plugin.state();
println!("State: {:?}", state);  // LifecycleState::Active

// Monitor rejected requests (due to concurrency limits)
let rejected_count = plugin.rejected_request_count();
if rejected_count > 0 {
    println!("Rejected: {} requests", rejected_count);
}
```

## Signature Verification

For production use, verify bundle signatures:

```rust
use rustbridge_consumer::{NativePluginLoader, PluginConfig};

// Load with signature verification (recommended for production)
let plugin = NativePluginLoader::load_bundle_with_verification(
    "my-plugin-1.0.0.rbp",
    &PluginConfig::default(),
    None,   // no log callback
    true,   // verify signatures
    None,   // use manifest's public key
)?;

// Or with a specific public key
let plugin = NativePluginLoader::load_bundle_with_verification(
    "my-plugin-1.0.0.rbp",
    &PluginConfig::default(),
    None,
    true,
    Some("RWS..."),  // override public key
)?;
```

## Plugin Lifecycle

The plugin implements `Drop`, so it's automatically shut down when it goes out of scope:

```rust
{
    let plugin = NativePluginLoader::load(plugin_path)?;
    plugin.call("echo", r#"{"message": "Hello"}"#)?;
}  // Plugin automatically shut down here

// Or explicit shutdown with error handling
let plugin = NativePluginLoader::load(plugin_path)?;
plugin.call("echo", r#"{"message": "Hello"}"#)?;
plugin.shutdown()?;  // Returns Result
```

## Thread Safety

`NativePlugin` implements `Send` and `Sync`, so it can be shared across threads:

```rust
use rustbridge_consumer::NativePluginLoader;
use std::sync::Arc;
use std::thread;

let plugin = Arc::new(NativePluginLoader::load(plugin_path)?);

let handles: Vec<_> = (0..4)
    .map(|i| {
        let plugin = plugin.clone();
        thread::spawn(move || {
            plugin.call("echo", &format!(r#"{{"message": "Thread {}"}}"#, i))
        })
    })
    .collect();

for handle in handles {
    let response = handle.join().expect("Thread panicked")?;
    println!("{response}");
}
```

## Complete Example

```rust
use rustbridge_consumer::{
    ConsumerResult, LogCallbackFn, LogLevel, NativePluginLoader, PluginConfig,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
struct AddRequest {
    a: i64,
    b: i64,
}

#[derive(Deserialize)]
struct AddResponse {
    result: i64,
}

fn main() -> ConsumerResult<()> {
    // Configure with logging
    let config = PluginConfig {
        log_level: LogLevel::Info,
        ..PluginConfig::default()
    };

    let log_callback: LogCallbackFn = Arc::new(|level, _target, message| {
        println!("[{:?}] {}", level, message);
    });

    // Load plugin
    let plugin = NativePluginLoader::load_with_config(
        "target/release/libcalculator_plugin.so",
        &config,
        Some(log_callback),
    )?;

    // Make typed call
    let response: AddResponse = plugin.call_typed("math.add", &AddRequest { a: 42, b: 58 })?;

    println!("42 + 58 = {}", response.result);

    Ok(())
}
```

## Memory Model

The Rust consumer follows the same "plugin allocates, plugin frees" pattern as other languages. Even though both sides are Rust, **they have separate heaps** because the plugin is a shared library with its own allocator.

1. Consumer serializes request and passes pointer to plugin
2. Plugin allocates response on its heap
3. Consumer **copies** the response data (via `serde_json::from_slice`)
4. Consumer calls `plugin_free_buffer` to release plugin memory
5. Consumer now owns an independent copy

This is handled automatically by `rustbridge-consumer` - you just get a `String` result.

See [../MEMORY_MODEL.md](../MEMORY_MODEL.md) for details.

## Performance Notes

Rust consumers have the lowest overhead among all supported languages since there's no GC or interpreter involved. However, there's still FFI crossing and JSON serialization overhead.

| Transport | Typical Latency |
|-----------|-----------------|
| Binary | ~90 ns |
| JSON | ~650 ns |

Binary transport is **7x faster** than JSON.

## Testing

See [../TESTING_RUST_CONSUMER.md](../TESTING_RUST_CONSUMER.md) for testing conventions.

```bash
# Run consumer tests
cargo test -p rustbridge-consumer

# Run integration tests (requires building hello-plugin first)
cargo build --release -p hello-plugin
cargo test -p rustbridge-consumer -- --ignored
```

## Related Documentation

- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership patterns
- [../TESTING_RUST_CONSUMER.md](../TESTING_RUST_CONSUMER.md) - Testing conventions
- [../ERROR_HANDLING.md](../ERROR_HANDLING.md) - Error codes and handling patterns
