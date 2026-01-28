# Creating Plugins

This guide walks you through creating your first rustbridge plugin. By the end, you'll have a working plugin packaged as a `.rbp` bundle ready for distribution.

## What You'll Build

A calculator plugin that can:
- Add two numbers
- Multiply two numbers
- Return results in a structured JSON format

This demonstrates the core concepts: message types, request handling, and cross-language communication.

## Prerequisites

- **Rust 1.90.0+** (2024 edition required)
  ```bash
  rustc --version  # Should be >= 1.90.0
  ```
  Install or update: https://rustup.rs/

- **rustbridge CLI** (for bundling)
  ```bash
  cargo install rustbridge-cli
  ```

## Step 1: Create the Project

### Option A: Using rustbridge CLI (Recommended)

```bash
rustbridge new calculator-plugin
cd calculator-plugin
```

This scaffolds a complete project with the correct structure.

### Option B: Manual Setup

```bash
cargo new --lib calculator-plugin
cd calculator-plugin
```

Edit `Cargo.toml`:

```toml
[package]
name = "calculator-plugin"
version = "0.5.0"
edition = "2024"

[workspace]  # Standalone project (not part of a parent workspace)

[lib]
crate-type = ["cdylib"]  # Required for FFI

[dependencies]
# rustbridge dependencies
rustbridge-core = "0.5"
rustbridge-transport = "0.5"
rustbridge-ffi = "0.5"
rustbridge-runtime = "0.5"
rustbridge-logging = "0.5"
rustbridge-macros = "0.5"

async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

> **Important**:
> - `crate-type = ["cdylib"]` generates a dynamic library (.so/.dylib/.dll) loadable via FFI.
> - `[workspace]` ensures this project is standalone, even if created inside another workspace directory.

## Step 2: Define Message Types

Create `src/lib.rs` with your message types:

```rust
use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginContext, PluginError, PluginResult};
use rustbridge_macros::{Message, rustbridge_entry};
use serde::{Deserialize, Serialize};

// ============================================================================
// Message Types
// ============================================================================

/// Request to add two numbers
#[derive(Debug, Serialize, Deserialize, Message)]
#[message(tag = "math.add")]
pub struct AddRequest {
    pub a: i64,
    pub b: i64,
}

/// Response with addition result
#[derive(Debug, Serialize, Deserialize)]
pub struct AddResponse {
    pub result: i64,
}

/// Request to multiply two numbers
#[derive(Debug, Serialize, Deserialize, Message)]
#[message(tag = "math.multiply")]
pub struct MultiplyRequest {
    pub a: i64,
    pub b: i64,
}

/// Response with multiplication result
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiplyResponse {
    pub result: i64,
}
```

**Key points:**
- Request types use `#[derive(Message)]` with a `#[message(tag = "...")]` attribute
- Message tags follow a namespace convention (e.g., `math.add`, `user.create`)
- Response types are plain Serde types (no special macros needed)

## Step 3: Implement the Plugin Trait

Add the plugin implementation:

```rust
/// Calculator plugin implementation
#[derive(Default)]
pub struct CalculatorPlugin;

#[async_trait]
impl Plugin for CalculatorPlugin {
    /// Called when the plugin starts
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("Calculator plugin started");
        Ok(())
    }

    /// Handle incoming requests
    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "math.add" => {
                let req: AddRequest = serde_json::from_slice(payload)?;
                let result = req.a + req.b;
                tracing::debug!("Adding {} + {} = {}", req.a, req.b, result);

                let resp = AddResponse { result };
                Ok(serde_json::to_vec(&resp)?)
            }
            "math.multiply" => {
                let req: MultiplyRequest = serde_json::from_slice(payload)?;
                let result = req.a * req.b;
                tracing::debug!("Multiplying {} * {} = {}", req.a, req.b, result);

                let resp = MultiplyResponse { result };
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    /// Called when the plugin stops
    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("Calculator plugin stopped");
        Ok(())
    }
}
```

**The Plugin trait:**
- `on_start()` - Initialization (database connections, caches, etc.)
- `handle_request()` - Main entry point for all messages
- `on_stop()` - Cleanup (close connections, flush data)

## Step 4: Generate FFI Entry Point

Add the FFI exports at the end of `src/lib.rs`:

```rust
// Generate FFI entry point
rustbridge_entry!(CalculatorPlugin::default);

// Re-export FFI functions for the compiled library
pub use rustbridge_ffi::{
    plugin_init,
    plugin_call,
    plugin_free_buffer,
    plugin_shutdown,
    plugin_set_log_level,
    plugin_get_state,
    plugin_get_rejected_count,
};
```

**What does `rustbridge_entry!` do?**
- Creates a `plugin_create()` function for your plugin
- Sets up panic handlers for FFI safety
- Initializes the async runtime

## Step 5: Build the Plugin

```bash
# Release build (recommended for distribution)
cargo build --release

# Debug build (for development)
cargo build
```

This creates:
- **Linux**: `target/release/libcalculator_plugin.so`
- **macOS**: `target/release/libcalculator_plugin.dylib`
- **Windows**: `target/release/calculator_plugin.dll`

## Step 6: Create a Bundle

Package your plugin as a `.rbp` bundle for distribution:

```bash
# Linux
rustbridge bundle create \
  --name calculator-plugin \
  --version 0.1.0 \
  --lib linux-x86_64:target/release/libcalculator_plugin.so \
  --output calculator-plugin-0.1.0.rbp

# macOS (Apple Silicon)
rustbridge bundle create \
  --name calculator-plugin \
  --version 0.1.0 \
  --lib darwin-aarch64:target/release/libcalculator_plugin.dylib \
  --output calculator-plugin-0.1.0.rbp

# Windows
rustbridge bundle create \
  --name calculator-plugin \
  --version 0.1.0 \
  --lib windows-x86_64:target/release/calculator_plugin.dll \
  --output calculator-plugin-0.1.0.rbp
```

### Verify the Bundle

```bash
rustbridge bundle list calculator-plugin-0.1.0.rbp
```

Output:
```
Bundle: calculator-plugin v0.1.0
Bundle format: v1.0

Platforms:
  linux-x86_64:
    release:
      Library: lib/linux-x86_64/release/libcalculator_plugin.so
      Checksum: sha256:abc123...
```

## Step 7: Test Locally (Optional)

Create `tests/integration_test.rs`:

```rust
use rustbridge_core::{Plugin, PluginContext, PluginConfig};
use calculator_plugin::{CalculatorPlugin, AddRequest, AddResponse};

#[tokio::test]
async fn test_add() {
    let plugin = CalculatorPlugin::default();
    let ctx = PluginContext::new(PluginConfig::default());

    plugin.on_start(&ctx).await.unwrap();

    let request = AddRequest { a: 5, b: 3 };
    let request_bytes = serde_json::to_vec(&request).unwrap();

    let response_bytes = plugin
        .handle_request(&ctx, "math.add", &request_bytes)
        .await
        .unwrap();

    let response: AddResponse = serde_json::from_slice(&response_bytes).unwrap();
    assert_eq!(response.result, 8);

    plugin.on_stop(&ctx).await.unwrap();
}
```

Run tests:

```bash
cargo test
```

## Error Handling

Use `PluginError` for structured errors:

```rust
"math.divide" => {
    let req: DivideRequest = serde_json::from_slice(payload)?;

    if req.b == 0 {
        return Err(PluginError::HandlerError(
            "Division by zero: field 'b' cannot be zero".to_string()
        ));
    }

    let result = req.a / req.b;
    let resp = DivideResponse { result };
    Ok(serde_json::to_vec(&resp)?)
}
```

Errors are propagated to the host language with error codes and messages.

## Common Issues

### Missing `crate-type = ["cdylib"]`
**Symptom**: No .so/.dylib/.dll file generated.
**Fix**: Add `crate-type = ["cdylib"]` to `[lib]` in Cargo.toml.

### Missing FFI re-exports
**Symptom**: "Undefined symbol" when loading.
**Fix**: Ensure `pub use rustbridge_ffi::{...}` is in lib.rs.

### Panic in Rust code
**Symptom**: Error code 11 (InternalError).
**Fix**: Check logs for panic message. Use `Result` instead of `.unwrap()`.

## Next Steps

- **[Packaging Guide](../packaging/README.md)** - Multi-platform bundles, signing
- **[Using Plugins](../using-plugins/README.md)** - Load from Java/Kotlin/C#/Python
- **[Binary Transport](../TRANSPORT.md)** - Faster alternative to JSON

## Complete Example

See `examples/hello-plugin/` for a complete working example.
