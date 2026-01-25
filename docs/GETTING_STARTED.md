# Getting Started with rustbridge

This guide walks you through creating your first rustbridge plugin from scratch. By the end, you'll have a working plugin that can be called from Java, Kotlin, or other languages.

## What You'll Build

A simple calculator plugin that can:
- Add two numbers
- Multiply two numbers
- Return results in a structured format

This demonstrates the core concepts of rustbridge: message types, request handling, and cross-language communication.

## Prerequisites

Before starting, ensure you have:

- **Rust 1.85.0 or later** (Rust 2024 edition required)
  ```bash
  rustc --version  # Should be >= 1.85.0
  ```
  Install or update: https://rustup.rs/

- **Java 21+ for FFM** (recommended) or **Java 8+ for JNI**
  ```bash
  java --version
  ```

- **Gradle** (if using Java/Kotlin bindings)
  ```bash
  gradle --version
  ```

- **Basic familiarity with:**
  - Rust async/await
  - Serde for serialization
  - JSON

## Step 1: Create a New Plugin Project

### Option A: Using cargo (manual setup)

Create a new library project:

```bash
cargo new --lib calculator-plugin
cd calculator-plugin
```

### Option B: Using rustbridge CLI (if available)

```bash
rustbridge new calculator-plugin
cd calculator-plugin
```

> **Note**: The `rustbridge` CLI is part of the `rustbridge-cli` crate. Install with `cargo install rustbridge-cli` if you want scaffolding support.

## Step 2: Configure Cargo.toml

Edit your `Cargo.toml` to add rustbridge dependencies and configure the crate as a dynamic library:

```toml
[package]
name = "calculator-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]  # Required for FFI

[dependencies]
rustbridge-core = "0.1"
rustbridge-transport = "0.1"
rustbridge-ffi = "0.1"
rustbridge-runtime = "0.1"
rustbridge-logging = "0.1"
rustbridge-macros = "0.1"

async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

> **Important**: The `crate-type = ["cdylib"]` setting is required to generate a dynamic library (.so/.dylib/.dll) that can be loaded via FFI.

## Step 3: Define Message Types

Create `src/lib.rs` and define your message types. Each message type represents a request or response that crosses the FFI boundary.

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
- Request types use the `#[derive(Message)]` macro from rustbridge
- The `#[message(tag = "...")]` attribute defines the message type identifier
- Message tags follow a namespace convention (e.g., "math.add", "user.create")
- Response types are plain Serde types (no special macros needed)

## Step 4: Implement the Plugin Trait

Add your plugin implementation to the same file:

```rust
/// Calculator plugin implementation
#[derive(Default)]
pub struct CalculatorPlugin;

#[async_trait]
impl Plugin for CalculatorPlugin {
    /// Called when the plugin starts (after initialization)
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
                // Deserialize request
                let req: AddRequest = serde_json::from_slice(payload)?;

                // Process request
                let result = req.a + req.b;
                tracing::debug!("Adding {} + {} = {}", req.a, req.b, result);

                // Serialize response
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

**Understanding the Plugin trait:**
- `on_start()`: Initialization logic (optional, can be empty)
- `handle_request()`: Main entry point for all requests
  - Receives a type tag (the message type identifier)
  - Receives raw bytes (JSON payload)
  - Returns raw bytes (JSON response) or an error
- `on_stop()`: Cleanup logic (optional, but recommended for resource cleanup)

## Step 5: Generate FFI Entry Point

Add the FFI entry point macro and re-export FFI functions:

```rust
// Generate FFI entry point
rustbridge_entry!(CalculatorPlugin::default);

// Re-export FFI functions so they're available in the compiled library
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
- Generates a `plugin_create()` function that creates your plugin instance
- Sets up panic handlers for FFI safety
- Ensures proper initialization of the async runtime

## Step 6: Build Your Plugin

Build the plugin in release mode for optimal performance:

```bash
cargo build --release
```

This creates a dynamic library at:
- **Linux**: `target/release/libcalculator_plugin.so`
- **macOS**: `target/release/libcalculator_plugin.dylib`
- **Windows**: `target/release/calculator_plugin.dll`

For development and debugging, you can build without `--release`:

```bash
cargo build
```

This creates a debug build at `target/debug/libcalculator_plugin.{so,dylib,dll}` with symbols for debugging.

## Step 6b: Create a Plugin Bundle (Recommended)

Instead of distributing raw shared libraries, it's recommended to create a plugin bundle (`.rbp` file). Bundles provide:
- Single file for all platforms
- Metadata (name, version, description)
- SHA256 checksum verification
- Optional schema embedding
- Optional code signing (for production)

### Install rustbridge CLI

First, install the rustbridge CLI tool:

```bash
cargo install rustbridge-cli
```

### Create a Bundle

Create a bundle for your current platform:

```bash
# Linux
rustbridge bundle create \
  --name calculator-plugin \
  --version 0.1.0 \
  --lib linux-x86_64:target/release/libcalculator_plugin.so \
  --output calculator-plugin-0.1.0.rbp

# macOS (Intel)
rustbridge bundle create \
  --name calculator-plugin \
  --version 0.1.0 \
  --lib darwin-x86_64:target/release/libcalculator_plugin.dylib \
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

### Multi-Platform Bundles

For production, create a single bundle with libraries for all platforms:

```bash
rustbridge bundle create \
  --name calculator-plugin \
  --version 0.1.0 \
  --lib linux-x86_64:target/x86_64-unknown-linux-gnu/release/libcalculator_plugin.so \
  --lib linux-aarch64:target/aarch64-unknown-linux-gnu/release/libcalculator_plugin.so \
  --lib darwin-x86_64:target/x86_64-apple-darwin/release/libcalculator_plugin.dylib \
  --lib darwin-aarch64:target/aarch64-apple-darwin/release/libcalculator_plugin.dylib \
  --lib windows-x86_64:target/x86_64-pc-windows-msvc/release/calculator_plugin.dll \
  --output calculator-plugin-0.1.0.rbp
```

> **Note**: Cross-compilation requires appropriate Rust targets installed. See the Rust documentation on cross-compilation.

### List Bundle Contents

Verify your bundle was created correctly:

```bash
rustbridge bundle list calculator-plugin-0.1.0.rbp
```

Output:
```
Bundle: calculator-plugin v0.1.0
Bundle format: v1.0

Platforms:
  linux-x86_64:
    Library: lib/linux-x86_64/libcalculator_plugin.so
    Checksum: sha256:abc123...
    Size: 1.2 MB
```

### Code Signing (Optional)

For production deployments, you can sign your bundles to ensure authenticity. This is **optional** for development but recommended for production.

Code signing is covered in a separate guide. For now, unsigned bundles work perfectly for development and testing.

## Step 7: Test from Rust (Optional)

Before using your plugin from another language, you can test it with Rust integration tests:

Create `tests/integration_test.rs`:

```rust
use rustbridge_core::{Plugin, PluginContext, PluginConfig};
use calculator_plugin::{CalculatorPlugin, AddRequest, AddResponse};

#[tokio::test]
async fn test_add() {
    let plugin = CalculatorPlugin::default();
    let ctx = PluginContext::new(PluginConfig::default());

    // Start the plugin
    plugin.on_start(&ctx).await.unwrap();

    // Create request
    let request = AddRequest { a: 5, b: 3 };
    let request_bytes = serde_json::to_vec(&request).unwrap();

    // Call plugin
    let response_bytes = plugin
        .handle_request(&ctx, "math.add", &request_bytes)
        .await
        .unwrap();

    // Parse response
    let response: AddResponse = serde_json::from_slice(&response_bytes).unwrap();
    assert_eq!(response.result, 8);

    // Stop the plugin
    plugin.on_stop(&ctx).await.unwrap();
}
```

Run the tests:

```bash
cargo test
```

## Step 8: Use from Java (FFM - Java 21+)

### 8.1 Add rustbridge-java dependency

In your Java project's `build.gradle`:

```gradle
dependencies {
    implementation 'com.rustbridge:rustbridge-core:0.1.0'
    implementation 'com.rustbridge:rustbridge-ffm:0.1.0'
}
```

### 8.2 Load from Bundle (Recommended)

Load the plugin from a `.rbp` bundle file:

```java
import com.rustbridge.BundleLoader;
import com.rustbridge.ffm.FfmPluginLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginConfig;
import java.nio.file.Path;

public class CalculatorExample {
    public static void main(String[] args) {
        try {
            // Load bundle and extract library for current platform
            BundleLoader bundleLoader = BundleLoader.builder()
                .bundlePath("calculator-plugin-0.1.0.rbp")
                .verifySignatures(false)  // Skip signature verification (development)
                .build();

            Path libraryPath = bundleLoader.extractLibrary();

            // Load plugin from extracted library
            try (Plugin plugin = FfmPluginLoader.load(libraryPath.toString())) {
                // Call the add operation
                String request = "{\"a\": 10, \"b\": 20}";
                String response = plugin.call("math.add", request);
                System.out.println("Add result: " + response);
                // Output: Add result: {"result":30}

                // Call the multiply operation
                request = "{\"a\": 5, \"b\": 7}";
                response = plugin.call("math.multiply", request);
                System.out.println("Multiply result: " + response);
                // Output: Multiply result: {"result":35}
            }

            bundleLoader.close();  // Cleanup
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

**Bundle loading benefits:**
- Automatic platform detection (works on Linux, macOS, Windows)
- SHA256 checksum verification
- Metadata available (plugin name, version, description)
- Single file distribution

### 8.2b Load from Raw Library (Alternative)

For development, you can also load the raw shared library directly:

```java
import com.rustbridge.ffm.FfmPluginLoader;
import com.rustbridge.Plugin;

public class CalculatorExample {
    public static void main(String[] args) {
        // Direct path to compiled library (platform-specific)
        String pluginPath = "target/release/libcalculator_plugin.so";  // Linux
        // String pluginPath = "target/release/libcalculator_plugin.dylib";  // macOS
        // String pluginPath = "target/release/calculator_plugin.dll";  // Windows

        try (Plugin plugin = FfmPluginLoader.load(pluginPath)) {
            String request = "{\"a\": 10, \"b\": 20}";
            String response = plugin.call("math.add", request);
            System.out.println("Add result: " + response);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

> **Note**: Loading raw libraries requires platform-specific paths. Bundles are preferred for portability.

### 8.3 Custom configuration

You can customize the plugin behavior with `PluginConfig`:

```java
PluginConfig config = PluginConfig.defaults()
    .logLevel("debug")                    // Set log level
    .workerThreads(4)                     // Number of async worker threads
    .maxConcurrentOps(100)                // Limit concurrent operations
    .shutdownTimeoutMs(5000);             // Shutdown timeout

try (Plugin plugin = FfmPluginLoader.load(pluginPath, config)) {
    // Use plugin...
}
```

### 8.4 Log callback (optional)

To receive logs from the Rust plugin:

```java
import com.rustbridge.LogCallback;

LogCallback callback = (level, target, message) -> {
    System.out.printf("[%s] %s: %s%n", level, target, message);
};

try (Plugin plugin = FfmPluginLoader.load(pluginPath, config, callback)) {
    // Logs from Rust will be forwarded to your callback
    plugin.call("math.add", "{\"a\": 1, \"b\": 2}");
}
```

## Step 9: Use from Kotlin

Kotlin usage is even more ergonomic with data classes and extension functions:

```kotlin
import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader
import com.rustbridge.PluginConfig
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue

// Define data classes matching your Rust types
data class AddRequest(val a: Long, val b: Long)
data class AddResponse(val result: Long)

// Extension function for type-safe calls
val mapper = jacksonObjectMapper()

inline fun <reified T> com.rustbridge.Plugin.callTyped(
    messageType: String,
    request: Any
): T {
    val requestJson = mapper.writeValueAsString(request)
    val responseJson = call(messageType, requestJson)
    return mapper.readValue(responseJson)
}

fun main() {
    // Load from bundle (recommended)
    val bundleLoader = BundleLoader.builder()
        .bundlePath("calculator-plugin-0.1.0.rbp")
        .verifySignatures(false)  // Skip signature verification (development)
        .build()

    val libraryPath = bundleLoader.extractLibrary()
    val config = PluginConfig.defaults().logLevel("info")

    FfmPluginLoader.load(libraryPath.toString(), config).use { plugin ->
        // Type-safe calls with data classes
        val addResult = plugin.callTyped<AddResponse>(
            "math.add",
            AddRequest(a = 42, b = 58)
        )
        println("42 + 58 = ${addResult.result}")  // Output: 42 + 58 = 100
    }

    bundleLoader.close()
}

// Alternative: Load from raw library (development)
fun loadRawLibrary() {
    FfmPluginLoader.load("target/release/libcalculator_plugin.so").use { plugin ->
        val result = plugin.callTyped<AddResponse>(
            "math.add",
            AddRequest(a = 10, b = 20)
        )
        println("Result: ${result.result}")
    }
}
```

## Step 10: Error Handling

### In Rust

Use `PluginError` for structured errors:

```rust
async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    match type_tag {
        "math.divide" => {
            let req: DivideRequest = serde_json::from_slice(payload)?;

            // Check for division by zero
            if req.b == 0 {
                return Err(PluginError::InvalidInput {
                    field: "b".to_string(),
                    message: "Division by zero".to_string(),
                });
            }

            let result = req.a / req.b;
            let resp = DivideResponse { result };
            Ok(serde_json::to_vec(&resp)?)
        }
        _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
    }
}
```

### In Java/Kotlin

Errors are propagated as `PluginException`:

```java
try {
    String response = plugin.call("math.divide", "{\"a\": 10, \"b\": 0}");
} catch (PluginException e) {
    System.err.println("Plugin error: " + e.getMessage());
    System.err.println("Error code: " + e.getErrorCode());
    // Output:
    // Plugin error: Invalid input for field 'b': Division by zero
    // Error code: 2
}
```

## Next Steps

Congratulations! You've created your first rustbridge plugin. Here are some next steps:

### 1. Learn Best Practices
- Read [docs/SKILLS.md](./SKILLS.md) for development best practices
- Review [docs/ARCHITECTURE.md](./ARCHITECTURE.md) to understand the system design
- See [docs/TESTING.md](./TESTING.md) for testing conventions

### 2. Advanced Features
- **Binary transport**: For performance-critical paths, see [docs/BINARY_TRANSPORT.md](./BINARY_TRANSPORT.md)
- **Code generation**: Generate Java classes from Rust types with `rustbridge generate` (see [docs/CODE_GENERATION.md](./CODE_GENERATION.md))
- **Code signing**: Sign your bundles with minisign for production security (separate guide coming soon)
- **Schema embedding**: Include JSON schemas and C headers in your bundles for self-documenting APIs

### 3. Real-World Patterns
- **Database connections**: Use `on_start()` to establish connections, `on_stop()` to close them
- **Caching**: Store state in your plugin struct (wrapped in `Arc<RwLock<_>>` for thread safety)
- **Configuration**: Access config values via `ctx.config()`
- **Async operations**: Use Tokio for HTTP requests, database queries, etc.

### 4. Explore Examples
- Check out `examples/hello-plugin/` for a comprehensive example
- See `examples/kotlin-examples/` for Kotlin usage patterns

### 5. Debugging
When things don't work as expected:
- Enable debug logging: `config.logLevel("debug")`
- Use a log callback to see what's happening in Rust
- Check `cargo build` output for compilation errors
- Verify your library path is correct
- See [docs/DEBUGGING.md](./DEBUGGING.md) (coming soon) for detailed troubleshooting

## Common Gotchas

### 1. Forgot `crate-type = ["cdylib"]`
**Symptom**: Build succeeds but no .so/.dylib/.dll file is generated.
**Solution**: Add `crate-type = ["cdylib"]` to `[lib]` section in Cargo.toml.

### 2. Bundle not found
**Symptom**: `FileNotFoundException` when loading bundle.
**Solution**: Verify the `.rbp` file path is correct relative to your working directory. Use absolute paths or check current directory.

### 3. Wrong platform in bundle
**Symptom**: "Platform not supported" error when loading bundle.
**Solution**: Ensure your bundle includes a library for your platform. Use `rustbridge bundle list` to check. Add missing platform with `--lib platform:path`.

### 4. Checksum verification failed
**Symptom**: "Checksum verification failed" error.
**Solution**: The library file was corrupted or modified after bundle creation. Rebuild the bundle with fresh libraries.

### 5. Missing FFI re-exports
**Symptom**: "Undefined symbol" errors when loading the plugin.
**Solution**: Ensure you have `pub use rustbridge_ffi::{...}` at the bottom of lib.rs.

### 6. JSON serialization mismatch
**Symptom**: Deserialization errors or unexpected null values.
**Solution**: Ensure your Java/Kotlin JSON matches your Rust struct fields exactly (case-sensitive).

### 7. Panic in Rust code
**Symptom**: Plugin returns error code 11 (InternalError).
**Solution**: Panics are caught at the FFI boundary. Check your logs for panic messages. Fix the panic or use `Result` for error handling.

### 8. rustbridge CLI not found
**Symptom**: `rustbridge: command not found` when creating bundles.
**Solution**: Install the CLI with `cargo install rustbridge-cli`. Ensure `~/.cargo/bin` is in your PATH.

## Getting Help

- **Documentation**: See the `docs/` directory for detailed guides
- **Examples**: Review `examples/` for working code
- **Issues**: Report bugs or ask questions at https://github.com/yourorg/rustbridge/issues

## Summary

You've learned how to:
- ✅ Create a rustbridge plugin project
- ✅ Define message types with Serde
- ✅ Implement the Plugin trait
- ✅ Build a dynamic library
- ✅ Create plugin bundles (.rbp files)
- ✅ Load and call the plugin from Java/Kotlin
- ✅ Handle errors gracefully
- ✅ Configure logging and runtime behavior

Your plugin is now ready to be distributed and used in production applications!

**Next steps for production:**
- Create multi-platform bundles with libraries for all target platforms
- Add schema files to your bundle for API documentation
- Consider adding code signing for security (see separate guide)
