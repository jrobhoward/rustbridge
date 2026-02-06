# Testing Rust Consumer

This document describes testing conventions for the `rustbridge-consumer` crate.

## Running Tests

```bash
# Run all consumer tests
cargo test -p rustbridge-consumer

# Run tests matching a pattern
cargo test -p rustbridge-consumer NativePluginLoader

# Run with verbose output
cargo test -p rustbridge-consumer -- --nocapture
```

## Test Naming Convention

Tests follow the project-wide triple-underscore naming convention:

```
subject___condition___expectedResult
```

Examples:
```rust
#[test]
fn NativePluginLoader___load___nonexistent_library___returns_error() { }

#[test]
fn NativePlugin___call___returns_json_response() { }

#[test]
fn ffi_log_callback___null_pointers___uses_empty_strings() { }
```

## Test Structure

Tests use the Arrange-Act-Assert pattern with blank lines separating sections:

```rust
#[test]
fn NativePluginLoader___load___nonexistent_library___returns_error() {
    // Arrange (implicit - no setup needed)

    let result = NativePluginLoader::load("/nonexistent/library.so");

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, ConsumerError::LibraryLoad(_)));
}
```

## Integration Tests

Integration tests require building a plugin first:

```bash
# Build the hello-plugin example
cargo build --release -p hello-plugin

# Run integration tests (these are ignored by default)
cargo test -p rustbridge-consumer -- --ignored
```

## Writing Integration Tests

Integration tests should:
1. Load a real plugin (e.g., `hello-plugin`)
2. Make actual FFI calls
3. Verify the responses

```rust
#[test]
#[ignore]  // Requires hello-plugin to be built
fn integration___hello_plugin___echo_roundtrip() {
    let plugin = NativePluginLoader::load(
        "../../target/release/libhello_plugin.so"
    ).expect("Failed to load plugin");

    let response = plugin.call("echo", r#"{"message":"Hello"}"#).unwrap();

    assert!(response.contains("Hello"));
}
```

## Test Modules

Tests are organized in each module file:

```
src/
├── error.rs          # ConsumerError tests
├── ffi_bindings.rs   # FfiBuffer and RbResponse tests
├── loader.rs         # NativePluginLoader tests
├── plugin.rs         # NativePlugin tests
└── lib.rs            # Module exports
```

## Mocking

Since the consumer crate deals with dynamic library loading, most tests are:
1. **Unit tests** - Test individual functions and types
2. **Error path tests** - Verify error handling for invalid inputs
3. **Integration tests** - Test with real plugins (marked `#[ignore]`)

For callback testing, use `Arc<AtomicBool>` to verify callbacks are invoked:

```rust
#[test]
fn ffi_log_callback___with_callback___invokes_callback() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let callback: LogCallbackFn = Arc::new(move |level, target, message| {
        assert_eq!(level, LogLevel::Info);
        called_clone.store(true, Ordering::SeqCst);
    });

    set_log_callback(Some(callback));

    // Invoke the callback...

    assert!(called.load(Ordering::SeqCst));
    set_log_callback(None);  // Clean up
}
```

## Test Dependencies

The crate uses these test dependencies (in `Cargo.toml`):

```toml
[dev-dependencies]
tempfile = "3.24"
tokio = { workspace = true }
```

## Running Clippy on Tests

```bash
cargo clippy -p rustbridge-consumer --tests -- -D warnings
```

## Coverage

To generate test coverage:

```bash
cargo tarpaulin -p rustbridge-consumer --out Html
```
