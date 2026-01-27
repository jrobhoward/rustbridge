# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rustbridge is a framework for developing Rust shared libraries callable from other languages (Java, C#, Python, etc.). It uses C ABI under the hood but abstracts the complexity, providing OSGI-like lifecycle, mandatory async (Tokio), logging callbacks, and JSON-based data transport with optional binary transport for performance-critical paths.

## Quick Reference

```bash
# Pre-commit validation (run before committing)
./scripts/pre-commit.sh              # Full validation (Linux/macOS)
./scripts/pre-commit.sh --fast       # Skip clippy and integration tests
./scripts/pre-commit.sh --smart      # Only test changed components
scripts\pre-commit.bat               # Windows (full validation)
scripts\pre-commit.bat --fast        # Windows (skip clippy/integration)

# Common development commands
cargo fmt --all                                                    # Format code
cargo clippy --workspace --examples --tests -- -D warnings         # Lint (must pass)
cargo test --workspace                                             # Test all crates
cargo test -p rustbridge-ffi                                       # Test specific crate
cargo test lifecycle___installed                                   # Run tests matching pattern
cargo bench -p rustbridge-transport -- small_roundtrip             # Run specific benchmark

# Java/Kotlin (from rustbridge-java/)
./gradlew build && ./gradlew test    # Linux/macOS
gradlew.bat build && gradlew.bat test  # Windows
```

## Rust Version

- **Edition**: Rust 2024 | **MSRV**: 1.85.0
- Use `cargo msrv verify` when adding dependencies

## Architecture Overview

```
Host Language → FFI Boundary → Async Runtime → Plugin Implementation → Response → FFI → Host
```

**Layered crate structure:**
- **Core** (`rustbridge-core`, `rustbridge-transport`): Traits, types, serialization
- **Runtime** (`rustbridge-runtime`, `rustbridge-logging`): Tokio integration, tracing callbacks
- **FFI** (`rustbridge-ffi`, `rustbridge-jni`): C ABI exports, buffer management, JNI bridge for Java 17+
- **Tooling** (`rustbridge-macros`, `rustbridge-cli`, `rustbridge-bundle`): Code generation, build tools, `.rbp` packaging

Memory follows "Rust allocates, host frees" pattern. See [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) for details.

## Code Quality Requirements

### Forbidden in Production Code

- `.unwrap()` / `.expect()` - Use `?` or proper error handling. If truly safe, annotate with `#[allow(clippy::unwrap_used)]` and comment.
- `panic!()` - Handle errors gracefully, especially in FFI code

**Exceptions**: Allowed in test code (`#[cfg(test)]`) and examples.

### Error Handling

- Use `thiserror` for library crates, `anyhow` only in CLI/examples
- All errors have stable numeric codes for FFI
- Never panic across FFI boundary

### Lock Safety

**Never call external code while holding a lock.** This includes logging, callbacks, and async operations. Release locks before any external calls. See [docs/SKILLS.md](./docs/SKILLS.md) for patterns.

## Testing Conventions

See [docs/TESTING.md](./docs/TESTING.md), [docs/TESTING_KOTLIN.md](./docs/TESTING_KOTLIN.md), [docs/TESTING_JAVA.md](./docs/TESTING_JAVA.md), [docs/TESTING_CSHARP.md](./docs/TESTING_CSHARP.md).

**Key conventions:**
- Test naming: `subjectUnderTest___condition___expectedResult` (triple underscores)
- Arrange-Act-Assert pattern with blank lines (no comments)
- Tests in separate files (Rust: `module_tests.rs`)
- Async: `#[tokio::test]` (Rust), `runTest` (Kotlin)

## FFI Safety

All FFI functions must:
1. Be marked `unsafe extern "C"` with thorough `# Safety` docs
2. Validate all pointers before dereferencing
3. Handle null pointers gracefully
4. Never panic across the FFI boundary (use `catch_unwind`)

**Do NOT use `panic = "abort"`** - this crashes the host application. The default `panic = "unwind"` allows graceful panic handling.

## Java/Kotlin Integration

The `rustbridge-java/` directory contains:
- `rustbridge-core`: Core Java interfaces
- `rustbridge-ffm`: FFM implementation (Java 21+, recommended)
- `rustbridge-jni`: JNI implementation (Java 17+, fallback)

## C# Integration

The `rustbridge-csharp/` directory contains:
- `RustBridge.Core`: Core interfaces and types (IPlugin, PluginConfig)
- `RustBridge.Native`: P/Invoke-based native plugin loader
- `RustBridge.Tests`: Unit and integration tests

```bash
# C# development (from rustbridge-csharp/)
dotnet build                                        # Build all projects
dotnet test                                         # Run all tests
dotnet test --filter "FullyQualifiedName~FromCode"  # Run tests matching pattern
```

See [docs/TESTING_CSHARP.md](./docs/TESTING_CSHARP.md) for C# testing conventions.

## Python Integration

The `rustbridge-python/` directory contains:
- `rustbridge.core`: Core types (LogLevel, LifecycleState, PluginConfig, etc.)
- `rustbridge.native`: ctypes-based native plugin loader
- `rustbridge.core.bundle_loader`: Bundle loading with minisign signature verification

```bash
# Python development (from rustbridge-python/)
python -m venv .venv && source .venv/bin/activate  # Create virtual environment
pip install -e ".[dev]"                             # Install with dev dependencies
python -m pytest tests/ -v                          # Run all tests
python -m pytest tests/test_log_level.py -v         # Run tests matching pattern
```

See [docs/TESTING_PYTHON.md](./docs/TESTING_PYTHON.md) for Python testing conventions.

## Documentation

- [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) - Tutorial for creating your first plugin
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - System architecture and design decisions
- [docs/BUNDLE_FORMAT.md](./docs/BUNDLE_FORMAT.md) - .rbp bundle specification
- [docs/TRANSPORT.md](./docs/TRANSPORT.md) - JSON and binary transport (binary is 7x faster)
- [docs/MEMORY_MODEL.md](./docs/MEMORY_MODEL.md) - Memory ownership patterns
- [docs/SKILLS.md](./docs/SKILLS.md) - Development best practices
- [docs/PLUGIN_LIFECYCLE.md](./docs/PLUGIN_LIFECYCLE.md) - Plugin lifecycle and resource cleanup
- [docs/CODE_GENERATION.md](./docs/CODE_GENERATION.md) - Code generation guide
