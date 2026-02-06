# Plan: Configurable Thread Naming with Numbering

## Overview

Add numbered thread names (e.g., `rb-myplugin-0`, `rb-myplugin-1`) observable in htop/tools, configurable from consumer languages.

## Current State

- `RuntimeConfig.thread_name` exists with default `"rustbridge-worker"`
- Uses `builder.thread_name()` - all threads have identical names (no numbering)
- Not exposed through `PluginConfig` or consumer language configs

## Changes

### Phase 1: Add Numbering (Rust only, ~30 min)

**File: `crates/rustbridge-runtime/src/runtime.rs`**

Change from static `thread_name()` to dynamic `thread_name_fn()`:

```rust
// Before (line 71-72):
builder
    .thread_name(&config.thread_name)

// After:
let thread_prefix = config.thread_name.clone();
builder
    .thread_name_fn(move || {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
        let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
        format!("{}-{}", thread_prefix, id)
    })
```

**Issue:** Static atomic means IDs continue across plugin instances. For per-plugin numbering, store the atomic in `RuntimeConfig` or accept global numbering.

**Simpler approach** (recommended): Use a thread-local or accept that thread IDs are globally unique across all plugins. This is actually useful for distinguishing threads when multiple plugins are loaded.

**Test:** Add test in `runtime_tests.rs` that spawns work and verifies thread names.

### Phase 2: Expose in PluginConfig (Rust, ~20 min)

**File: `crates/rustbridge-core/src/config.rs`**

Add field to `PluginConfig`:

```rust
/// Thread name prefix for async worker threads (default: "rb")
/// Threads will be named "{prefix}-{id}" (e.g., "rb-0", "rb-1")
#[serde(default = "default_thread_prefix")]
pub thread_name_prefix: String,

fn default_thread_prefix() -> String {
    "rb".to_string()
}
```

**File: `crates/rustbridge-ffi/src/handle.rs`**

Update `PluginHandle::new()` (lines 83-86):

```rust
let runtime_config = RuntimeConfig {
    worker_threads: config.worker_threads,
    thread_name: config.thread_name_prefix.clone(),  // Add this
    ..Default::default()
};
```

### Phase 3: Java/Kotlin (~25 min)

**File: `rustbridge-java/rustbridge-core/src/main/java/com/rustbridge/PluginConfig.java`**

Add field and builder method:

```java
private String threadNamePrefix = "rb";

/**
 * Set the thread name prefix for async worker threads.
 * Threads will be named "{prefix}-{id}" (e.g., "rb-0", "rb-1").
 *
 * @param prefix the thread name prefix
 * @return this config for chaining
 */
public @NotNull PluginConfig threadNamePrefix(@NotNull String prefix) {
    this.threadNamePrefix = prefix;
    return this;
}
```

Update `toJsonBytes()`:

```java
json.put("thread_name_prefix", threadNamePrefix);
```

**File: `rustbridge-java/rustbridge-core/src/test/java/com/rustbridge/PluginConfigTest.java`**

Add test for serialization.

**File: `rustbridge-java/rustbridge-kotlin/src/main/kotlin/com/rustbridge/kotlin/PluginConfigDsl.kt`**

Add DSL property if pattern exists.

### Phase 4: C# (~20 min)

**File: `rustbridge-csharp/RustBridge.Core/PluginConfig.cs`**

Add field and builder method:

```csharp
private string _threadNamePrefix = "rb";

/// <summary>
/// Set the thread name prefix for async worker threads.
/// Threads will be named "{prefix}-{id}" (e.g., "rb-0", "rb-1").
/// </summary>
public PluginConfig ThreadNamePrefix(string prefix)
{
    _threadNamePrefix = prefix;
    return this;
}
```

Update `ToJsonBytes()`:

```csharp
json["thread_name_prefix"] = _threadNamePrefix;
```

**File: `rustbridge-csharp/RustBridge.Tests/PluginConfigTests.cs`**

Add test for serialization.

### Phase 5: Python (~20 min)

**File: `rustbridge-python/rustbridge/core/plugin_config.py`**

Add field and method:

```python
def __init__(self) -> None:
    # ... existing fields ...
    self._thread_name_prefix: str = "rb"

def thread_name_prefix(self, prefix: str) -> PluginConfig:
    """
    Set the thread name prefix for async worker threads.
    Threads will be named "{prefix}-{id}" (e.g., "rb-0", "rb-1").

    Args:
        prefix: The thread name prefix.

    Returns:
        This config for chaining.
    """
    self._thread_name_prefix = prefix
    return self
```

Update `to_json_bytes()` and `to_dict()`:

```python
config["thread_name_prefix"] = self._thread_name_prefix
```

**File: `rustbridge-python/tests/test_plugin_config.py`**

Add test for serialization.

### Phase 6: Documentation (~15 min)

**File: `docs/ARCHITECTURE.md`** or appropriate location

Document the thread naming convention and how to customize it.

## Validation

```bash
# Rust
cargo test -p rustbridge-runtime thread_name
cargo test -p rustbridge-core config

# Java
./gradlew :rustbridge-core:test --tests "*PluginConfigTest*"

# C#
dotnet test --filter "FullyQualifiedName~PluginConfig"

# Python
python -m pytest tests/test_plugin_config.py -v
```

## Total Estimate

| Phase | Time |
|-------|------|
| Phase 1: Numbering | 30 min |
| Phase 2: PluginConfig | 20 min |
| Phase 3: Java/Kotlin | 25 min |
| Phase 4: C# | 20 min |
| Phase 5: Python | 20 min |
| Phase 6: Docs | 15 min |
| **Total** | **~2.5 hours** |

## References

- [Tokio Builder docs](https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html) - `thread_name_fn` method
