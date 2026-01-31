# rustbridge Testing Conventions

This document describes the testing conventions for the rustbridge workspace. These conventions apply to Rust code, with language-specific variations documented in separate files.

## Cross-Language Testing Conventions

All rustbridge code (Rust, Java, Kotlin, C#, Python) follows the same core conventions:

| Convention | All Languages |
|------------|---------------|
| **Naming** | `subject___condition___expectedResult` (triple underscores) |
| **Structure** | Arrange-Act-Assert pattern with blank line separation |
| **Comments** | No `// Arrange`, `// Act`, `// Assert` comments |
| **Assertions** | Prefer specific assertions over generic ones |

See language-specific guides for implementation details:
- [TESTING_JAVA.md](./TESTING_JAVA.md) - Java testing conventions
- [TESTING_KOTLIN.md](./TESTING_KOTLIN.md) - Kotlin testing conventions
- [TESTING_CSHARP.md](./TESTING_CSHARP.md) - C# testing conventions
- [TESTING_PYTHON.md](./TESTING_PYTHON.md) - Python testing conventions

## Code Quality Requirements

### Clippy

All code (including tests and examples) must pass clippy cleanly:

```bash
cargo clippy --workspace --examples --tests -- -D warnings
```

This command must run with **zero warnings** before code is considered ready for review.

### Use of `.unwrap()` and `.expect()` in Tests

Unlike production code, tests **may** use `.unwrap()` and `.expect()` freely:

```rust
#[test]
fn PluginConfig___from_valid_json___parses_correctly() {
    let json = r#"{"log_level": "debug"}"#;

    let config = PluginConfig::from_json(json.as_bytes()).unwrap();

    assert_eq!(config.log_level, LogLevel::Debug);
}
```

**Rationale**: Test failures should be visible and obvious. Using `.unwrap()` in tests causes a clear panic with a backtrace, making debugging easier.

### Clippy Configuration for Tests

The workspace `clippy.toml` automatically allows `.unwrap()` and `.expect()` in test code:

```toml
# clippy.toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
```

This applies to:
- Functions marked with `#[test]`
- Code inside `#[cfg(test)]` modules

**Note**: This does NOT apply to integration tests in `tests/` directory, examples, or benchmarks. Those require explicit allow attributes if needed.

### Test File Header

Test files should include the `non_snake_case` allow for the triple-underscore naming convention:

```rust
#![allow(non_snake_case)]

use super::*;
```

For integration tests in `tests/` that need `.unwrap()`:

```rust
#![allow(non_snake_case)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use rustbridge_core::prelude::*;
```

## File Organization

### Separate Test Files

Tests are isolated to separate files rather than inline `mod tests` blocks:

```
crates/rustbridge-core/src/
├── lib.rs
├── error.rs
├── error/
│   └── error_tests.rs    # Tests for error module
├── lifecycle.rs
├── lifecycle/
│   └── lifecycle_tests.rs
├── plugin.rs
└── plugin/
    └── plugin_tests.rs
```

### Module Import

The test module is imported at the **end** of the parent file:

```rust
// error.rs

// ... module implementation ...

#[cfg(test)]
mod error_tests;
```

### Test File Naming

Test files follow the pattern: `{parent_module_name}_tests.rs`

- `error.rs` → `error/error_tests.rs`
- `lifecycle.rs` → `lifecycle/lifecycle_tests.rs`
- `lib.rs` → `lib_tests.rs` (special case: lives in `src/` directly)

## Test File Structure

### File Header

Each test file starts with:

```rust
#![allow(non_snake_case)]

use super::*;  // Import from parent module
// Additional imports as needed
```

The `#![allow(non_snake_case)]` directive permits the triple-underscore naming convention.

## Test Naming Convention

Tests follow a structured naming pattern with **triple underscores** as separators:

```
subject_under_test___condition___expected_result
```

### Components

1. **subject_under_test**: The function, method, or component being tested
2. **condition**: The specific scenario or input condition
3. **expected_result**: What should happen (the assertion)

### Examples

```rust
#[test]
fn PluginConfig___from_empty_json___returns_defaults() { ... }

#[test]
fn LifecycleState___active_to_stopping___transition_succeeds() { ... }

#[test]
fn FfiBuffer___from_vec___preserves_data() { ... }

#[tokio::test]
async fn Plugin___handle_unknown_type___returns_error() { ... }
```

### Guidelines

- Use lowercase with single underscores within each component
- Use triple underscores (`___`) only as separators between components
- Be specific but concise
- The test name should read as a specification

## Test Body Structure

Tests follow the **Arrange-Act-Assert** pattern, separated by blank lines (no comments):

```rust
#[test]
fn LifecycleState___installed_to_starting___transition_succeeds() {
    let state = LifecycleState::Installed;

    let can_transition = state.can_transition_to(LifecycleState::Starting);

    assert!(can_transition);
}
```

### Structure

1. **Arrange**: Set up test data and preconditions (first block)
2. **Act**: Execute the code under test (second block)
3. **Assert**: Verify the results (third block)

### Guidelines

- Separate sections with a single blank line
- Do NOT add `// Arrange`, `// Act`, `// Assert` comments
- Keep each section focused and minimal
- For simple tests, sections may be combined if clarity is maintained

## Async Tests

For async tests, use `#[tokio::test]`:

```rust
#[tokio::test]
async fn Plugin___on_start___transitions_to_active() {
    let plugin = TestPlugin::default();
    let ctx = PluginContext::new(PluginConfig::default());

    let result = plugin.on_start(&ctx).await;

    assert!(result.is_ok());
}
```

## FFI Tests

FFI functions require special care due to unsafe code:

```rust
#[test]
fn plugin_call___null_handle___returns_error_buffer() {
    let result = unsafe {
        plugin_call(
            std::ptr::null_mut(),
            b"test\0".as_ptr() as *const _,
            std::ptr::null(),
            0,
        )
    };

    assert!(result.is_error());
    assert_eq!(result.error_code, 1);

    unsafe { result.free() };
}

#[test]
fn plugin_call___null_type_tag___returns_error_buffer() {
    let result = unsafe {
        plugin_call(
            1 as FfiPluginHandle,
            std::ptr::null(),
            std::ptr::null(),
            0,
        )
    };

    assert!(result.is_error());

    unsafe { result.free() };
}
```

**Key points for FFI tests:**
- Always free buffers after assertions
- Test null pointer handling
- Test invalid handle handling
- Test boundary conditions (empty data, max lengths)

## Test Assertions

Prefer specific assertions over generic ones:

```rust
// Good - specific
assert_eq!(result, expected_value);
assert!(matches!(result, Ok(LifecycleState::Active)));
assert_eq!(error.error_code(), 6);

// Avoid - too generic
assert!(result.is_ok());  // Only use when the Ok value doesn't matter
```

## Integration Tests

Integration tests live in `tests/` directory and test end-to-end flows:

```
rustbridge/
├── crates/
└── tests/
    ├── plugin_lifecycle_test.rs
    └── ffi_roundtrip_test.rs
```

Integration tests follow the **same naming conventions** as unit tests:

```rust
#![allow(non_snake_case)]

use rustbridge_core::prelude::*;
use rustbridge_ffi::prelude::*;

#[tokio::test]
async fn Plugin___full_lifecycle___start_call_stop_succeeds() {
    let plugin = TestPlugin::default();
    let config = PluginConfig::default();

    let handle = PluginHandle::new(Box::new(plugin), config).unwrap();
    handle.start().unwrap();
    let response = handle.call("echo", b"hello").unwrap();
    handle.shutdown(1000).unwrap();

    assert_eq!(response, b"hello");
    assert_eq!(handle.state(), LifecycleState::Stopped);
}
```

## Property-Based Tests

Property-based tests use `proptest` to generate random inputs and verify invariants.

Location: `tests/proptest_tests.rs`

```rust
#![allow(non_snake_case)]

use proptest::prelude::*;
use rustbridge_core::PluginConfig;

proptest! {
    #[test]
    fn PluginConfig___any_json_object___parses_without_panic(
        json in "[a-z]{0,10}"
    ) {
        let _ = PluginConfig::from_json(json.as_bytes());
    }

    #[test]
    fn FfiBuffer___any_vec___roundtrips_correctly(
        data in prop::collection::vec(any::<u8>(), 0..1000)
    ) {
        let mut buffer = FfiBuffer::from_vec(data.clone());

        let slice = unsafe { buffer.as_slice() };
        prop_assert_eq!(slice, &data[..]);

        unsafe { buffer.free() };
    }
}
```

**Run property tests:**
```bash
cargo test --test proptest_tests
```

## Benchmarks

Benchmarks use `criterion` and live in `benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustbridge_transport::JsonCodec;

fn bench_json_codec(c: &mut Criterion) {
    let codec = JsonCodec::new();
    let data = TestMessage { value: 42 };

    c.bench_function("encode_message", |b| {
        b.iter(|| codec.encode(black_box(&data)))
    });

    let encoded = codec.encode(&data).unwrap();
    c.bench_function("decode_message", |b| {
        b.iter(|| codec.decode::<TestMessage>(black_box(&encoded)))
    });
}

criterion_group!(benches, bench_json_codec);
criterion_main!(benches);
```

**Run benchmarks:**
```bash
cargo bench
```

## Test Coverage

Measure test coverage using `cargo-tarpaulin`:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

**Coverage targets:**
- Core traits and types: >90%
- FFI boundary: >80%
- Error handling: >80%
- Overall: >70%

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p rustbridge-core

# Run a specific test
cargo test lifecycle___installed_to_starting

# Run tests with output
cargo test -- --nocapture

# Run only unit tests (no integration tests)
cargo test --lib

# Run only integration tests
cargo test --test '*'
```

## Test Utilities

Create test utilities in a `test_utils` module when needed:

```rust
// In tests/common/mod.rs or src/test_utils.rs

pub fn create_test_plugin() -> impl Plugin {
    TestPlugin::default()
}

pub fn create_test_context() -> PluginContext {
    PluginContext::new(PluginConfig::default())
}

pub struct TestPlugin;

#[async_trait]
impl Plugin for TestPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "echo" => Ok(payload.to_vec()),
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }
}
```

## Memory Safety Tests

For FFI code, run tests with sanitizers:

```bash
# Address Sanitizer (Linux)
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test -p rustbridge-ffi

# Memory Sanitizer (Linux)
RUSTFLAGS="-Z sanitizer=memory" cargo +nightly test -p rustbridge-ffi

# Valgrind
cargo build --tests
valgrind --leak-check=full ./target/debug/deps/rustbridge_ffi-*
```

## Test Timeouts

**Always add timeouts to integration tests** to prevent builds from hanging indefinitely. This is especially important for tests involving:
- Async operations
- FFI callbacks
- Concurrent/parallel operations
- Resource cleanup / lifecycle management

### Per-Test Timeout with Tokio

```rust
#[tokio::test(flavor = "multi_thread")]
async fn plugin___long_operation___completes_in_time() {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        plugin.async_operation()
    ).await;

    assert!(result.is_ok(), "Operation timed out");
}
```

### Timeout Helper Function

Create a reusable timeout wrapper:

```rust
// In test_utils.rs
pub async fn with_timeout<F, T>(duration_secs: u64, future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(
        std::time::Duration::from_secs(duration_secs),
        future
    )
    .await
    .expect("Test timed out")
}

// Usage
#[tokio::test]
async fn plugin___operation___succeeds() {
    let result = with_timeout(30, async {
        plugin.do_something().await
    }).await;

    assert!(result.is_ok());
}
```

### Timeout Guidelines

| Test Type | Recommended Timeout |
|-----------|-------------------|
| Unit tests | 5-10 seconds |
| Integration tests | 30-60 seconds |
| FFI roundtrip tests | 30 seconds |
| Stress tests | 2-5 minutes |

### Deadlock Prevention

For tests involving locks, add timeout assertions:

```rust
#[test]
fn lock___concurrent_access___no_deadlock() {
    let result = std::thread::spawn(|| {
        // Code that acquires locks
        manager.do_locked_operation()
    });

    // Fail if thread doesn't complete in 10 seconds
    let handle = result.join();
    assert!(handle.is_ok(), "Possible deadlock detected");
}
```

**Rationale**: Tests that hang indefinitely can block CI pipelines and developer workflows. Timeouts provide fail-fast behavior and clear error messages about which tests are problematic.

## CI Testing

Tests run in CI with the following matrix:
- **OS**: Linux x64, macOS ARM64, Windows x64
- **Rust**: stable, beta, nightly
- **Features**: default, all-features

Example GitHub Actions workflow:
```yaml
test:
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      rust: [stable, beta]
  runs-on: ${{ matrix.os }}
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
    - run: cargo test --all
```
