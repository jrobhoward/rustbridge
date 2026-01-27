# Using Plugins

This guide covers loading and using rustbridge plugins from your application. Choose your language below for detailed instructions.

## Language Guides

| Language | Guide | Requirements | Notes |
|----------|-------|--------------|-------|
| **Kotlin** | [KOTLIN.md](./KOTLIN.md) | Java 21+ | Idiomatic Kotlin with data classes |
| **Java (FFM)** | [JAVA_FFM.md](./JAVA_FFM.md) | Java 21+ | Recommended for new projects |
| **Java (JNI)** | [JAVA_JNI.md](./JAVA_JNI.md) | Java 17+ | Legacy support for Java 17-20 |
| **C#** | [CSHARP.md](./CSHARP.md) | .NET 6.0+ | P/Invoke-based loader |
| **Python** | [PYTHON.md](./PYTHON.md) | Python 3.9+ | ctypes-based loader |

## Quick Start

All languages follow the same pattern:

1. **Load the bundle** - Open the `.rbp` file
2. **Extract the library** - Get the native library for your platform
3. **Initialize the plugin** - Create a plugin instance with configuration
4. **Call methods** - Send JSON messages, receive JSON responses
5. **Cleanup** - Close the plugin when done

### Example (Kotlin)

```kotlin
val bundleLoader = BundleLoader.builder()
    .bundlePath("my-plugin-1.0.0.rbp")
    .verifySignatures(false)  // Set true for production
    .build()

val libraryPath = bundleLoader.extractLibrary()

FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
    val response = plugin.call("echo", """{"message": "Hello"}""")
    println(response)  // {"message": "Hello", "length": 5}
}

bundleLoader.close()
```

### Example (C#)

```csharp
using var bundleLoader = BundleLoader.Create()
    .WithBundlePath("my-plugin-1.0.0.rbp")
    .WithSignatureVerification(false)
    .Build();

var libraryPath = bundleLoader.ExtractLibrary();

using var plugin = NativePluginLoader.Load(libraryPath);
var response = plugin.Call("echo", "{\"message\": \"Hello\"}");
Console.WriteLine(response);
```

### Example (Python)

```python
loader = BundleLoader(verify_signatures=False)
plugin = loader.load("my-plugin-1.0.0.rbp")

response = plugin.call("echo", '{"message": "Hello"}')
print(response)

plugin.close()
```

## Core Concepts

### Bundle Loading

A `.rbp` bundle is a ZIP file containing native libraries for multiple platforms. The loader:

1. Opens the bundle
2. Reads `manifest.json` for metadata
3. Verifies checksums (and optionally signatures)
4. Extracts the correct library for your OS/architecture
5. Loads the library via FFI

### Signature Verification

Bundles can be signed for authenticity. Verification is **enabled by default** in production:

```kotlin
// Development - skip verification
BundleLoader.builder()
    .bundlePath("plugin.rbp")
    .verifySignatures(false)
    .build()

// Production - verify (default)
BundleLoader.builder()
    .bundlePath("plugin.rbp")
    .verifySignatures(true)
    .publicKey("RWS...")  // Optional: override manifest key
    .build()
```

### Plugin Configuration

Configure runtime behavior:

```kotlin
val config = PluginConfig.defaults()
    .logLevel("debug")          // trace, debug, info, warn, error
    .workerThreads(4)           // Tokio runtime threads
    .maxConcurrentOps(100)      // Limit concurrent operations
    .shutdownTimeoutMs(5000)    // Graceful shutdown timeout
```

### Message Format

All messages are JSON. The `type_tag` identifies the operation:

```kotlin
// Request
plugin.call("math.add", """{"a": 10, "b": 20}""")

// Response
// {"result": 30}
```

### Error Handling

Errors are returned as exceptions with codes and messages:

```kotlin
try {
    plugin.call("math.divide", """{"a": 10, "b": 0}""")
} catch (e: PluginException) {
    println("Error: ${e.message}")      // "Division by zero"
    println("Code: ${e.errorCode}")     // 2 (InvalidInput)
}
```

### Log Callbacks

Receive logs from the Rust plugin:

```kotlin
val logCallback = LogCallback { level, target, message ->
    println("[$level] $target: $message")
}

FfmPluginLoader.load(libraryPath, config, logCallback).use { plugin ->
    plugin.call("echo", """{"message": "test"}""")
}
```

## Variant Selection

Bundles can contain multiple variants (release, debug, etc.). By default, `release` is loaded:

```kotlin
// Load release (default)
bundleLoader.extractLibrary()

// Load specific variant
bundleLoader.extractLibrary(platform, "debug", outputDir)

// List available variants
val variants = bundleLoader.listVariants("linux-x86_64")
// ["release", "debug"]
```

## Schema Access

Bundles may include schema files for documentation or code generation:

```kotlin
// List schemas
val schemas = bundleLoader.getSchemas()
// {"messages.json": SchemaInfo(...), "messages.h": SchemaInfo(...)}

// Extract a schema
val schemaPath = bundleLoader.extractSchema("messages.json", outputDir)

// Read schema content
val content = bundleLoader.readSchema("messages.json")
```

## Type-Safe Calls

Instead of raw JSON strings, use typed wrappers for safety:

### Kotlin (Jackson)

```kotlin
data class AddRequest(val a: Long, val b: Long)
data class AddResponse(val result: Long)

val mapper = jacksonObjectMapper()

inline fun <reified T> Plugin.callTyped(tag: String, request: Any): T {
    val json = mapper.writeValueAsString(request)
    val response = call(tag, json)
    return mapper.readValue(response)
}

val result = plugin.callTyped<AddResponse>("math.add", AddRequest(10, 20))
println(result.result)  // 30
```

### C# (System.Text.Json)

```csharp
public record AddRequest(long A, long B);
public record AddResponse(long Result);

var request = new AddRequest(10, 20);
var json = JsonSerializer.Serialize(request);
var response = plugin.Call("math.add", json);
var result = JsonSerializer.Deserialize<AddResponse>(response);
Console.WriteLine(result.Result);  // 30
```

### Python (dataclasses)

```python
from dataclasses import dataclass
import json

@dataclass
class AddRequest:
    a: int
    b: int

@dataclass
class AddResponse:
    result: int

request = AddRequest(10, 20)
response_json = plugin.call("math.add", json.dumps(request.__dict__))
response = AddResponse(**json.loads(response_json))
print(response.result)  # 30
```

## Code Generation

For larger APIs, generate type-safe clients from Rust types:

```bash
# Generate Java classes
rustbridge generate java \
  --input src/messages.rs \
  --output generated/java \
  --package com.myapp.messages

# Generate JSON Schema
rustbridge generate json-schema \
  --input src/messages.rs \
  --output generated/schema.json
```

See [Code Generation Guide](../CODE_GENERATION.md) for details.

## Common Issues

### Platform Not Supported

**Symptom**: "Platform not supported: linux-x86_64"
**Cause**: Bundle doesn't include a library for your platform
**Fix**: Use `rustbridge bundle list` to check, rebuild with missing platform

### Checksum Verification Failed

**Symptom**: "Checksum verification failed"
**Cause**: Library was corrupted or modified
**Fix**: Re-download or rebuild the bundle

### Signature Verification Failed

**Symptom**: "Manifest signature verification failed"
**Cause**: Bundle was tampered with or signed with a different key
**Fix**: Verify bundle source, or use `verifySignatures(false)` for development

### Library Already Exists

**Symptom**: "Library already exists at target path"
**Cause**: Previous extraction wasn't cleaned up
**Fix**: Use `extractLibrary()` (no args) for auto temp directory, or clean up manually

## Best Practices

1. **Always close plugins** - Use try-with-resources or `use { }` blocks
2. **Enable signature verification in production** - Protects against tampering
3. **Use typed wrappers** - Catch serialization errors at compile time
4. **Configure appropriate thread counts** - Match your workload
5. **Set reasonable timeouts** - Prevent hung operations
6. **Handle errors gracefully** - Don't let plugin errors crash your app

## Next Steps

- Choose your language guide above for detailed instructions
- [Bundle Format](../BUNDLE_FORMAT.md) - Understand the `.rbp` format
- [Code Generation](../CODE_GENERATION.md) - Generate typed clients
- [Binary Transport](../TRANSPORT.md) - Faster alternative to JSON
