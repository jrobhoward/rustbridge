# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rustbridge is a framework for developing Rust shared libraries callable from other languages (Java, C#, Python, etc.). It uses C ABI under the hood but abstracts the complexity, providing OSGI-like lifecycle, mandatory async (Tokio), logging callbacks, and JSON-based data transport with optional binary transport for performance-critical paths.

## Quick Reference

```bash
# Pre-commit validation (recommended before committing)
./scripts/pre-commit.sh              # Full validation
./scripts/pre-commit.sh --fast       # Skip clippy and integration tests
./scripts/pre-commit.sh --smart      # Only test changed components

# Common development commands
cargo test -p rustbridge-ffi         # Test a specific crate
cargo test lifecycle___installed     # Run tests matching pattern
cargo clippy --workspace --examples --tests -- -D warnings
```

## Rust Version Policy

- **Edition**: Rust 2024
- **MSRV**: 1.85.0 (required for Rust 2024 edition)
- **Testing**: CI should test on MSRV, stable, and nightly

Use `cargo msrv verify` to check MSRV compatibility when adding new dependencies.

## Documentation

- [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) - Complete tutorial for creating your first plugin
- [docs/ERROR_HANDLING.md](./docs/ERROR_HANDLING.md) - Error handling best practices and patterns
- [docs/DEBUGGING.md](./docs/DEBUGGING.md) - Debugging techniques for Rust, Java, and FFI
- [docs/PLUGIN_LIFECYCLE.md](./docs/PLUGIN_LIFECYCLE.md) - Plugin lifecycle, reload patterns, and resource cleanup
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - System architecture and design decisions
- [docs/SKILLS.md](./docs/SKILLS.md) - Development best practices and coding conventions
- [docs/TESTING.md](./docs/TESTING.md) - Rust testing conventions and guidelines
- [docs/TESTING_KOTLIN.md](./docs/TESTING_KOTLIN.md) - Kotlin testing conventions and guidelines
- [docs/TESTING_JAVA.md](./docs/TESTING_JAVA.md) - Java testing conventions and guidelines
- [docs/TASKS.md](./docs/TASKS.md) - Project roadmap and task tracking
- [docs/BINARY_TRANSPORT.md](./docs/BINARY_TRANSPORT.md) - Binary transport migration guide (opt-in performance feature)
- [docs/CODE_GENERATION.md](./docs/CODE_GENERATION.md) - Code generation guide (JSON Schema, Java classes)

## Architecture Overview

The framework follows a layered architecture:

- **Core layer** (`rustbridge-core`, `rustbridge-transport`): Traits, types, serialization
- **Runtime layer** (`rustbridge-runtime`, `rustbridge-logging`): Tokio integration, tracing callbacks
- **FFI layer** (`rustbridge-ffi`): C ABI exports, buffer management
- **Tooling** (`rustbridge-macros`, `rustbridge-cli`, `rustbridge-bundle`): Code generation, build tools

Data flow: Host → FFI boundary → Async runtime → Plugin implementation → Response → FFI boundary → Host

Memory follows "Rust allocates, host frees" pattern. See [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) for details.

## Git Workflow

**The user controls git operations.** Do not commit, push, or create branches without explicit user request.

- Never auto-commit: Wait for the user to request a commit
- Never push: The user decides when and where to push
- Stage selectively: Only stage files related to the current task

## Build Commands

### Pre-commit Validation (Recommended)

```bash
./scripts/pre-commit.sh           # Full validation (Linux/macOS)
./scripts/pre-commit.sh --fast    # Skip clippy and integration tests
./scripts/pre-commit.sh --smart   # Only test changed components
```

**Note**: `--fast` skips clippy, which can catch real bugs. Use full validation before final commits.

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
rustbridge new my-plugin              # Create a new plugin project
rustbridge build --release            # Build a plugin
rustbridge check                      # Validate manifest

# Code generation
rustbridge generate json-schema -i src/messages.rs -o schema.json
rustbridge generate java -i src/messages.rs -o src/main/java -p com.example.messages
rustbridge generate-header --output include/messages.h

# Bundle operations (recommended for distribution)
# Create bundle without signing (development)
rustbridge bundle create --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so

# Create multi-platform bundle
rustbridge bundle create --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --lib darwin-aarch64:target/release/libmyplugin.dylib \
  --lib windows-x86_64:target/release/myplugin.dll

# Create bundle with code signing (production)
rustbridge bundle create --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --sign-key ~/.minisign/my-plugin.key

# Inspect and extract bundles
rustbridge bundle list my-plugin-1.0.0.rbp
rustbridge bundle extract my-plugin-1.0.0.rbp --output ./lib
```

See [docs/CODE_GENERATION.md](./docs/CODE_GENERATION.md) for detailed code generation options.

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

### Lock Safety

**Never call external code while holding a lock.** This includes logging, callbacks, and async operations. Release locks before any external calls to prevent deadlocks. See [docs/SKILLS.md](./docs/SKILLS.md) for detailed patterns.

## Testing Conventions

**Rust**: See [docs/TESTING.md](./docs/TESTING.md) for complete guidelines.

**Kotlin**: See [docs/TESTING_KOTLIN.md](./docs/TESTING_KOTLIN.md) for complete guidelines.

**Java**: See [docs/TESTING_JAVA.md](./docs/TESTING_JAVA.md) for complete guidelines.

**Unified approach across all languages:**
- Test naming: `subjectUnderTest___condition___expectedResult` (triple underscores)
- Arrange-Act-Assert pattern with blank lines (no comments)
- Tests in separate files (Rust: `module_tests.rs`, Kotlin/Java: mirrored structure in `test/`)
- Async support: `#[tokio::test]` (Rust), `runTest` (Kotlin), exceptions (Java)
- Code quality: Clippy (Rust), Ktlint (Kotlin), Checkstyle (Java)

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

## Transport Options

**JSON (default)**: Universal, debuggable, no schema required. Use for most cases.

**Binary (opt-in)**: 7x faster for latency-sensitive hot paths. Requires C struct definitions. See [docs/BINARY_TRANSPORT.md](./docs/BINARY_TRANSPORT.md).

## Schema Embedding

Bundles can include schema files (C headers, JSON schemas) for self-documenting APIs. Use `--generate-header` for automatic C header generation from `#[repr(C)]` structs, or `--schema` for manual files. See [docs/BINARY_TRANSPORT.md](./docs/BINARY_TRANSPORT.md) for details.
