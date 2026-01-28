# rustbridge Development Skills & Best Practices

This document captures best practices and conventions for the rustbridge workspace.

## Git Workflow

**The user controls git operations.** Do not commit, push, or create branches without explicit user request.

- **Never auto-commit**: Wait for the user to request a commit
- **Never push**: The user decides when and where to push
- **Describe changes**: When asked to commit, summarize what changed clearly
- **Stage selectively**: Only stage files related to the current task

### Creating Release Tags

Use annotated tags for releases (includes tagger name, date, and message):

```bash
# Create an annotated tag
git tag -a v0.5.0 -m "Release v0.5.0"

# Push the tag to remote
git push origin v0.5.0

# Or push branch and tag together
git push origin main v0.5.0

# Or push all tags at once
git push origin --tags
```

Lightweight tags (without `-a`) are just pointers to commits and lack metadata.

## Error Handling

### Use `thiserror` for Error Types

Prefer `thiserror` over `anyhow` for defining error types in library crates. This provides:
- Strongly-typed errors with clear semantics
- Automatic `Display` and `Error` trait implementations
- Better error composition and matching
- Clearer API contracts

**Example:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("invalid lifecycle state: expected {expected}, got {actual}")]
    InvalidState {
        expected: String,
        actual: String,
    },

    #[error("initialization failed: {0}")]
    InitializationFailed(String),

    #[error("unknown message type: {0}")]
    UnknownMessageType(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("handler error: {0}")]
    HandlerError(String),
}
```

**Guidelines:**
- Create specific error variants for each failure mode
- Include actionable context in error messages
- Use `#[from]` for automatic conversion from underlying errors
- Use `#[source]` to preserve error chains
- Keep `anyhow` for top-level error aggregation in CLI/examples only

### Error Codes for FFI

All errors should have stable numeric codes for FFI communication:

```rust
impl PluginError {
    pub fn error_code(&self) -> u32 {
        match self {
            PluginError::InvalidState { .. } => 1,
            PluginError::InitializationFailed(_) => 2,
            PluginError::SerializationError(_) => 5,
            PluginError::UnknownMessageType(_) => 6,
            // ...
        }
    }
}
```

### Avoiding `.unwrap()` and `.expect()`

In **production code** (library crates), avoid `.unwrap()` and `.expect()`:

```rust
// BAD - will panic on failure
let config = PluginConfig::from_json(data).unwrap();

// GOOD - propagate errors
let config = PluginConfig::from_json(data)?;

// GOOD - provide default
let config = PluginConfig::from_json(data).unwrap_or_default();

// GOOD - handle explicitly
let config = match PluginConfig::from_json(data) {
    Ok(c) => c,
    Err(e) => {
        tracing::warn!("Invalid config, using defaults: {}", e);
        PluginConfig::default()
    }
};
```

**When `.unwrap()` or `.expect()` is truly safe** (e.g., compile-time known values), add an allow attribute with explanation:

```rust
#[allow(clippy::unwrap_used)] // Safe: regex pattern is valid at compile time
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});
```

**Exceptions where `.unwrap()` / `.expect()` are allowed:**
- Test code (`#[cfg(test)]` modules) - automatically via `clippy.toml`
- Examples (`examples/` directory) - configured in crate's `Cargo.toml`
- Truly infallible operations with `#[allow(...)]` annotation

The workspace `clippy.toml` sets `allow-unwrap-in-tests = true` and `allow-expect-in-tests = true` to suppress these warnings in test code automatically.

## Code Quality

### Formatting

All code must be formatted with `rustfmt` before committing:

```bash
cargo fmt --all
```

Check formatting in CI:

```bash
cargo fmt --all -- --check
```

**Important**: Always run `cargo fmt --all` before committing code. Unformatted code will be rejected in code review.

### Clippy

All code must pass clippy with no warnings:

```bash
cargo clippy --workspace --examples --tests -- -D warnings
```

This includes:
- All library crates
- All examples
- All test code

Run this command before considering any code complete.

### Workspace Clippy Configuration

The workspace `Cargo.toml` should configure strict lints:

```toml
[workspace.lints.clippy]
unwrap_used = "warn"      # Warn on .unwrap() in production code
expect_used = "warn"      # Warn on .expect() in production code
panic = "warn"            # Warn on panic!() in production code
```

Individual crates inherit these with:

```toml
[lints]
workspace = true
```

## Code Organization

### Import Style

**Prefer `use` imports at the top of each file** rather than fully-qualified paths scattered throughout the code.

**Guidelines:**
- Group imports in this order: std, external crates, workspace crates, local modules
- Use explicit imports for types/functions used multiple times
- Fully-qualified paths are acceptable for one-off usages within a function

```rust
// Good - imports at top, grouped logically
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{info, warn};

use rustbridge_core::{Plugin, PluginConfig, PluginContext};
use rustbridge_transport::ResponseEnvelope;

use crate::buffer::FfiBuffer;
use crate::handle::PluginHandle;
```

### Crate Organization

Each crate should have a clear, focused responsibility:

| Crate | Responsibility |
|-------|---------------|
| `rustbridge-core` | Core traits, types, lifecycle |
| `rustbridge-transport` | Serialization, message envelopes |
| `rustbridge-ffi` | C ABI exports, buffer management |
| `rustbridge-runtime` | Async runtime, shutdown signals |
| `rustbridge-logging` | Tracing integration, FFI callbacks |
| `rustbridge-macros` | Procedural macros |
| `rustbridge-cli` | Build tool, code generation |

### Lock Safety Rules

**Never call external code while holding a lock.** This prevents deadlocks caused by lock reentrancy or callback cycles.

**Forbidden patterns while holding a lock:**

```rust
// ❌ BAD - logging while holding lock
let guard = self.data.write();
tracing::debug!("Updated data");  // May try to acquire same lock via callback!
*guard = new_value;

// ❌ BAD - callback while holding lock
let guard = self.data.read();
self.callback.invoke(&guard);  // Callback might try to acquire lock!

// ❌ BAD - async/await while holding lock (Clippy: await_holding_lock)
let guard = self.data.lock().await;
some_async_operation().await;  // Holds lock across await point!
```

**Safe patterns:**

```rust
// ✅ GOOD - release lock before logging
{
    let mut guard = self.data.write();
    *guard = new_value;
} // Lock released
tracing::debug!("Updated data");  // Safe to log now

// ✅ GOOD - copy data, release lock, then use
let data_copy = {
    let guard = self.data.read();
    guard.clone()
}; // Lock released
self.callback.invoke(&data_copy);  // Safe to invoke callback

// ✅ GOOD - minimize critical section
let result = {
    let mut guard = self.data.write();
    guard.compute_result()
}; // Lock released immediately
process_result(result);  // Long operation outside lock
```

**Lock ordering:** If multiple locks must be held simultaneously, document and enforce a consistent acquisition order to prevent deadlocks:

```rust
// Document lock order: always acquire `users` before `sessions`
let users = self.users.write();
let sessions = self.sessions.write();  // Always after users
```

**Why this matters:** The log callback deadlock in `rustbridge-logging` was caused by calling `tracing::debug!()` while holding a write lock. The tracing layer tried to acquire a read lock on the same RwLock, causing a deadlock. Rust's `parking_lot::RwLock` does not support reentrancy.

### FFI Safety

All FFI functions must:
1. Be marked `unsafe extern "C"`
2. Have thorough safety documentation
3. Validate all pointer arguments before dereferencing
4. Handle null pointers gracefully
5. Never panic across the FFI boundary

```rust
/// # Safety
/// - `handle` must be a valid handle from `plugin_init`
/// - `type_tag` must be a valid null-terminated C string
/// - `request` must be valid for `request_len` bytes
#[no_mangle]
pub unsafe extern "C" fn plugin_call(
    handle: FfiPluginHandle,
    type_tag: *const std::ffi::c_char,
    request: *const u8,
    request_len: usize,
) -> FfiBuffer {
    // Validate inputs before use
    if handle.is_null() {
        return FfiBuffer::error(1, "Invalid handle");
    }
    // ...
}
```

### Constants

Define constants for magic values:

```rust
/// Default timeout for RPC calls
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(5);

/// Default shutdown timeout in milliseconds
pub const DEFAULT_SHUTDOWN_TIMEOUT_MS: u64 = 5000;

/// Maximum concurrent async operations
pub const DEFAULT_MAX_CONCURRENT_OPS: usize = 1000;
```

### Async Patterns

All plugin operations are async-first with Tokio:

```rust
#[async_trait]
impl Plugin for MyPlugin {
    async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
        // Async initialization
        Ok(())
    }

    async fn handle_request(
        &self,
        ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        // Async request handling
    }

    async fn on_stop(&self, ctx: &PluginContext) -> PluginResult<()> {
        // Async cleanup
        Ok(())
    }
}
```

For sync FFI calls, use the AsyncBridge:

```rust
pub fn call(&self, type_tag: &str, request: &[u8]) -> PluginResult<Vec<u8>> {
    self.bridge.call_sync(
        self.plugin.handle_request(&self.context, type_tag, request)
    )
}
```

## Message Design

### Request/Response Types

Use Serde for all message types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub user_id: String,
    pub created_at: String,
}
```

### Type Tags

Use dot-separated namespacing for type tags:

- `user.create` - Create a user
- `user.delete` - Delete a user
- `order.submit` - Submit an order
- `math.add` - Add numbers

### Envelope Pattern

Wrap requests and responses in envelopes for FFI transport:

```rust
#[derive(Serialize, Deserialize)]
pub struct RequestEnvelope {
    pub type_tag: String,
    pub payload: serde_json::Value,
    pub request_id: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub status: ResponseStatus,
    pub payload: Option<serde_json::Value>,
    pub error_code: Option<u32>,
    pub error_message: Option<String>,
}
```

## Testing

See [TESTING.md](./TESTING.md) for complete testing conventions.

### Key Points

- **Tests belong in separate files**, not inline `mod tests` blocks
- Test naming: `subject_under_test___condition___expected_result`
- Use Arrange-Act-Assert pattern separated by whitespace (no comments)
- Use `#[tokio::test]` for async tests
- Test error paths, not just happy paths
- Test FFI functions with both valid and invalid inputs

## Documentation

### Module-Level Docs

Every `lib.rs` should have comprehensive module documentation:

```rust
//! rustbridge-core - Core traits and types
//!
//! This crate provides:
//! - [`Plugin`] trait for implementing plugins
//! - [`LifecycleState`] for managing plugin lifecycle
//! - [`PluginError`] for error handling
```

### Function Documentation

Document all public functions with:
- Brief description
- Parameters (for non-obvious cases)
- Returns
- Errors (what can fail)
- Examples (for complex APIs)
- Safety (for unsafe functions)

```rust
/// Initialize a plugin instance.
///
/// # Safety
/// - `plugin_ptr` must be a valid pointer from `plugin_create`
/// - `config_json` must be valid for `config_len` bytes if not null
///
/// # Returns
/// Handle to the initialized plugin, or null on failure.
#[no_mangle]
pub unsafe extern "C" fn plugin_init(...) -> FfiPluginHandle { ... }
```

## Java Integration

### FFM (Java 21+)

Prefer Project Panama FFM for Java 21+:
- Pure Java bindings, no JNI code needed
- Arena-based memory management
- Better performance than JNI

### JNI (Java 17+)

JNI fallback for older JVM compatibility:
- More complex but widely supported
- Requires native code generation

### Memory Management

Follow "Rust allocates, host frees" pattern:
1. `plugin_call()` returns `FfiBuffer` with Rust-allocated data
2. Host copies data to managed heap
3. Host calls `plugin_free_buffer()` to release native memory

## Plugin Reload Safety

rustbridge is designed to support plugin reload cycles (load → shutdown → reload). This section documents safe patterns for global state management.

### Global State in rustbridge

The framework uses the following global state:

1. **HANDLE_MANAGER** (`OnceCell<PluginHandleManager>`) - Stores active plugin handles in a DashMap
2. **CALLBACK_MANAGER** (`OnceCell<LogCallbackManager>`) - Manages FFI log callbacks with reference counting
3. **BINARY_HANDLERS** (`thread_local! HashMap`) - Stores binary message handlers per thread
4. **ReloadHandle** - Tracing subscriber filter reload handle

### Reload Safety Guarantees

**Automatic cleanup on shutdown:**
- Plugin handles are removed from HANDLE_MANAGER when `plugin_shutdown()` is called
- Log callback is cleared when the last plugin unregisters (ref count reaches 0)
- Binary handlers are cleared on shutdown to prevent stale handlers
- Reload handle persists across reloads (logging only initializes once per process)

**Thread-local state:**
- Binary handlers are cleared per-thread on shutdown
- Each thread maintains its own handler registry

### Safe Global State Patterns for Plugin Authors

When writing plugins, follow these patterns to ensure reload safety:

**✅ GOOD: State scoped to plugin handle**
```rust
pub struct MyPlugin {
    // Plugin-specific state, automatically cleaned up when plugin is dropped
    state: Arc<RwLock<PluginState>>,
}
```

**✅ GOOD: Resettable global state**
```rust
use parking_lot::RwLock;
use std::sync::OnceLock;

static MANAGER: OnceLock<RwLock<Manager>> = OnceLock::new();

impl Plugin for MyPlugin {
    async fn on_stop(&self, ctx: &PluginContext) -> PluginResult<()> {
        // Clear the manager on shutdown
        if let Some(manager) = MANAGER.get() {
            *manager.write() = Manager::new(); // Reset to default state
        }
        Ok(())
    }
}
```

**❌ BAD: Non-resettable global state**
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

// This will persist across reloads, causing incorrect behavior!
static mut COUNTER: AtomicUsize = AtomicUsize::new(0);
```

**❌ BAD: Unclearable caches**
```rust
use std::collections::HashMap;
use std::sync::OnceLock;

// This cache will grow indefinitely across reloads!
static CACHE: OnceLock<HashMap<String, Data>> = OnceLock::new();
```

### Guidelines for Plugin Developers

1. **Prefer handle-scoped state** - Store state in your Plugin struct, not in globals
2. **Use `on_stop()` for cleanup** - Reset any global state in `on_stop()`
3. **Avoid `static mut`** - Use thread-safe primitives (`RwLock`, `Mutex`, `Atomic*`)
4. **Document global state** - Comment why global state is needed and how it's cleaned up
5. **Test reload cycles** - Verify your plugin works correctly across multiple load/unload cycles

### Reload Cycle Testing

Test your plugin with reload cycles to ensure proper cleanup:

```java
// Java test example
@Test
void testReloadCycle() {
    // First load
    FfmPlugin plugin1 = loader.load();
    plugin1.init(config, callback);
    plugin1.start();
    plugin1.call("echo", request);
    plugin1.shutdown();

    // Reload same plugin
    FfmPlugin plugin2 = loader.load();
    plugin2.init(config, callback);
    plugin2.start();
    plugin2.call("echo", request); // Should work correctly
    plugin2.shutdown();
}
```

### Known Limitations

1. **OnceCell globals persist** - The OnceCell containers themselves are never cleared (only their contents)
2. **Static initialization order** - Rust doesn't guarantee initialization order across reloads
3. **Platform-specific behavior** - Some platforms may cache dynamic libraries differently

These limitations are acceptable for the framework's use cases. The important guarantee is that **no stale data persists** across reloads that would cause incorrect behavior.
