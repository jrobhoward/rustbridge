# rustbridge Documentation

This directory contains the technical documentation for rustbridge.

## Getting Started

| Document | Description |
|----------|-------------|
| [GETTING_STARTED.md](./GETTING_STARTED.md) | Quick start guide for new users |
| [INSTALL.md](./INSTALL.md) | Installation instructions |
| [creating-plugins/](./creating-plugins/README.md) | Guide to creating your first plugin |
| [using-plugins/](./using-plugins/README.md) | Loading and using plugins from host languages |

## Architecture & Design

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System architecture, crate structure, design decisions |
| [FFI.md](./FFI.md) | C ABI interface specification |
| [MEMORY_MODEL.md](./MEMORY_MODEL.md) | Memory ownership patterns across FFI |
| [PLUGIN_LIFECYCLE.md](./PLUGIN_LIFECYCLE.md) | Plugin lifecycle states and transitions |
| [TRANSPORT.md](./TRANSPORT.md) | JSON and binary transport layer |
| [BUNDLE_FORMAT.md](./BUNDLE_FORMAT.md) | `.rbp` bundle format specification |

## Development

| Document | Description |
|----------|-------------|
| [SKILLS.md](./SKILLS.md) | Best practices and coding conventions |
| [ERROR_HANDLING.md](./ERROR_HANDLING.md) | Error types, codes, and handling patterns |
| [DEBUGGING.md](./DEBUGGING.md) | Debugging techniques across FFI |
| [BENCHMARK_RESULTS.md](./BENCHMARK_RESULTS.md) | Performance benchmarks |

## Testing

| Document | Description |
|----------|-------------|
| [TESTING.md](./TESTING.md) | Cross-language testing conventions |
| [TESTING_JAVA.md](./TESTING_JAVA.md) | Java testing conventions |
| [TESTING_KOTLIN.md](./TESTING_KOTLIN.md) | Kotlin testing conventions |
| [TESTING_CSHARP.md](./TESTING_CSHARP.md) | C# testing conventions |
| [TESTING_PYTHON.md](./TESTING_PYTHON.md) | Python testing conventions |

## Language-Specific Guides

### Using Plugins

| Language | Guide | Requirements |
|----------|-------|--------------|
| Java | [JAVA_FFM.md](./using-plugins/JAVA_FFM.md) | Java 21+ |
| Kotlin | [KOTLIN.md](./using-plugins/KOTLIN.md) | Java 21+, Kotlin 2.0+ |
| C# | [CSHARP.md](./using-plugins/CSHARP.md) | .NET 8.0+ |
| Python | [PYTHON.md](./using-plugins/PYTHON.md) | Python 3.10+ |

## Tutorials

Step-by-step tutorials for common tasks:

| Tutorial | Description |
|----------|-------------|
| [01-regex-plugin](./tutorials/01-regex-plugin/README.md) | Build a regex plugin with LRU caching |
| [02-kotlin-consumer](./tutorials/02-kotlin-consumer/README.md) | Consume plugins from Kotlin |
| [03-json-plugin](./tutorials/03-json-plugin/README.md) | JSON validation and formatting plugin |
| [04-java-consumer](./tutorials/04-java-consumer/README.md) | Consume plugins from Java |
| [05-production-bundles](./tutorials/05-production-bundles/README.md) | Code signing, schemas, SBOMs |
| [06-cross-compilation](./tutorials/06-cross-compilation/README.md) | Cross-platform builds |
| [07-backpressure-queues](./tutorials/07-backpressure-queues/README.md) | Handling concurrency limits |
| [08-binary-transport](./tutorials/08-binary-transport/README.md) | High-performance binary transport |

## Packaging & Distribution

| Document | Description |
|----------|-------------|
| [packaging/](./packaging/README.md) | Multi-platform bundles, signing, CI/CD |

## Version Requirements

| Component | Minimum Version |
|-----------|----------------|
| Rust | 1.90.0 (Edition 2024) |
| Java | 21+ (22+ recommended for FFM) |
| .NET | 8.0+ |
| Python | 3.10+ |
