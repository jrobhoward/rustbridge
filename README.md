# rustbridge

> [!WARNING]
> This project is in early development. APIs are unstable and may change without notice.

A framework for developing Rust shared libraries callable from other languages. Uses C ABI under the hood but abstracts the complexity, providing OSGI-like lifecycle, mandatory async (Tokio), logging callbacks, and JSON-based data transport.

## Overview

rustbridge lowers the barrier for creating Rust plugins that can be loaded and called from Java, C#, Python, Go, and other languages. Instead of manually managing FFI complexity, you implement a simple `Plugin` trait and rustbridge handles:

- **Memory management**: Safe buffer allocation and deallocation across FFI boundary
- **Async runtime**: Tokio runtime included in every plugin
- **Lifecycle management**: OSGI-inspired state machine (Installed → Starting → Active → Stopping → Stopped)
- **Logging**: Tracing integration with callbacks to host language
- **Serialization**: JSON-based message transport with typed envelopes

## Features

- **Cross-language interoperability**: Call Rust code from Java, Kotlin, C#, Python, and more
- **Multiple JVM implementations**: FFM for Java 21+ (modern, fast) and JNI for Java 8+ (compatibility)
- **Kotlin-friendly**: Idiomatic Kotlin usage with data classes, extension functions, and type-safe DSL
- **JSON-based transport**: Simple, universal data serialization
- **OSGI-inspired lifecycle**: Structured plugin startup and shutdown
- **Async-first**: Built on Tokio with mandatory async runtime
- **FFI logging**: Tracing integration with host language callbacks
- **Type-safe macros**: Procedural macros for reduced boilerplate
- **CLI tooling**: Project scaffolding and code generation

## Project Structure

```
rustbridge/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── rustbridge-core/          # Core traits, types, lifecycle
│   ├── rustbridge-transport/     # JSON codec, message envelopes
│   ├── rustbridge-ffi/           # C ABI exports, buffer management
│   ├── rustbridge-runtime/       # Tokio integration
│   ├── rustbridge-logging/       # Tracing → FFI callback bridge
│   ├── rustbridge-macros/        # Procedural macros
│   └── rustbridge-cli/           # Build tool and code generator
├── rustbridge-java/              # Java/Kotlin bindings
│   ├── rustbridge-core/          # Core interfaces
│   ├── rustbridge-ffm/           # FFM implementation (Java 21+)
│   └── rustbridge-jni/           # JNI fallback (Java 8+)
├── examples/
│   ├── hello-plugin/             # Example Rust plugin
│   └── kotlin-examples/          # Kotlin usage examples
├── docs/
│   ├── ARCHITECTURE.md           # System architecture
│   ├── SKILLS.md                 # Development best practices
│   ├── TESTING.md                # Testing conventions
│   └── TASKS.md                  # Project roadmap
└── CLAUDE.md                     # Project instructions for Claude Code
```

## Quick Start

> 📖 **New to rustbridge?** See the [complete Getting Started guide](./docs/GETTING_STARTED.md) for a step-by-step tutorial.

### Creating a Plugin (Rust)

```rust
use async_trait::async_trait;
use rustbridge_core::{Plugin, PluginContext, PluginError, PluginResult};
use rustbridge_macros::{rustbridge_entry, Message};
use serde::{Deserialize, Serialize};

// Define message types
#[derive(Debug, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
}

// Implement the plugin
#[derive(Default)]
pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("Plugin started");
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
                let resp = EchoResponse { message: req.message };
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("Plugin stopped");
        Ok(())
    }
}

// Generate FFI entry point
rustbridge_entry!(MyPlugin::default);

// Re-export FFI functions
pub use rustbridge_ffi::{
    plugin_init, plugin_call, plugin_free_buffer, plugin_shutdown,
    plugin_set_log_level, plugin_get_state,
};
```

### Using from Java (FFM, Java 21+)

```java
import com.rustbridge.ffm.FfmPluginLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginConfig;

try (Plugin plugin = FfmPluginLoader.load("libmyplugin.so")) {
    String response = plugin.call("echo", "{\"message\": \"Hello, World!\"}");
    System.out.println(response);  // {"message": "Hello, World!"}
}
```

### Using from Java (JNI, Java 8+)

```java
import com.rustbridge.jni.JniPluginLoader;
import com.rustbridge.Plugin;

try (Plugin plugin = JniPluginLoader.load("libmyplugin.so")) {
    String response = plugin.call("echo", "{\"message\": \"Hello!\"}");
    System.out.println(response);
}
```

### Using from Kotlin

```kotlin
import com.rustbridge.ffm.FfmPluginLoader

// Data classes for type-safe requests
data class EchoRequest(val message: String)
data class EchoResponse(val message: String, val length: Int)

// Extension function for typed calls
inline fun <reified T> Plugin.callTyped(messageType: String, request: Any): T {
    val mapper = ObjectMapper()
    val responseJson = call(messageType, mapper.writeValueAsString(request))
    return mapper.readValue(responseJson, T::class.java)
}

// Use block for automatic cleanup
FfmPluginLoader.load("libmyplugin.so").use { plugin ->
    val response = plugin.callTyped<EchoResponse>("echo", EchoRequest("Hello!"))
    println(response.message)
}
```

See [examples/kotlin-examples](./examples/kotlin-examples) for complete examples.

## FFI API

The following C functions are exported by plugins:

```c
// Create plugin instance (called by plugin_init internally)
void* plugin_create();

// Initialize plugin with config and optional log callback
void* plugin_init(
    void* plugin_ptr,
    const uint8_t* config_json,
    size_t config_len,
    void (*log_callback)(uint8_t level, const char* target, const uint8_t* msg, size_t len)
);

// Make a synchronous request
FfiBuffer plugin_call(
    void* handle,
    const char* type_tag,      // null-terminated
    const uint8_t* request,
    size_t request_len
);

// Free a buffer returned by plugin_call
void plugin_free_buffer(FfiBuffer* buffer);

// Shutdown the plugin
bool plugin_shutdown(void* handle);

// Set log level (0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error, 5=Off)
void plugin_set_log_level(void* handle, uint8_t level);

// Get current lifecycle state
uint8_t plugin_get_state(void* handle);

// Async API (placeholder for future)
uint64_t plugin_call_async(...);
bool plugin_cancel_async(void* handle, uint64_t request_id);
```

### FfiBuffer Structure

```c
typedef struct {
    uint8_t* data;      // Pointer to data
    size_t len;         // Data length
    size_t capacity;    // Allocation capacity
    uint32_t error_code; // 0 = success, non-zero = error
} FfiBuffer;
```

## Lifecycle States

```
Installed → Starting → Active → Stopping → Stopped
               ↑                    │
               └────────────────────┘ (restart)
           Any state → Failed (on error)
```

| State | Description |
|-------|-------------|
| `Installed` | Plugin created but not initialized |
| `Starting` | Initializing runtime, resources |
| `Active` | Ready to handle requests |
| `Stopping` | Graceful shutdown in progress |
| `Stopped` | Shutdown complete |
| `Failed` | Error occurred |

## Building

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Build a specific plugin
cargo build -p hello-plugin

# Run tests
cargo test

# Build CLI tool
cargo build -p rustbridge-cli
```

## CLI Usage

```bash
# Create a new plugin project
rustbridge new my-plugin

# Build a plugin
rustbridge build --release

# Generate host language bindings
rustbridge generate --lang java --output ./generated
rustbridge generate --lang csharp --output ./generated
rustbridge generate --lang python --output ./generated

# Validate manifest
rustbridge check
```

## Configuration

### Plugin Configuration (PluginConfig)

```json
{
  "worker_threads": 4,
  "log_level": "info",
  "max_concurrent_ops": 1000,
  "shutdown_timeout_ms": 5000,
  "data": {
    "custom_key": "custom_value"
  }
}
```

**Configuration Options:**

- **`worker_threads`** (optional): Number of async worker threads (default: number of CPU cores)
- **`log_level`**: Initial log level - "trace", "debug", "info", "warn", "error", "off" (default: "info")
- **`max_concurrent_ops`**: Maximum concurrent requests (default: 1000)
  - Set to `0` for unlimited (use with caution - can cause memory exhaustion)
  - Requests exceeding this limit are immediately rejected with error code 13 (TooManyRequests)
  - Monitor rejected requests using `plugin.getRejectedRequestCount()` (Java) or `handle.rejected_request_count()` (Rust)
- **`shutdown_timeout_ms`**: Maximum milliseconds to wait during shutdown (default: 5000)
- **`data`** (optional): Plugin-specific configuration data (JSON object)

**Example: Configuring concurrency limits in Java:**

```java
PluginConfig config = PluginConfig.defaults()
    .maxConcurrentOps(100)  // Limit to 100 concurrent requests
    .workerThreads(4)
    .logLevel(LogLevel.INFO);

try (Plugin plugin = FfmPluginLoader.load(pluginPath, config)) {
    // Make calls...

    // Monitor rejected requests
    long rejectedCount = plugin.getRejectedRequestCount();
    if (rejectedCount > 0) {
        System.out.println("Rejected " + rejectedCount + " requests due to concurrency limit");
    }
}
```

### Manifest (rustbridge.toml)

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "My awesome plugin"
authors = ["Your Name"]

[messages."user.create"]
description = "Create a new user"
request_schema = "schemas/CreateUserRequest.json"
response_schema = "schemas/CreateUserResponse.json"

[messages."user.delete"]
description = "Delete a user"

[platforms]
linux-x86_64 = "libmyplugin.so"
linux-aarch64 = "libmyplugin.so"
darwin-x86_64 = "libmyplugin.dylib"
darwin-aarch64 = "libmyplugin.dylib"
windows-x86_64 = "myplugin.dll"
```

## Error Handling

Errors are represented with stable numeric codes:

| Code | Error Type |
|------|-----------|
| 0 | Success |
| 1 | Invalid State |
| 2 | Initialization Failed |
| 3 | Shutdown Failed |
| 4 | Config Error |
| 5 | Serialization Error |
| 6 | Unknown Message Type |
| 7 | Handler Error |
| 8 | Runtime Error |
| 9 | Cancelled |
| 10 | Timeout |
| 11 | Internal Error |
| 12 | FFI Error |
| 13 | Too Many Requests (concurrency limit exceeded) |

## Target Languages

| Language | Status | Implementation |
|----------|--------|---------------|
| Java/Kotlin | Tier 1 | FFM (Java 21+) + JNI (Java 8+) |
| C# | Tier 2 | P/Invoke |
| Python | Tier 2 | ctypes/cffi |
| Go | Tier 3 | cgo |
| Erlang | Tier 3 | NIF |

## Documentation

- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - System architecture and design decisions
- [docs/SKILLS.md](./docs/SKILLS.md) - Development best practices and coding conventions
- [docs/TESTING.md](./docs/TESTING.md) - Testing conventions and guidelines
- [docs/TASKS.md](./docs/TASKS.md) - Project roadmap and task tracking

## Contributing

1. Read [docs/SKILLS.md](./docs/SKILLS.md) for coding conventions
2. Read [docs/TESTING.md](./docs/TESTING.md) for testing guidelines
3. Check [docs/TASKS.md](./docs/TASKS.md) for open tasks
4. Follow the git workflow (user controls commits)

## License

MIT OR Apache-2.0

### Attribution

This project includes software licensed under the Unicode License (Unicode-3.0). See [NOTICES](./NOTICES) for details.
