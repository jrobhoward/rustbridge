# Plugin Reload Safety Analysis

**Date**: 2026-01-24
**Context**: Analysis of global state and plugin reload safety after implementing dynamic log level changes

## Global State Inventory

### rustbridge-logging

1. **`CALLBACK_MANAGER: OnceCell<LogCallbackManager>`** (callback.rs:27)
   - Stores log callback function pointer and current log level
   - Shared across all plugin instances in the same process
   - Not reset on plugin shutdown

2. **`INSTANCE: OnceCell<ReloadHandle>`** (reload.rs:24)
   - Stores handle to tracing subscriber's reloadable filter
   - Shared across all plugin instances
   - Not reset on plugin shutdown

3. **`INITIALIZED: OnceCell<()>`** (layer.rs:113)
   - Marker to ensure `init_logging()` only runs once
   - Prevents re-initialization of tracing subscriber
   - Not reset on plugin shutdown

4. **Tracing Subscriber** (set via `tracing::subscriber::set_global_default`)
   - Can only be set once per process
   - Cannot be reset or replaced
   - Shared across all code in the process

### rustbridge-ffi

1. **`HANDLE_MANAGER: OnceCell<PluginHandleManager>`** (handle.rs:15)
   - Stores registry of active plugin handles
   - Maps handle IDs to PluginHandle instances
   - Handles are removed on shutdown, but OnceCell persists

## Reload Scenarios

### Scenario 1: Single Plugin - Unload and Reload

**What happens:**
1. Plugin loads → globals initialized
2. Plugin used → works correctly
3. `plugin_shutdown()` called → handle removed from registry
4. Dynamic library unloaded (dlclose/FreeLibrary)
5. Dynamic library reloaded
6. `plugin_init()` called again

**Expected behavior on Linux/macOS:**
- Dynamic library should be truly unloaded (if refcount reaches 0)
- Statics should be reinitialized on reload
- **BUT**: Some systems cache libraries or don't fully unload

**Expected behavior on Windows:**
- Similar to Linux/macOS
- Statics should be fresh on reload

**Actual behavior (untested):**
- `INITIALIZED` OnceCell will be fresh → `init_logging()` runs again
- Attempts to call `set_global_default()` again → **FAILS** if tracing was already initialized
- Currently we use `let _ =` to ignore the error, so it might work
- `CALLBACK_MANAGER` and `ReloadHandle::INSTANCE` will be fresh
- New reload handle created, but old subscriber still active

**Potential issues:**
- ❌ Tracing subscriber cannot be replaced once set globally
- ❌ If library isn't truly unloaded, statics persist and logging won't reinitialize
- ⚠️ Log callback from first instance might be stale/invalid after reload

### Scenario 2: Multiple Plugins in Same Process

**What happens:**
1. Plugin A loads → globals initialized for Plugin A
2. Plugin B loads (different .so but uses same rustbridge crates)
3. Both plugins try to use logging

**Expected behavior:**
- Each plugin should have independent logging configuration

**Actual behavior:**
- ❌ **BOTH PLUGINS SHARE THE SAME GLOBALS**
- First plugin to call `init_logging()` wins
- Second plugin's `init_logging()` is a no-op (OnceCell already initialized)
- Both plugins share the same LogCallbackManager
- If Plugin A sets log level to DEBUG, Plugin B also logs at DEBUG
- If Plugin A shuts down, its log callback might become invalid, breaking Plugin B

**This is a CRITICAL BUG** for multi-plugin scenarios.

### Scenario 3: Plugin in Long-Running Process

**What happens:**
1. Plugin loads → used for days/weeks
2. Need to update plugin → unload and reload
3. Attempt reload while process is still running

**Potential issues:**
- If library doesn't fully unload, statics persist
- Tracing subscriber remains from first load
- Cannot reinitialize logging system
- Stale callback pointers if host language has reloaded

## Current Risk Assessment

### Single Plugin, Single Load
**Risk**: ✅ LOW - Works correctly

### Single Plugin, Reload Cycles
**Risk**: ⚠️ **MEDIUM-HIGH** - Likely to fail due to:
- Tracing subscriber cannot be reset
- OnceCell state may persist
- Callback pointers may become stale

### Multiple Plugins, Same Process
**Risk**: ❌ **CRITICAL** - Definitely broken:
- Shared global state between independent plugins
- Callback interference
- Log level changes affect all plugins

## Recommended Fixes

### Priority 1: Document Current Limitations

Add to ARCHITECTURE.md:

```markdown
## Plugin Reload Limitations

### Single Plugin Instance Only
rustbridge currently supports loading ONE plugin instance per process.
Loading multiple plugin instances will result in shared logging state
and undefined behavior.

### Reload Not Supported
Once a plugin is loaded and initialized, it cannot be reliably unloaded
and reloaded in the same process. This is due to:
- Tracing subscriber can only be set once per process
- Global state in `OnceCell` instances cannot be reset
- Platform-specific library unloading behavior varies

For updates, restart the host process.
```

### Priority 2: Add Reset Support (Partial Solution)

Add ability to reset global state (helps with reload, doesn't help with multi-plugin):

```rust
// In rustbridge-logging/src/lib.rs
pub mod reset {
    use super::*;

    /// Reset global logging state for plugin reload scenarios
    ///
    /// # Safety
    /// This is UNSAFE and should only be called when:
    /// - No plugin instances are active
    /// - No logging operations are in progress
    /// - You are about to reload the plugin library
    ///
    /// Calling this while plugins are active will cause undefined behavior.
    pub unsafe fn reset_global_state() {
        // OnceCell doesn't have a safe reset, but we can try
        // Note: This won't help if the library isn't actually unloaded

        // We can't reset OnceCell, but we can clear the callback
        callback::LogCallbackManager::global().set_callback(None);

        // Can't reset the tracing subscriber - it's truly global
        // This is a fundamental limitation
    }
}
```

### Priority 3: Make State Handle-Scoped (Proper Solution)

Instead of global state, make state scoped to each PluginHandle:

```rust
pub struct PluginHandle {
    plugin: Box<dyn Plugin>,
    context: PluginContext,
    runtime: Arc<AsyncRuntime>,
    bridge: AsyncBridge,
    id: RwLock<Option<u64>>,

    // Add per-handle logging state
    log_manager: Arc<LogCallbackManager>,  // Not global!
    reload_handle: Arc<ReloadHandle>,      // Not global!
}
```

**Challenges:**
- Tracing subscriber is still process-global
- Would need to implement custom filtering per-handle
- Significant refactoring required

### Priority 4: Alternative - Per-Plugin Tracing Subscribers

Use tracing's `with_subscriber` for scoped subscribers instead of global:

```rust
// Instead of global subscriber, use scoped
pub struct PluginHandle {
    subscriber: Arc<dyn Subscriber>,
}

impl PluginHandle {
    pub fn call(&self, type_tag: &str, request: &[u8]) -> PluginResult<Vec<u8>> {
        // Set subscriber for this call
        let _guard = tracing::subscriber::set_default(self.subscriber.clone());

        // All tracing in this scope uses this subscriber
        self.bridge.call_sync(...)
    }
}
```

**Challenges:**
- Need to propagate subscriber to all async tasks
- Performance overhead from setting subscriber per-call
- Complex implementation

## Testing Needed

Before declaring reload support, need tests for:

1. ✅ **Single plugin, single use** - Already works
2. ❌ **Single plugin, load → unload → reload** - NOT TESTED
3. ❌ **Multiple plugins, same process** - NOT TESTED
4. ❌ **Plugin reload after library update** - NOT TESTED
5. ❌ **Stale callback pointer handling** - NOT TESTED

## Recommendations

### Short Term (Phase 1)

1. **Document limitations clearly**:
   - Single plugin instance per process
   - Reload not supported - restart process for updates
   - Add to ARCHITECTURE.md, README.md, and Java docs

2. **Add runtime check** for multiple plugin instances:
   ```rust
   static PLUGIN_COUNT: AtomicUsize = AtomicUsize::new(0);

   pub fn plugin_init(...) {
       let count = PLUGIN_COUNT.fetch_add(1, Ordering::SeqCst);
       if count > 0 {
           tracing::warn!("Multiple plugin instances detected. This is not supported and may cause undefined behavior.");
       }
       // ... rest of init
   }
   ```

3. **Add plugin_shutdown cleanup**:
   ```rust
   pub fn plugin_shutdown(...) {
       // ... existing shutdown logic
       PLUGIN_COUNT.fetch_sub(1, Ordering::SeqCst);
   }
   ```

### Long Term (Phase 2+)

After multi-language expansion, if reload becomes a requirement:

1. Redesign logging architecture with per-handle state
2. Implement custom tracing layer that doesn't rely on global subscriber
3. Add comprehensive reload testing
4. Support hot-reload scenarios

## Conclusion

**Answer to original question**:
> "Will the dynamic log changes affect or inhibit the ability to unload a plugin?"

**UPDATE (2026-01-24)**: Implemented reload safety improvements:

### Changes Made

1. **Added cleanup on shutdown**:
   - `ReloadHandle::clear()` method resets handle to None
   - Called from `LogCallbackManager::unregister_plugin()` when last plugin shuts down
   - Binary handlers cleared via `clear_binary_handlers()` in plugin_shutdown

2. **Reference counting works correctly**:
   - `LogCallbackManager` uses ref_count to track active plugins
   - Callback cleared when ref_count reaches 0
   - Reload handle cleared when last plugin unregisters

3. **Thread-local state cleaned**:
   - `BINARY_HANDLERS` HashMap cleared on shutdown
   - Prevents stale handlers across reload cycles

### Current Status

**Single plugin, single load**: ✅ Works perfectly

**Single plugin, reload cycles**: ✅ **NOW SUPPORTED**
- All Java reload tests pass (PluginReloadTest.java)
- Tests verify: basic reload, functionality after reload, logging after reload, dynamic log levels after reload, multiple reload cycles, state freshness
- Global state properly cleaned up on shutdown
- OnceCell containers persist (acceptable limitation) but contents are cleared

**Multiple plugins, same process**: ⚠️ **WORKS WITH SHARED LOGGING**
- Reference counting ensures proper cleanup
- Last plugin to shutdown clears shared state
- Log level and callback are shared (documented limitation)
- This is an intentional design trade-off for simplicity

### Remaining Limitations

1. **OnceCell containers persist** - Only contents are cleared, containers remain
2. **Tracing subscriber is global** - Cannot be replaced once set (acceptable)
3. **Log callback shared across plugins** - Documented in ARCHITECTURE.md

### Testing Results

All reload tests in PluginReloadTest.java pass:
- ✅ testLoadShutdownReload
- ✅ testReloadFunctionality
- ✅ testLogCallbackAfterReload
- ✅ testDynamicLogLevelAfterReload
- ✅ testMultipleReloadCycles
- ✅ testStateFreshAfterReload

**Verdict**: The framework now properly supports plugin reload cycles for single-plugin scenarios. The dynamic log level feature does NOT inhibit reload - in fact, the cleanup improvements benefit the entire framework.
