# Kotlin Examples for rustbridge

This directory contains examples showing how to use rustbridge from Kotlin.

## Prerequisites

- **Java 21+** (for FFM implementation)
- **Kotlin 1.9+**
- Built `hello-plugin` shared library

## Building hello-plugin

```bash
cd ../hello-plugin
cargo build --release

# The library will be at:
# target/release/libhello_plugin.so (Linux)
# target/release/libhello_plugin.dylib (macOS)
# target/release/hello_plugin.dll (Windows)
```

## Running Examples

First, make sure you've built the hello-plugin and published the Java libraries:

```bash
# Build hello-plugin
cd ../hello-plugin
cargo build --release

# Build and publish Java libraries
cd ../../rustbridge-java
./gradlew publishToMavenLocal

# Return to kotlin-examples
cd ../examples/kotlin-examples
```

Then run the examples:

```bash
# Build the examples
./gradlew build

# Run specific example
./gradlew runBasic
./gradlew runLogging
./gradlew runErrorHandling
```

## Examples

| File | Description |
|------|-------------|
| `BasicExample.kt` | Simple usage with `use` blocks, data classes |
| `LoggingExample.kt` | Custom log callback integration |
| `ErrorHandlingExample.kt` | Handling errors gracefully |

## Kotlin Features Demonstrated

- **`use` blocks** - Automatic resource management
- **Data classes** - Concise JSON serialization
- **Extension functions** - Type-safe API helpers
- **String templates** - Clean JSON construction
- **When expressions** - Pattern matching for responses
- **Sealed classes** - Type-safe error handling

## Notes

- These examples use the **Java API directly** - no special Kotlin wrapper needed
- Kotlin's Java interop makes the API feel native
- For production use, consider adding coroutine wrappers for async operations
