# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rustbridge is a framework for developing Rust shared libraries callable from other languages (Java, C#, Python, etc.). It uses C ABI under the hood but abstracts the complexity.

## Rust Version Policy

- **Edition**: Rust 2024
- **MSRV**: 1.85.0 (required for Rust 2024 edition)
- **Testing**: CI should test on MSRV, stable, and nightly
- **MSRV Update Policy**: May increase MSRV to Rust versions released ≥6 months ago (following Tokio's conservative approach)
- **Pre-1.0 Notice**: MSRV policy may be adjusted before 1.0 release based on ecosystem needs

### Dependency Versions

Key dependencies follow these version ranges:
- **tokio**: 1.43+ (LTS release with 1.70 MSRV)
- **serde**: 1.0+ (semver compatible)
- **clap**: 4.5+ (latest 4.x)
- **uuid**: 1.11+ (latest 1.x)

Use `cargo msrv verify` to check MSRV compatibility when adding new dependencies.

## Documentation

- [docs/SKILLS.md](./docs/SKILLS.md) - Development best practices and coding conventions
- [docs/TESTING.md](./docs/TESTING.md) - Testing conventions and guidelines
- [docs/TASKS.md](./docs/TASKS.md) - Project roadmap and task tracking
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - System architecture and design decisions

## Git Workflow

**The user controls git operations.** Do not commit, push, or create branches without explicit user request.

- Never auto-commit: Wait for the user to request a commit
- Never push: The user decides when and where to push
- Stage selectively: Only stage files related to the current task

## Code Quality Requirements

### Formatting

All code must be formatted with `rustfmt` before committing:

```bash
cargo fmt --all
```

Check formatting without modifying files:

```bash
cargo fmt --all -- --check
```

### Clippy

All code must pass clippy with no warnings:

```bash
cargo clippy --workspace --examples --tests -- -D warnings
```

### Pre-commit Checklist

Before committing code, run:

```bash
# Format code
cargo fmt --all

# Check for warnings
cargo clippy --workspace --examples --tests -- -D warnings

# Run tests
cargo test --workspace
```

### Forbidden Patterns in Production Code

The following are **not allowed** in production code (library crates):

- `.unwrap()` - Use `?`, `.expect()` with justification, or proper error handling
- `.expect()` - Only allowed with `#[allow(clippy::expect_used)]` and a comment explaining why it's safe
- `panic!()` - Handle errors gracefully, especially in FFI code

**Exception**: These patterns are acceptable in:
- Test code (`#[cfg(test)]` modules)
- Examples (`examples/` directory)
- Cases where failure is impossible (add `#[allow(...)]` with explanation)

### Clippy Lints

The workspace enables strict lints. When `.unwrap()` or `.expect()` is truly safe (e.g., compile-time known values), annotate with:

```rust
#[allow(clippy::unwrap_used)] // Safe: regex is valid at compile time
let re = Regex::new(r"^\d+$").unwrap();
```

## Testing Conventions

See [docs/TESTING.md](./docs/TESTING.md) for complete testing guidelines.

### Key Points

- Tests in separate files: `module/module_tests.rs`
- Test naming: `subject___condition___expected_result` (triple underscores)
- Arrange-Act-Assert pattern with blank lines (no comments)
- Run tests: `cargo test --workspace`
- Run clippy on tests: `cargo clippy --workspace --tests`

## Error Handling

- Use `thiserror` for error types in library crates
- Use `anyhow` only in CLI/examples
- All errors have stable numeric codes for FFI
- Never panic across FFI boundary

## Crate Structure

| Crate | Responsibility |
|-------|---------------|
| `rustbridge-core` | Core traits, types, lifecycle |
| `rustbridge-transport` | Serialization, message envelopes |
| `rustbridge-ffi` | C ABI exports, buffer management |
| `rustbridge-runtime` | Async runtime, shutdown signals |
| `rustbridge-logging` | Tracing integration, FFI callbacks |
| `rustbridge-macros` | Procedural macros |
| `rustbridge-cli` | Build tool, code generation |

## FFI Safety

All FFI functions must:
1. Be marked `unsafe extern "C"`
2. Have thorough `# Safety` documentation
3. Validate all pointer arguments before dereferencing
4. Handle null pointers gracefully
5. Never panic across the FFI boundary

### Panic Handling at FFI Boundary

**Critical**: Panics must never unwind across FFI boundaries into host language runtimes (Java/C#/Python).

The framework implements comprehensive panic handling:

1. **All FFI entry points** (`plugin_init`, `plugin_call`, `plugin_shutdown`) are wrapped with `catch_unwind`
2. **Panic hook** is installed during `plugin_init` to log panics via FFI callback
3. **Plugin state** transitions to `Failed` when a panic is caught
4. **Error code 11** (InternalError) is returned to the host language
5. **Panic = "unwind"** is used (NOT "abort") to allow panic recovery

**Important**: Do NOT use `panic = "abort"` in profile settings - this would crash the entire host application. The default `panic = "unwind"` allows the framework to catch and handle panics gracefully.

## Build Commands

### Rust

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run clippy (must pass with no warnings)
cargo clippy --workspace --examples --tests -- -D warnings

# Build release
cargo build --workspace --release

# Build a specific crate
cargo build -p rustbridge-core

# Run tests for a specific crate
cargo test -p rustbridge-ffi

# Run a specific test
cargo test lifecycle___installed_to_starting
```

### Java/Kotlin (optional)

The `rustbridge-java/` directory contains Java/Kotlin bindings with three subprojects:
- `rustbridge-core`: Core Java interfaces (Plugin, PluginConfig, etc.)
- `rustbridge-ffm`: FFM-based implementation (Java 21+, recommended)
- `rustbridge-jni`: JNI-based implementation (Java 8+, legacy fallback)

```bash
# Build Java bindings (from rustbridge-java/ directory)
cd rustbridge-java
./gradlew build

# Run Java tests
./gradlew test

# Build a specific subproject
./gradlew :rustbridge-ffm:build
```
