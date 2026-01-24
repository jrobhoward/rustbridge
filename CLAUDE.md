# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rustbridge is a framework for developing Rust shared libraries callable from other languages (Java, C#, Python, etc.). It uses C ABI under the hood but abstracts the complexity, providing OSGI-like lifecycle, mandatory async (Tokio), logging callbacks, and JSON-based data transport with optional binary transport for performance-critical paths.

## Rust Version Policy

- **Edition**: Rust 2024
- **MSRV**: 1.85.0 (required for Rust 2024 edition)
- **Testing**: CI should test on MSRV, stable, and nightly

Use `cargo msrv verify` to check MSRV compatibility when adding new dependencies.

## Documentation

- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - System architecture and design decisions
- [docs/SKILLS.md](./docs/SKILLS.md) - Development best practices and coding conventions
- [docs/TESTING.md](./docs/TESTING.md) - Testing conventions and guidelines
- [docs/TASKS.md](./docs/TASKS.md) - Project roadmap and task tracking
- [docs/BINARY_TRANSPORT.md](./docs/BINARY_TRANSPORT.md) - Binary transport migration guide (opt-in performance feature)

## Git Workflow

**The user controls git operations.** Do not commit, push, or create branches without explicit user request.

- Never auto-commit: Wait for the user to request a commit
- Never push: The user decides when and where to push
- Stage selectively: Only stage files related to the current task

## Build Commands

### Pre-commit Validation (Recommended)

```bash
./scripts/pre-commit.sh           # Full validation (Linux/macOS)
./scripts/pre-commit.sh --fast    # Skip slower tests
```

**Windows**: Run the equivalent commands manually (see below). The bash script requires WSL or Git Bash.

### Manual Commands

```bash
# Format code
cargo fmt --all

# Security and license checks
cargo deny check

# Clippy (must pass with no warnings)
cargo clippy --workspace --examples --tests -- -D warnings

# Run all Rust tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p rustbridge-ffi

# Run a specific test (uses triple-underscore naming convention)
cargo test lifecycle___installed_to_starting

# Build release
cargo build --workspace --release
```

### Java/Kotlin

```bash
cd rustbridge-java
./gradlew build    # Build all subprojects
./gradlew test     # Run tests
```

### CLI Tool

```bash
# Create a new plugin project
rustbridge new my-plugin

# Build a plugin
rustbridge build --release

# Generate C header from Rust structs (for binary transport)
rustbridge generate-header --output include/messages.h
rustbridge generate-header --verify  # Verify header compiles

# Code generation from Rust message types
rustbridge generate json-schema -i src/messages.rs -o schema.json
rustbridge generate java -i src/messages.rs -o src/main/java -p com.example.messages

# Generate signing keys (one-time setup)
rustbridge keygen --output ~/.rustbridge/signing.key

# Bundle operations
rustbridge bundle create --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --lib darwin-aarch64:target/release/libmyplugin.dylib \
  --schema README.md:README.md \
  --generate-header src/binary_messages.rs:messages.h \
  --generate-schema src/messages.rs:messages.json \
  --sign-key ~/.rustbridge/signing.key

rustbridge bundle list my-plugin-1.0.0.rbp
rustbridge bundle extract my-plugin-1.0.0.rbp --output ./lib

# Validate manifest
rustbridge check
```

### Benchmarks

```bash
cargo bench -p rustbridge-transport                    # All transport benchmarks
cargo bench -p rustbridge-transport -- small_roundtrip # Specific benchmark
```

## Code Quality Requirements

### Forbidden Patterns in Production Code

The following are **not allowed** in library crates:

- `.unwrap()` - Use `?`, `.expect()` with justification, or proper error handling
- `.expect()` - Only allowed with `#[allow(clippy::expect_used)]` and a comment explaining why it's safe
- `panic!()` - Handle errors gracefully, especially in FFI code

**Exceptions**: These patterns are acceptable in test code (`#[cfg(test)]`) and examples (`examples/`).

When `.unwrap()` or `.expect()` is truly safe, annotate with:

```rust
#[allow(clippy::unwrap_used)] // Safe: regex is valid at compile time
let re = Regex::new(r"^\d+$").unwrap();
```

### Error Handling

- Use `thiserror` for error types in library crates
- Use `anyhow` only in CLI/examples
- All errors have stable numeric codes for FFI
- Never panic across FFI boundary

## Testing Conventions

See [docs/TESTING.md](./docs/TESTING.md) for complete guidelines.

**Key points:**
- Tests in separate files: `module/module_tests.rs`
- Test naming: `subject___condition___expected_result` (triple underscores)
- Arrange-Act-Assert pattern with blank lines (no comments)
- Use `#[tokio::test]` for async tests

## Crate Structure

| Crate | Responsibility |
|-------|---------------|
| `rustbridge-core` | Core traits, types, lifecycle |
| `rustbridge-transport` | Serialization, message envelopes |
| `rustbridge-ffi` | C ABI exports, buffer management |
| `rustbridge-runtime` | Async runtime, shutdown signals |
| `rustbridge-logging` | Tracing integration, FFI callbacks |
| `rustbridge-macros` | Procedural macros |
| `rustbridge-cli` | Build tool, code generation, C header verification |
| `rustbridge-bundle` | Plugin bundling (.rbp format) |

## FFI Safety

All FFI functions must:
1. Be marked `unsafe extern "C"`
2. Have thorough `# Safety` documentation
3. Validate all pointer arguments before dereferencing
4. Handle null pointers gracefully
5. Never panic across the FFI boundary

### Panic Handling at FFI Boundary

**Critical**: Panics must never unwind across FFI boundaries into host language runtimes.

- All FFI entry points (`plugin_init`, `plugin_call`, `plugin_shutdown`) are wrapped with `catch_unwind`
- Panic hook is installed during `plugin_init` to log panics via FFI callback
- Plugin state transitions to `Failed` when a panic is caught
- Error code 11 (InternalError) is returned to the host language

**Important**: Do NOT use `panic = "abort"` in profile settings - this would crash the entire host application. The default `panic = "unwind"` allows the framework to catch and handle panics gracefully.

## Java/Kotlin Integration

The `rustbridge-java/` directory contains three subprojects:
- `rustbridge-core`: Core Java interfaces (Plugin, PluginConfig, etc.)
- `rustbridge-ffm`: FFM-based implementation (Java 21+, recommended)
- `rustbridge-jni`: JNI-based implementation (Java 8+, legacy fallback)

Memory follows "Rust allocates, host frees" pattern:
1. `plugin_call()` returns `FfiBuffer` with Rust-allocated data
2. Host copies data to managed heap
3. Host calls `plugin_free_buffer()` to release native memory

## Transport Options

**JSON (default)**: Universal, debuggable, no schema required. Use for most cases.

**Binary (opt-in)**: 7x faster for latency-sensitive hot paths. Requires C struct definitions. See [docs/BINARY_TRANSPORT.md](./docs/BINARY_TRANSPORT.md).

## Schema Embedding

Bundles can include schema files for self-documenting APIs:

### Automatic C Header Generation

Generate and embed C headers automatically during bundle creation:

```bash
rustbridge bundle create --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --generate-header src/binary_messages.rs:messages.h
```

This:
1. Generates a C header from Rust `#[repr(C)]` structs
2. Embeds it in `schema/messages.h` within the bundle
3. Adds metadata to manifest with checksum and format

### Manual Schema Files

Add arbitrary schema files:

```bash
rustbridge bundle create --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --schema api-schema.json:api-schema.json \
  --schema README.md:README.md
```

Supported formats (auto-detected):
- `*.h`, `*.hpp` → `c-header`
- `*.json` → `json-schema`
- Others → `unknown`

### Accessing Schemas in Java

```java
BundleLoader loader = BundleLoader.builder()
    .bundlePath("my-plugin-1.0.0.rbp")
    .build();

// List all schemas
Map<String, SchemaInfo> schemas = loader.getSchemas();

// Read schema content
String header = loader.readSchema("messages.h");

// Extract to file
Path schemaPath = loader.extractSchema("messages.h", Paths.get("./include"));
```

### Bundle Manifest Schema Catalog

Schemas are cataloged in the manifest:

```json
{
  "schemas": {
    "messages.h": {
      "path": "schema/messages.h",
      "format": "c-header",
      "checksum": "sha256:abc123...",
      "description": "C struct definitions for binary transport"
    }
  }
}
```
