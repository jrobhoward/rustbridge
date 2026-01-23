# rustbridge - Project Instructions

This document contains instructions and conventions for working on the rustbridge project.

## Project Overview

rustbridge is a framework for developing Rust shared libraries callable from other languages (Java, C#, Python, etc.). It uses C ABI under the hood but abstracts the complexity.

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

### Clippy

All code must pass clippy with no warnings:

```bash
cargo clippy --workspace --examples --tests -- -D warnings
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

## Build Commands

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run clippy (must pass with no warnings)
cargo clippy --workspace --examples --tests -- -D warnings

# Build release
cargo build --workspace --release
```
