# rustbridge-consumer

Rust consumer for dynamically loading and invoking rustbridge plugins.

## Overview

This crate enables Rust applications to load and invoke rustbridge plugin bundles (`.rbp` files) or shared libraries at runtime. It provides the same functionality available to Java, Kotlin, C#, and Python consumers.

## Key Features

- **Dynamic Loading**: Load plugins at runtime without compile-time linking
- **Bundle Support**: Load from `.rbp` bundles with automatic platform detection
- **JSON Transport**: Make calls using JSON serialization
- **Binary Transport**: High-performance binary struct transport (7x faster)
- **Lifecycle Management**: Full OSGI-inspired lifecycle state machine
- **Logging Integration**: Route plugin logs through host callbacks

## Usage

### Loading a Shared Library

```rust
use rustbridge_consumer::{NativePluginLoader, ConsumerResult};

fn main() -> ConsumerResult<()> {
    // Load plugin from shared library
    let plugin = NativePluginLoader::load("target/release/libmy_plugin.so")?;

    // Make a JSON call
    let response = plugin.call("echo", r#"{"message": "Hello"}"#)?;
    println!("Response: {response}");

    // Plugin is automatically shut down when dropped
    Ok(())
}
```

### Loading from a Bundle

```rust
use rustbridge_consumer::{NativePluginLoader, PluginConfig, ConsumerResult};

fn main() -> ConsumerResult<()> {
    let config = PluginConfig::default();

    // Load from bundle (extracts library for current platform)
    let plugin = NativePluginLoader::load_bundle_with_config(
        "my-plugin-1.0.0.rbp",
        &config,
        None,  // No log callback
    )?;

    // Make typed calls with automatic serialization
    let response: MyResponse = plugin.call_typed("my.handler", &MyRequest { ... })?;

    Ok(())
}
```

### With Logging

```rust
use rustbridge_consumer::{NativePluginLoader, LogCallbackFn};
use rustbridge_core::LogLevel;

fn main() -> ConsumerResult<()> {
    // Create a log callback
    let log_callback: LogCallbackFn = Box::new(|level, target, message| {
        println!("[{level}] {target}: {message}");
    });

    let plugin = NativePluginLoader::load_with_config(
        "target/release/libmy_plugin.so",
        &PluginConfig::default(),
        Some(log_callback),
    )?;

    // Plugin logs will now be routed to our callback
    Ok(())
}
```

### Binary Transport

For performance-critical paths, use binary transport:

```rust
use rustbridge_consumer::NativePluginLoader;

fn main() -> ConsumerResult<()> {
    let plugin = NativePluginLoader::load("target/release/libmy_plugin.so")?;

    // Check if binary transport is available
    if plugin.has_binary_transport() {
        // Make binary call (message_id maps to registered handler)
        let response_bytes = plugin.call_raw(1, &request_bytes)?;
    }

    Ok(())
}
```

## Plugin Lifecycle

Plugins follow an OSGI-inspired lifecycle:

```
Installed -> Starting -> Active -> Stopping -> Stopped
                           |
                      (any) -> Failed
```

- **Installed**: Plugin loaded but not initialized
- **Starting**: Plugin is initializing
- **Active**: Plugin is ready to handle requests
- **Stopping**: Plugin is shutting down
- **Stopped**: Plugin has been shut down
- **Failed**: Plugin encountered a fatal error

## Error Handling

All operations return `ConsumerResult<T>` which wraps `ConsumerError`:

```rust
use rustbridge_consumer::{ConsumerError, ConsumerResult};

fn handle_plugin() -> ConsumerResult<()> {
    let plugin = NativePluginLoader::load("path/to/plugin.so")?;

    match plugin.call("handler", "{}") {
        Ok(response) => println!("Success: {response}"),
        Err(ConsumerError::NotActive(state)) => {
            println!("Plugin not active: {state:?}");
        }
        Err(ConsumerError::CallFailed(e)) => {
            println!("Call failed with code {}: {e}", e.error_code());
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
```

## License

MIT OR Apache-2.0
