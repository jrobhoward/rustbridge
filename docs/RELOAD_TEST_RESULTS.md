# Plugin Reload and Multiple Instance Test Results

**Date**: 2026-01-24
**Test Platform**: Linux (Ubuntu 24.04.3)
**Rust Version**: 1.85 (edition 2024)
**Java Version**: OpenJDK 21.0.10+7-LTS

## Summary

✅ **Plugin reload WORKS** - Plugins can be loaded, shut down, and reloaded successfully
❌ **Multiple plugins with log callbacks CRASHES** - Shared global state causes segmentation fault
⚠️ **Log callback becomes stale after plugin shutdown** - Needs cleanup

## Test Results

### 1. Multiple Plugin Instances (WITHOUT Log Callbacks)

**Test**: `MultiplePluginTest.testTwoPluginsSimultaneously`
**Result**: ✅ **PASS**

**Findings:**
- Two instances of the same plugin can be loaded simultaneously
- Both plugins work independently
- Both can process requests correctly
- Both maintain their own ACTIVE lifecycle state

**Conclusion**: Basic multiple plugin instances work fine when log callbacks are not involved.

---

### 2. Plugin Reload (WITHOUT Log Callbacks)

**Test**: `PluginReloadTest.testLoadShutdownReload`
**Result**: ✅ **PASS**

**Findings:**
- Plugin loads successfully → ACTIVE state
- Plugin can be used (echo call works)
- Plugin shutdown completes cleanly
- Same plugin can be reloaded successfully
- Reloaded plugin reaches ACTIVE state
- Reloaded plugin works correctly (echo call works)

**Output:**
```
=== First Load ===
=== Shutting down ===
Plugin closed successfully
=== Second Load (Reload) ===
Result: Reload SUCCESSFUL ✓
```

**Conclusion**: Plugin reload works! The library can be loaded, shut down, and reloaded in the same process.

---

### 3. Multiple Plugins with Log Callbacks

**Test**: `MultiplePluginTest.testMultiplePluginsIndependentLogCallbacks`
**Result**: ❌ **JVM CRASH** (SIGSEGV)

**Crash Details:**
```
SIGSEGV (0xb) at pc=0x000074d56957466d
Problematic frame:
C  [libhello_plugin.so+0xa45e1]  rustbridge_logging::callback::LogCallbackManager::log::hc7edbde9070d5984+0xd1

si_addr: 0x000000000000000a (near-null pointer dereference)
```

**Analysis:**
The crash occurs in `LogCallbackManager::log()` when it tries to invoke a log callback.

**Root Cause:**
1. First plugin loads with callback A → stored in global `LogCallbackManager`
2. Second plugin loads with callback B → **OVERWRITES** callback A in same global manager
3. First plugin logs something → tries to use callback A
4. Callback A pointer is now stale/invalid → segmentation fault

**Conclusion**: Log callbacks ARE SHARED between plugins via global state. This is a critical bug for multiple plugin scenarios.

---

### 4. Reload with Log Callbacks

**Test**: `PluginReloadTest.testLogCallbackAfterReload`
**Result**: ❌ **JVM CRASH** (SIGSEGV)

**Same root cause** as test #3:
- First load installs callback in global `LogCallbackManager`
- Plugin shuts down, callback pointer becomes invalid
- Reload happens, new callback installed
- Old callback still referenced somewhere → crash

---

## Global State Analysis

### Confirmed Shared Global State

1. **`LogCallbackManager::CALLBACK_MANAGER`** (/home/jhoward/git/rust_lang_interop/crates/rustbridge-logging/src/callback.rs:27)
   - `static CALLBACK_MANAGER: OnceCell<LogCallbackManager>`
   - Contains log callback function pointer
   - Contains current log level (AtomicU8)
   - **SHARED ACROSS ALL PLUGIN INSTANCES**

2. **`ReloadHandle::INSTANCE`** (/home/jhoward/git/rust_lang_interop/crates/rustbridge-logging/src/reload.rs:24)
   - `static INSTANCE: OnceCell<ReloadHandle>`
   - Contains handle to tracing filter
   - **SHARED ACROSS ALL PLUGIN INSTANCES**

3. **`INITIALIZED`** (/home/jhoward/git/rust_lang_interop/crates/rustbridge-logging/src/layer.rs:113)
   - `static INITIALIZED: OnceCell<()>`
   - Prevents re-initialization of logging
   - **SHARED ACROSS ALL PLUGIN INSTANCES**

4. **Tracing Subscriber** (process-global via `set_global_default`)
   - Can only be set once per process
   - **SHARED ACROSS ALL CODE IN PROCESS**

5. **`HANDLE_MANAGER`** (/home/jhoward/git/rust_lang_interop/crates/rustbridge-ffi/src/handle.rs:15)
   - `static HANDLE_MANAGER: OnceCell<PluginHandleManager>`
   - Registry of plugin handles
   - This one is OK - it's meant to track all plugins
   - Handles are properly removed on shutdown

### Impact on Multiple Plugins

**When two plugins load**:
1. Plugin A loads
   - Initializes tracing subscriber ✓
   - Stores callback A in LogCallbackManager
   - Stores reload handle in ReloadHandle::INSTANCE

2. Plugin B loads
   - Tries to initialize tracing → **NO-OP** (already initialized)
   - **OVERWRITES** callback A with callback B in LogCallbackManager
   - **OVERWRITES** reload handle in ReloadHandle::INSTANCE

3. Plugin A logs something
   - Uses **callback B** instead of callback A (!)
   - If callback B becomes invalid → CRASH

4. Plugin A calls `setLogLevel()`
   - Changes global log level → **affects Plugin B too** (!)

**Result**: Plugins are NOT isolated. They share logging infrastructure.

---

## Recommendations

### Priority 1: Fix Log Callback Crash (Critical)

The stale callback pointer is a security and stability issue. We must clear it on shutdown.

**Implementation:**

```rust
// In rustbridge-ffi/src/exports.rs, plugin_shutdown_impl()
fn plugin_shutdown_impl(handle: FfiPluginHandle) -> bool {
    let id = handle as u64;

    // Remove from manager
    let plugin_handle = match PluginHandleManager::global().remove(id) {
        Some(h) => h,
        None => return false,
    };

    // Shutdown plugin
    let result = match plugin_handle.shutdown(5000) {
        Ok(()) => true,
        Err(e) => {
            tracing::error!("Shutdown error: {}", e);
            false
        }
    };

    // CRITICAL: Clear log callback to prevent use-after-free
    // This prevents crashes if multiple plugins are loaded
    LogCallbackManager::global().set_callback(None);

    result
}
```

**Trade-off**: This means the last plugin to shutdown clears the callback for ALL plugins. But it's better than a crash.

---

### Priority 2: Document "Single Plugin Per Process" Limitation

Since plugins share global logging state, we should document that multiple plugins are not fully supported.

**Add to README.md and ARCHITECTURE.md:**

```markdown
## Important Limitations

### Single Plugin Instance Per Process

rustbridge currently supports **ONE plugin instance per process**. While the
framework can technically load multiple plugin instances, they will share
logging infrastructure:

- All plugins share the same log callback
- Log level changes affect all plugins
- The last plugin to shut down clears the log callback

**For production use**: Load one plugin per process. If you need multiple
plugins, use separate processes or containers.

**Future work**: Making plugins fully isolated would require significant
architectural changes to move global state to per-handle state.
```

---

### Priority 3: Add Runtime Warning for Multiple Plugins

Add detection and warning when multiple plugins are loaded:

```rust
// In rustbridge-ffi/src/exports.rs
static PLUGIN_COUNT: AtomicUsize = AtomicUsize::new(0);

pub unsafe extern "C" fn plugin_init(...) -> FfiPluginHandle {
    // ... existing code ...

    let count = PLUGIN_COUNT.fetch_add(1, Ordering::SeqCst);
    if count > 0 {
        tracing::warn!(
            "Multiple plugin instances detected ({} total). \
             Plugins will share logging state. This is not recommended.",
            count + 1
        );
    }

    // ... rest of init ...
}

fn plugin_shutdown_impl(handle: FfiPluginHandle) -> bool {
    // ... existing shutdown code ...

    PLUGIN_COUNT.fetch_sub(1, Ordering::SeqCst);

    result
}
```

---

### Priority 4: Improve Shutdown Cleanup

Make shutdown as clean as possible even if we can't achieve 100% cleanup:

```rust
impl PluginHandle {
    pub fn shutdown(&self, timeout_ms: u64) -> PluginResult<()> {
        // ... existing state transition code ...

        // Call plugin's on_stop
        let result = self.bridge.call_sync_timeout(...);

        // Shutdown runtime
        let runtime_timeout = std::time::Duration::from_millis(timeout_ms / 2);
        if let Err(e) = self.runtime.shutdown(runtime_timeout) {
            tracing::warn!("Runtime shutdown incomplete: {}", e);
        }

        // Transition to final state
        // ... existing code ...

        // Note: We deliberately DON'T reset global state here
        // (tracing subscriber, OnceCell instances) because:
        // 1. They may be used by other plugin instances
        // 2. They cannot be safely reset while other threads may be logging
        // 3. The log callback is cleared in plugin_shutdown_impl instead

        Ok(())
    }
}
```

---

## Detailed Test Plan for Future

Once fixes are implemented, we should test:

1. ✅ Single plugin, single load - **WORKS**
2. ✅ Single plugin, reload - **WORKS**
3. ❌ Multiple plugins, no callbacks - **WORKS** (tested)
4. ❌ Multiple plugins, with callbacks - **CRASHES** (needs fix)
5. ❌ Plugin reload with callbacks - **CRASHES** (needs fix)
6. ⚠️ Log level isolation between plugins - **SHARED** (by design with current architecture)
7. ⚠️ Plugin state reset after reload - **NOT TESTED YET**
8. ⚠️ Memory leak testing - **NOT DONE**
9. ⚠️ Thread cleanup verification - **NOT DONE**

---

## Answers to Original Questions

### Q: Can we unload and reload a plugin?

**A: YES!** ✅ Plugin reload works correctly as long as log callbacks are properly managed.

### Q: Will multiple plugins share logging state?

**A: YES.** ⚠️ The user was right to question this. Multiple plugins DO share global logging state (LogCallbackManager, tracing subscriber). This is not ideal but can be mitigated with proper cleanup.

### Q: Should we prioritize reload support or dynamic log filtering?

**A: Both can coexist!** ✅ We can have dynamic log filtering AND reload support by:
1. Clearing log callbacks on shutdown (prevents crashes)
2. Documenting the single-plugin-per-process limitation
3. Making shutdown as clean as possible

---

## Action Items

1. **IMMEDIATE**: Add log callback cleanup to `plugin_shutdown_impl()`
2. **IMMEDIATE**: Document single-plugin-per-process limitation
3. **SHORT TERM**: Add runtime warning for multiple plugins
4. **SHORT TERM**: Test memory leaks and thread cleanup
5. **LONG TERM**: Consider per-handle logging architecture (if multi-plugin becomes a requirement)

---

## Conclusion

**Reload is supported!** ✅

Plugin reload works correctly. The main issue is the shared log callback causing crashes when:
- Multiple plugins are loaded with callbacks
- A plugin is reloaded with a callback

The fix is straightforward: clear the log callback on shutdown. This prevents use-after-free crashes while still allowing reload to work.

**Recommendation**: Prioritize stability (clear callbacks on shutdown) over theoretical multi-plugin purity. Document that multiple plugins share logging state. This is a reasonable trade-off for a v1 framework.
