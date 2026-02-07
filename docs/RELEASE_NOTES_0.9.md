# rustbridge 0.9 — Seeking Feedback Before 1.0

rustbridge is a framework for building and bundling Rust shared libraries callable from Java, Kotlin, C#, Python, or Rust. It handles the hard parts of cross-language interop — lifecycle management, async execution, memory safety, logging, and portable distribution — so you can focus on your plugin logic.

The 0.9 series represents the feature-complete milestone. The core APIs, transport formats, bundle format, and language bindings are all implemented and tested. This release is about stabilization: we're looking for feedback on the design before committing to a stable 1.0.

## What It Does

Build a Rust plugin and call it from five host languages with no glue code:

```
rustbridge new my-plugin --all         # Scaffold plugin + consumers
cargo build --release                  # Build once
rustbridge pack --no-sign              # Package as portable .rbp bundle
```

Each host language gets a native-feeling API backed by the same Rust binary:

| Language | API Style | Minimum Version |
|----------|-----------|-----------------|
| **Java** | FFM (Foreign Function & Memory) | Java 21+ |
| **Kotlin** | Coroutine-friendly DSL | Java 21+ |
| **C#** | P/Invoke with `IDisposable` | .NET 8.0+ |
| **Python** | ctypes with context managers | Python 3.10+ |
| **Rust** | `rustbridge-consumer` crate | Rust 1.90+ |

Under the hood, rustbridge uses a C ABI with an OSGI-inspired plugin lifecycle (Installed, Starting, Active, Stopping, Stopped, Failed) and a mandatory Tokio async runtime. Rust `tracing` logs are forwarded to host loggers via FFI callbacks. Plugins are distributed as portable `.rbp` bundles with optional Ed25519 signing.

## What's New Since 0.5

### Facade Crate (0.6.0)

Plugin authors now add a single `rustbridge` dependency instead of juggling `rustbridge-core`, `rustbridge-ffi`, `rustbridge-macros`, etc.

### Simplified Java Integration (0.8.0)

Java now uses the Foreign Function & Memory API (FFM) exclusively, requiring Java 21+. This eliminated the JNI bridge layer, reducing complexity and improving binary transport performance.

### Rust Consumer Support (0.8.1)

Rust applications can load and call rustbridge plugins via the `rustbridge-consumer` crate, with bundle loading, signature verification, and full lifecycle management.

### `pack` and `promote` Workflow (0.9)

`rustbridge pack` reads `Cargo.toml` to auto-detect plugin name, version, platform, and library path — replacing the verbose `bundle create` for everyday use. `rustbridge promote` converts dev bundles to signed release bundles, supporting a clean dev/release pipeline.

### Tutorials

Eight step-by-step tutorials cover a wide range of rustbridge use cases, from building a basic regex plugin with LRU caching through to binary transport for high-performance paths.

## What We Believe Is Stable

The following areas have no more planned breaking changes. We're looking for confirmation that nothing has been overlooked:

- **JSON transport** — Request/response envelope format, error codes, type tags
- **Binary transport** — `#[repr(C)]` struct marshaling, `RbString`/`RbBytes` types
- **C ABI surface** — `plugin_init`, `plugin_call`, `plugin_shutdown`, `plugin_free_buffer`, `plugin_get_state`, `plugin_set_log_level`, `plugin_get_rejected_count`
- **Language bindings** — Java/Kotlin (FFM), C# (P/Invoke), Python (ctypes), Rust consumer
- **`.rbp` bundle format** — ZIP-based archive with manifest, platform libraries, optional schemas and signatures
- **Manifest format** — `manifest.json` schema including `bundle_version`, `plugin`, `platforms`, `build_info`
- **Ed25519 signing** — minisign-compatible key generation, signing, and verification
- **CLI subcommands** — `new`, `pack`, `promote`, `bundle`, `keygen`, `generate-header`

## The Road to 1.0

### What Remains

- Documentation and tutorial improvements
- Test coverage improvements
- Fix any critical bugs that are identified

### What We Need Help With

The most important question before 1.0 is whether we've missed anything that would require a breaking change — a new file/manifest format field, a change to the C ABI function signatures, a missing error code, an oversight in the memory model.

We *think* the design is solid, but additional eyes would be valuable. Areas where review would be particularly useful:

- **C ABI surface**: Are `plugin_init()`, `plugin_call()`, `plugin_shutdown()`, `plugin_free_buffer()`, and the `FfiBuffer` struct sufficient? Are there operations that should be exposed but aren't?
- **Error codes**: The 14 stable error codes (0-13) cover the cases we've encountered. Are there failure modes we haven't considered?
- **Bundle format**: The `.rbp` manifest covers platforms, variants, checksums, signing, build provenance, and SBOMs. Is anything missing for real-world distribution?
- **Transport protocol**: JSON envelopes with `type_tag`/`payload`/`request_id` plus optional binary transport. Does this cover your use cases?

If you spot a design issue that would require breaking changes, now is the time to raise it — before 1.0 locks the API.

## Getting Started

```bash
cargo install rustbridge-cli
rustbridge new my-plugin --all
cd my-plugin
cargo build --release
rustbridge pack --no-sign
```

See the [Getting Started Guide](./GETTING_STARTED.md) for the full walkthrough, or dive into the [tutorials](./tutorials/README.md) for hands-on examples.

## Contributing

Contributions are welcome. If you're interested in helping improve documentation, test coverage, language bindings, or have feedback on the design, please open an issue or PR on [GitHub](https://github.com/jrobhoward/rustbridge).

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](./GETTING_STARTED.md) | Build, package, and run your first plugin |
| [CLI Reference](./CLI.md) | All commands and options |
| [Architecture](./ARCHITECTURE.md) | Crate structure, data flow, design decisions |
| [Bundle Format](./BUNDLE_FORMAT.md) | `.rbp` archive specification |
| [Transport](./TRANSPORT.md) | JSON and binary transport |
| [Error Handling](./ERROR_HANDLING.md) | Error types, codes, and patterns for all languages |
| [Packaging](./packaging/README.md) | Multi-platform bundles, signing, CI/CD |
| [Tutorials](./tutorials/README.md) | Eight step-by-step tutorials |

## License

MIT OR Apache-2.0
