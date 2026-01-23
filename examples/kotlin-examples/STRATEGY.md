# Kotlin Support Strategy

## Implementation Approach

**Strategy**: Single Java implementation + Idiomatic Kotlin examples

We chose **not** to create a separate Kotlin implementation. Instead, we leverage Kotlin's excellent Java interoperability and provide idiomatic examples showing best practices.

## Rationale

### Why Not a Kotlin Implementation?

1. **Java interop is excellent**: Kotlin can use Java APIs seamlessly
2. **Single codebase**: Less maintenance burden
3. **Compatibility**: Java 8+ users don't need Kotlin runtime
4. **Simplicity**: No need to maintain parallel implementations

### Why Kotlin Examples?

1. **Show best practices**: Demonstrate idiomatic Kotlin usage
2. **Highlight benefits**: Show how Kotlin makes the API better
3. **Lower barrier**: Help Kotlin developers get started quickly
4. **Real-world patterns**: Common use cases and error handling

## What We Provide

### For Java Users
- Pure Java API (no Kotlin dependencies)
- Works with Java 8+ (JNI) or Java 21+ (FFM)
- Traditional try-catch-finally patterns
- Builder pattern for configuration

### For Kotlin Users
- **Same Java API** (no wrapper needed)
- Examples showing idiomatic usage:
  - Data classes for request/response
  - Extension functions for type safety
  - `use` blocks for resource management
  - Sealed classes for error handling
  - Collection operators for batch operations
- Comparison guide (Java vs Kotlin)
- Quick start guide

## User Experience

### Java Developer Experience
```java
// Traditional Java code
PluginConfig config = PluginConfig.builder()
    .logLevel("info")
    .build();

Plugin plugin = null;
try {
    plugin = FfmPluginLoader.load("libmyplugin.so", config);
    String response = plugin.call("echo", requestJson);
    // Process response...
} finally {
    if (plugin != null) {
        plugin.close();
    }
}
```

### Kotlin Developer Experience
```kotlin
// Idiomatic Kotlin - same API, cleaner code
val config = PluginConfig.builder()
    .logLevel("info")
    .build()

FfmPluginLoader.load("libmyplugin.so", config).use { plugin ->
    val response = plugin.callTyped<EchoResponse>("echo", request)
    // Automatic cleanup!
}
```

## Future Enhancements (Optional)

If demand warrants, we could add:

### Option 1: Kotlin Extension Module
A lightweight `rustbridge-kotlin` module providing:
- Extension functions
- Coroutine integration
- DSL builders
- Type-safe wrappers

**Pros**: More idiomatic, better IDE support
**Cons**: More code to maintain, Kotlin dependency

### Option 2: Code Generation for Kotlin
Generate Kotlin code from `rustbridge.toml`:
- Data classes from JSON schemas
- Sealed classes for message types
- Suspend functions for async

**Pros**: Type-safe, zero-cost
**Cons**: Tooling complexity

### Option 3: Coroutine Wrapper
Suspend function wrapper for async operations:
```kotlin
suspend fun Plugin.callAsync<T>(messageType: String, request: Any): T
```

**Pros**: Natural Kotlin async
**Cons**: Requires async FFI API first

## Current Recommendation

**Start simple**: Use the Java API from Kotlin with the provided examples.

**If you need more**:
1. Copy extension functions from examples into your project
2. Add your own DSL helpers as needed
3. Consider proposing them for inclusion if they're generally useful

## Comparison with Other Approaches

### Alternative: Kotlin-First Implementation
Some projects (like Exposed, Ktor) are Kotlin-first.

**When it makes sense**:
- Coroutines are fundamental
- DSL is the primary API
- JVM is the only target

**Why we didn't**:
- FFI boundary is language-agnostic (C ABI)
- Java users are significant audience
- Kotlin can use Java APIs naturally

### Alternative: Multi-Language Wrapper
Some projects provide separate wrappers per language.

**When it makes sense**:
- Very complex API surface
- Language-specific features (Python context managers, Rust traits)
- Performance-critical paths

**Why we didn't**:
- Our API is simple (just `call()`)
- Kotlin's extension functions work great
- Maintenance burden not justified

## Key Takeaway

**Kotlin support ✓**
**Kotlin wrapper ✗**

We support Kotlin by:
1. Ensuring the Java API is Kotlin-friendly (uses AutoCloseable, etc.)
2. Providing comprehensive examples
3. Documenting idiomatic patterns
4. Showing Java vs Kotlin comparisons

This gives Kotlin users a great experience without fragmenting the codebase.
