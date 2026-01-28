# Contributing to rustbridge

Thank you for your interest in contributing to rustbridge! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/rustbridge.git`
3. Create a branch: `git checkout -b my-feature`
4. Make your changes
5. Run validation: `./scripts/pre-commit.sh`
6. Submit a pull request

## Development Environment

### Prerequisites

- **Rust**: Edition 2024, MSRV 1.90.0
- **Java**: JDK 21+ (for FFM), JDK 17+ (for JNI)
- **C#**: .NET 8.0+
- **Python**: 3.10+

### Building

```bash
# Build all Rust crates
cargo build --workspace

# Build Java/Kotlin
cd rustbridge-java && ./gradlew build

# Build C#
cd rustbridge-csharp && dotnet build

# Build Python (install in development mode)
cd rustbridge-python && pip install -e ".[dev]"
```

### Running Tests

```bash
# Full validation (recommended before submitting PR)
./scripts/pre-commit.sh

# Quick validation (skip clippy and integration tests)
./scripts/pre-commit.sh --fast

# Rust only
cargo test --workspace

# Java/Kotlin only
cd rustbridge-java && ./gradlew test

# C# only
cd rustbridge-csharp && dotnet test

# Python only
cd rustbridge-python && python -m pytest tests/ -v
```

## Code Quality Standards

### Rust

#### Forbidden in Production Code

- **`.unwrap()` / `.expect()`**: Use `?` or proper error handling. If truly safe, add `#[allow(clippy::unwrap_used)]` with a comment explaining why.
- **`panic!()`**: Handle errors gracefully, especially in FFI code.

These are allowed in test code (`#[cfg(test)]`) and examples.

#### Error Handling

- Use `thiserror` for library crates
- Use `anyhow` only in CLI/examples
- All errors must have stable numeric codes for FFI

#### Lock Safety

**Never call external code while holding a lock.** This includes logging, callbacks, and async operations. The workspace enforces `await_holding_lock = "deny"`.

```rust
// BAD: Logging while holding lock
let guard = self.state.lock();
tracing::info!("State: {:?}", *guard);  // Could deadlock!

// GOOD: Release lock before external calls
let state_copy = {
    let guard = self.state.lock();
    guard.clone()
};
tracing::info!("State: {:?}", state_copy);
```

#### FFI Safety

All FFI functions must:
1. Be marked `unsafe extern "C"` with thorough `# Safety` docs
2. Validate all pointers before dereferencing
3. Handle null pointers gracefully
4. Never panic across the FFI boundary (use `catch_unwind`)

### All Languages

#### Test Naming Convention

Use triple underscores to separate test components:

```
subjectUnderTest___condition___expectedResult
```

Examples:
- `lifecycle___installed_state___can_transition_to_starting`
- `plugin_call___with_null_handle___returns_error`
- `bundle_loader___missing_platform___throws_exception`

#### Test Structure

Use Arrange-Act-Assert pattern with blank lines (no comments):

```rust
#[test]
fn config___default_values___are_sensible() {
    let config = PluginConfig::default();

    let worker_threads = config.worker_threads;
    let log_level = config.log_level;

    assert!(worker_threads > 0);
    assert_eq!(log_level, LogLevel::Info);
}
```

## Commit Guidelines

- Write clear, descriptive commit messages
- Reference issues when applicable: `Fix #123`
- Keep commits focused on a single change
- Run `./scripts/pre-commit.sh` before committing

## Pull Request Process

1. Ensure all tests pass: `./scripts/pre-commit.sh`
2. Update documentation if adding/changing features
3. Add changelog entry under `[Unreleased]` in CHANGELOG.md
4. Request review from maintainers

### PR Title Format

Use a clear, descriptive title:
- `Add binary transport support to Python bindings`
- `Fix memory leak in FFI buffer management`
- `Update Java minimum version to 17`

## Project Structure

```
rustbridge/
├── crates/                  # Rust crates
│   ├── rustbridge-core/     # Core traits and types
│   ├── rustbridge-transport/# Serialization
│   ├── rustbridge-ffi/      # C ABI exports
│   ├── rustbridge-runtime/  # Tokio integration
│   ├── rustbridge-logging/  # Tracing bridge
│   ├── rustbridge-macros/   # Procedural macros
│   ├── rustbridge-bundle/   # Bundle format
│   ├── rustbridge-cli/      # CLI tool
│   └── rustbridge-jni/      # JNI bridge
├── rustbridge-java/         # Java/Kotlin bindings
├── rustbridge-csharp/       # C# bindings
├── rustbridge-python/       # Python bindings
├── templates/               # Project templates
├── examples/                # Example plugins
└── docs/                    # Documentation
```

## Documentation

- [ARCHITECTURE.md](./docs/ARCHITECTURE.md) - System design
- [SKILLS.md](./docs/SKILLS.md) - Development best practices
- [TESTING.md](./docs/TESTING.md) - Testing conventions (Rust)
- [TESTING_JAVA.md](./docs/TESTING_JAVA.md) - Java testing
- [TESTING_KOTLIN.md](./docs/TESTING_KOTLIN.md) - Kotlin testing
- [TESTING_CSHARP.md](./docs/TESTING_CSHARP.md) - C# testing
- [TESTING_PYTHON.md](./docs/TESTING_PYTHON.md) - Python testing

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- For questions, use GitHub Discussions

## License

By contributing, you agree that your contributions will be licensed under the same license as the project: MIT OR Apache-2.0.
