# rustbridge 1.0 Release Notes

rustbridge is a framework for building Rust shared libraries callable from Java, Kotlin, C#, Python, and Rust. It handles the hard parts of cross-language interop — lifecycle management, async execution, memory safety, logging, and portable distribution — so you can focus on your plugin logic.

## Highlights

### Write Once, Call from Anywhere

Build a Rust plugin and call it from five host languages with no glue code:

```
rustbridge new my-plugin --all        # Scaffold plugin + consumers
cargo build --release                  # Build once
rustbridge pack --no-sign              # Package as portable .rbp
```

Each host language gets a native-feeling API backed by the same Rust binary:

| Language | API | Requirements |
|----------|-----|--------------|
| **Java** | FFM (Foreign Function & Memory) | Java 21+ |
| **Kotlin** | Coroutine-friendly DSL | Java 21+ |
| **C#** | P/Invoke with `IDisposable` | .NET 8.0+ |
| **Python** | ctypes with context managers | Python 3.10+ |
| **Rust** | `rustbridge-consumer` crate | Rust 1.90+ |

### OSGI-Inspired Plugin Lifecycle

Plugins follow a state machine (Installed, Starting, Active, Stopping, Stopped, Failed) with well-defined transitions. The framework manages Tokio runtime initialization, graceful shutdown with configurable timeouts, and resource cleanup — your plugin just implements `on_start`, `handle_request`, and `on_stop`.

### Safe by Default

- **No panics across FFI**: `catch_unwind` guards at every boundary
- **No memory leaks**: "Rust allocates, host frees" pattern with RAII cleanup
- **No deadlocks**: `await_holding_lock = "deny"` enforced at compile time
- **No unbounded concurrency**: Configurable `max_concurrent_ops` with fail-fast backpressure

### Portable Bundle Format (.rbp)

The `.rbp` format packages multi-platform native libraries into a single distributable ZIP:

- **Multi-platform**: Linux, macOS, Windows (x86_64 and ARM64) in one file
- **Multi-variant**: Ship release and debug builds side by side
- **Code signing**: Ed25519 signatures via minisign for authenticity verification
- **Build provenance**: Git commit, compiler version, timestamps, custom metadata
- **SBOM support**: CycloneDX and SPDX for dependency transparency
- **Schema embedding**: JSON Schema and C headers for consumer code generation

### Two Transport Modes

- **JSON transport**: Zero-config, works everywhere, automatic serialization with serde
- **Binary transport**: `#[repr(C)]` structs for latency-sensitive paths (7x faster than JSON)

### Developer-Friendly CLI

The `rustbridge` CLI handles project scaffolding and packaging:

| Command | Purpose |
|---------|---------|
| `rustbridge new` | Scaffold plugin + consumer projects for any combination of languages |
| `rustbridge pack` | Auto-detect and bundle from `Cargo.toml` in one step |
| `rustbridge promote` | Convert dev bundles to signed release bundles |
| `rustbridge bundle combine` | Merge per-platform CI bundles into one multi-platform bundle |
| `rustbridge keygen` | Generate Ed25519 signing keys |
| `rustbridge generate-header` | Generate C headers from Rust `#[repr(C)]` structs |

## What's New Since 0.5

### Rust Consumer Support (0.8.1)

Rust applications can now load and call rustbridge plugins via the `rustbridge-consumer` crate, with bundle loading, signature verification, and full lifecycle management.

### Simplified Java Integration (0.8.0)

Java now uses the Foreign Function & Memory API (FFM) exclusively, requiring Java 21+. This eliminated the JNI bridge layer, reducing complexity and improving binary transport performance.

### pack and promote Workflow (Unreleased)

`rustbridge pack` reads `Cargo.toml` to auto-detect plugin name, version, platform, and library path — replacing the verbose `bundle create` for everyday use. `rustbridge promote` converts dev bundles to signed release bundles, supporting a clean dev/release pipeline.

### Facade Crate (0.6.0)

Plugin authors now add a single `rustbridge` dependency instead of juggling `rustbridge-core`, `rustbridge-ffi`, `rustbridge-macros`, etc. Use `rustbridge::prelude::*` and `rustbridge::ffi_exports::*`.

### Tutorials

Eight step-by-step tutorials cover the full range of rustbridge usage:

1. Building a regex plugin with LRU caching
2. Consuming from Kotlin with type-safe wrappers
3. JSON validation and formatting plugin
4. Consuming from Java with error handling
5. Production bundles: signing, schemas, SBOMs
6. Cross-compilation for multi-platform distribution
7. Backpressure queues and concurrency limits
8. Binary transport for high-performance paths

## Getting Started

```bash
# Install the CLI
cargo install rustbridge-cli

# Create a plugin with consumers for all languages
rustbridge new my-plugin --all
cd my-plugin

# Build and package
cargo build --release
rustbridge pack --no-sign

# Run from Kotlin
cd consumers/kotlin
cp ../../target/bundle/my-plugin-*.rbp .
./gradlew run
```

See the [Getting Started Guide](./GETTING_STARTED.md) for the full walkthrough, or dive into the [tutorials](./tutorials/README.md) for hands-on examples.

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](./GETTING_STARTED.md) | Build, package, and run your first plugin |
| [CLI Reference](./CLI.md) | All commands and options |
| [Architecture](./ARCHITECTURE.md) | Crate structure, data flow, design decisions |
| [Bundle Format](./BUNDLE_FORMAT.md) | `.rbp` archive specification |
| [Error Handling](./ERROR_HANDLING.md) | Error types, codes, and patterns for all languages |
| [Packaging](./packaging/README.md) | Multi-platform bundles, signing, CI/CD |
| [Tutorials](./tutorials/README.md) | Eight step-by-step tutorials |

## Version Requirements

| Component | Minimum Version |
|-----------|----------------|
| Rust | 1.90.0 (Edition 2024) |
| Java | 21+ (22+ recommended) |
| .NET | 8.0+ |
| Python | 3.10+ |

## License

MIT OR Apache-2.0
