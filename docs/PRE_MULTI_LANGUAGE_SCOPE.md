# Pre-Multi-Language Expansion Scope

**Status**: Planning
**Last Updated**: 2026-01-24

This document outlines the remaining scope of work to finalize before expanding rustbridge support beyond JVM languages. The goal is to ensure the core framework is robust, ergonomic, and feature-complete for the JVM implementation, which will serve as the reference for other language bindings.

---

## 1. Plugin Initialization Ergonomics

### 1.1 Optional Plugin Startup Parameters

**Current State:**
- Plugins are created via `plugin_create()` which calls a factory function (e.g., `MyPlugin::default()`)
- Only `PluginConfig` (JSON) is passed to `plugin_init()`
- No way to pass compile-time or constant parameters during plugin instantiation

**Proposed Enhancement:**
- Allow plugins to be initialized with custom parameters beyond the JSON config
- Support both default construction and parameterized construction

**Options to Consider:**

**Option A: Extend `plugin_create` with parameters**
```rust
// Allow factory function to accept parameters
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_create(
    params: *const u8,
    params_len: usize
) -> *mut std::ffi::c_void {
    let plugin = if params.is_null() {
        MyPlugin::default()
    } else {
        let params_data = unsafe { std::slice::from_raw_parts(params, params_len) };
        MyPlugin::from_params(params_data)
    };
    Box::into_raw(Box::new(plugin)) as *mut std::ffi::c_void
}
```

**Option B: Multiple factory functions**
```rust
// Export multiple factory functions
rustbridge_entry!(MyPlugin::default);         // Default factory
rustbridge_entry!(MyPlugin::with_params, "plugin_create_with_params");  // Parameterized factory
```

**Option C: Initialization parameters in PluginConfig**
```rust
// Add a generic "init_params" field to PluginConfig
pub struct PluginConfig {
    // ... existing fields ...
    pub init_params: Option<serde_json::Value>,  // Plugin-specific init data
}

// Plugin parses its own init params
impl Plugin for MyPlugin {
    async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
        if let Some(params) = &ctx.config.init_params {
            // Plugin-specific initialization
        }
        Ok(())
    }
}
```

**Recommendation**: Option C (init_params in PluginConfig) is the most flexible and requires no FFI changes. Option A would require FFI changes and complicates the API.

**Tasks:**
- [ ] Add `init_params: Option<serde_json::Value>` to `PluginConfig`
- [ ] Update `PluginConfig` deserialization to support the new field
- [ ] Document initialization patterns in plugin guide
- [ ] Add example showing parameterized plugin initialization
- [ ] Test with hello-plugin example

**Priority**: Medium
**Effort**: Small (1-2 days)

---

## 2. Dynamic Log Level Management

### 2.1 Runtime Log Level Changes

**Current State:**
- `plugin_set_log_level` FFI function exists (exports.rs:279)
- Exposed in Java `Plugin` interface (Plugin.java:47)
- Implementation calls `LogCallbackManager::global().set_level(level)` (handle.rs:204)

**Verification Needed:**
- End-to-end test confirming log level changes work at runtime
- Test from both Java and Kotlin
- Verify log level changes affect subsequent log statements immediately

**Tasks:**
- [ ] Add Java/Kotlin integration test for dynamic log level changes
  - Start plugin with INFO level
  - Verify DEBUG logs are not emitted
  - Change to DEBUG level via `setLogLevel()`
  - Verify DEBUG logs are now emitted
  - Change to ERROR level
  - Verify INFO logs are no longer emitted
- [ ] Document log level management in Java/Kotlin guides
- [ ] Add Rust-side test for LogCallbackManager level changes

**Priority**: Medium
**Effort**: Small (1 day)

---

## 3. Plugin Development Ergonomics

### 3.1 Macro Improvements

**Current State:**
- `rustbridge_entry!` macro simplifies FFI boilerplate
- `#[derive(Message)]` adds type tag metadata
- `#[rustbridge_plugin]` and `#[rustbridge_handler]` exist but are not fully utilized
- Manual dispatch in `handle_request` is verbose

**Proposed Enhancements:**

**A. Automatic Handler Dispatch Macro**
```rust
#[rustbridge_plugin]
impl MyPlugin {
    #[rustbridge_handler("echo")]
    async fn echo(&self, ctx: &PluginContext, req: EchoRequest) -> PluginResult<EchoResponse> {
        Ok(EchoResponse { message: req.message })
    }

    #[rustbridge_handler("greet")]
    async fn greet(&self, ctx: &PluginContext, req: GreetRequest) -> PluginResult<GreetResponse> {
        Ok(GreetResponse { greeting: format!("Hello, {}!", req.name) })
    }
}

// Macro generates the Plugin::handle_request implementation with automatic dispatch
```

**B. Builder Pattern for PluginConfig**
```rust
// Simplify plugin testing
let config = PluginConfig::builder()
    .worker_threads(4)
    .log_level(LogLevel::Debug)
    .init_param("api_key", "test-key")
    .build();
```

**Tasks:**
- [ ] Enhance `#[rustbridge_plugin]` to generate Plugin trait implementation
- [ ] Implement automatic deserialization and handler dispatch in macro
- [ ] Add `PluginConfig::builder()` with fluent API
- [ ] Update hello-plugin to demonstrate macro usage
- [ ] Document macro patterns in SKILLS.md

**Priority**: Medium
**Effort**: Medium (3-5 days)

### 3.2 Java/Kotlin Code Generation Gradle Integration

**Current State:**
- CLI has `rustbridge generate json-schema` and `rustbridge generate java`
- Code generation is manual, not integrated into build
- No Gradle task to auto-generate Java classes from Rust types

**Proposed Enhancement:**
```gradle
// In plugin's build.gradle.kts
plugins {
    id("com.rustbridge.codegen")
}

rustbridge {
    plugin {
        name = "hello-plugin"
        rustSource = file("../rust/src/messages.rs")
    }

    codegen {
        outputDir = file("src/main/java")
        packageName = "com.example.hello.messages"
    }
}
```

**Tasks:**
- [ ] Create Gradle plugin: `rustbridge-gradle-plugin`
- [ ] Implement `RustbridgeCodegenTask` that invokes CLI
- [ ] Add source sets for generated code
- [ ] Integrate with Gradle build lifecycle (generate before compileJava)
- [ ] Document Gradle integration in CODE_GENERATION.md
- [ ] Add example project using Gradle plugin

**Priority**: Medium
**Effort**: Medium (3-5 days)

### 3.3 Documentation Improvements

**Current State:**
- ARCHITECTURE.md, SKILLS.md, TESTING.md, CODE_GENERATION.md exist
- Examples in examples/hello-plugin and examples/kotlin-examples
- CLI commands documented in CLAUDE.md and README.md

**Gaps:**
- No step-by-step "Getting Started" tutorial
- Limited documentation on error handling patterns
- No guide for debugging plugins
- Missing best practices for lifecycle management
- No documentation on plugin reload/restart patterns

**Tasks:**
- [ ] Create `docs/GETTING_STARTED.md` with step-by-step tutorial
  - Create new plugin project
  - Add message types
  - Implement handlers
  - Test from Java/Kotlin
  - Build and bundle
- [ ] Create `docs/ERROR_HANDLING.md` with patterns and best practices
- [ ] Create `docs/DEBUGGING.md` with troubleshooting guide
  - Debugging Rust plugin (GDB, lldb)
  - Java FFM debugging
  - Common issues and solutions
- [ ] Expand CLAUDE.md with plugin development patterns
- [ ] Add inline documentation to hello-plugin explaining each component

**Priority**: Medium
**Effort**: Medium (3-5 days)

---

## 4. Resource Monitoring & Observability

### 4.1 Memory Consumption Tracking

**Current State:**
- No visibility into plugin memory usage
- Rust allocations happen via standard allocator
- No per-plugin memory accounting

**Proposed Enhancement:**
- Optional custom allocator to track heap usage
- Query API for current/peak memory consumption
- Expose via FFI and Java API

**Design:**

```rust
// Optional feature flag: "memory-tracking"
#[cfg(feature = "memory-tracking")]
pub struct TrackingAllocator {
    inner: System,
    bytes_allocated: AtomicUsize,
    bytes_deallocated: AtomicUsize,
    peak_bytes: AtomicUsize,
}

#[global_allocator]
#[cfg(feature = "memory-tracking")]
static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

// FFI function to query memory stats
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_get_memory_stats(
    handle: FfiPluginHandle
) -> MemoryStats {
    // Return current/peak heap bytes
}

// Rust API
impl PluginHandle {
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            heap_bytes: ALLOCATOR.current_usage(),
            peak_heap_bytes: ALLOCATOR.peak_usage(),
        }
    }
}
```

**Limitations:**
- Only tracks heap allocations via Rust allocator
- Does not track stack usage
- Does not track memory used by Tokio runtime internals
- Adds small overhead to all allocations

**Tasks:**
- [ ] Implement `TrackingAllocator` in new crate: `rustbridge-allocator`
- [ ] Add `plugin_get_memory_stats` FFI function
- [ ] Expose `getMemoryStats()` in Java Plugin interface
- [ ] Add feature flag `memory-tracking` (off by default)
- [ ] Document memory tracking in ARCHITECTURE.md
- [ ] Add example showing memory monitoring

**Priority**: Low (nice-to-have for diagnostics)
**Effort**: Medium (3-5 days)

### 4.2 CPU/Task Metrics from Tokio

**Current State:**
- Tokio runtime runs plugin tasks
- No visibility into task counts, pending tasks, or CPU usage

**Proposed Enhancement:**
- Expose Tokio metrics via `tokio-metrics` crate (if enabled)
- Query API for task statistics

**Design:**

```rust
// Optional dependency: tokio-metrics
#[cfg(feature = "runtime-metrics")]
use tokio_metrics::RuntimeMonitor;

pub struct RuntimeStats {
    pub active_tasks: usize,
    pub total_tasks_spawned: u64,
    pub workers: usize,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_get_runtime_stats(
    handle: FfiPluginHandle
) -> RuntimeStats {
    // Return Tokio runtime metrics
}
```

**Tasks:**
- [ ] Add optional `tokio-metrics` dependency
- [ ] Wrap RuntimeMonitor in AsyncRuntime
- [ ] Add `plugin_get_runtime_stats` FFI function
- [ ] Expose `getRuntimeStats()` in Java Plugin interface
- [ ] Add feature flag `runtime-metrics` (off by default)
- [ ] Document runtime metrics in ARCHITECTURE.md

**Priority**: Low (nice-to-have for diagnostics)
**Effort**: Small (1-2 days)

---

## 5. Backpressure & Resource Limits

### 5.1 Request Concurrency Limits

**Current State:**
- Java can call plugin concurrently (thread-safe after recent changes)
- No limit on concurrent in-flight requests
- If host spawns too many tasks, could exhaust memory
- No mechanism to reject requests when under load

**Proposed Enhancement:**
- Add optional semaphore-based request limiting
- Configurable max concurrent requests
- Return error when limit exceeded (backpressure)

**Design:**

```rust
pub struct PluginConfig {
    // ... existing fields ...
    pub max_concurrent_requests: Option<usize>,  // None = unlimited
}

pub struct PluginHandle {
    // ... existing fields ...
    request_semaphore: Option<Arc<tokio::sync::Semaphore>>,
}

impl PluginHandle {
    pub fn call(&self, type_tag: &str, request: &[u8]) -> PluginResult<Vec<u8>> {
        // Try to acquire permit
        if let Some(sem) = &self.request_semaphore {
            let permit = sem.try_acquire()
                .map_err(|_| PluginError::TooManyRequests)?;

            // Call with permit held (will be released on drop)
            let result = self.bridge.call_sync(
                self.plugin.handle_request(&self.context, type_tag, request)
            );

            drop(permit);
            result
        } else {
            // No limit, call directly
            self.bridge.call_sync(
                self.plugin.handle_request(&self.context, type_tag, request)
            )
        }
    }
}

// New error variant
pub enum PluginError {
    // ... existing variants ...
    TooManyRequests,  // Error code 13
}
```

**Tasks:**
- [ ] Add `max_concurrent_requests` to PluginConfig
- [ ] Add semaphore to PluginHandle when configured
- [ ] Implement try_acquire in call path
- [ ] Add `TooManyRequests` error variant (code 13)
- [ ] Map to HTTP 429 / appropriate error in Java
- [ ] Add test for concurrency limiting
- [ ] Document backpressure configuration

**Priority**: Medium (important for production robustness)
**Effort**: Small (1-2 days)

### 5.2 Memory-Based Backpressure

**Current State:**
- No memory-based backpressure
- Could run out of memory if too many large requests

**Proposed Enhancement:**
- Monitor memory usage (requires memory tracking from 4.1)
- Reject requests when memory exceeds threshold
- Optional: sleep/retry when near threshold

**Design:**

```rust
pub struct PluginConfig {
    // ... existing fields ...
    pub max_heap_bytes: Option<usize>,  // None = unlimited
    pub heap_threshold_for_backpressure: Option<f32>,  // e.g., 0.9 = 90%
}

impl PluginHandle {
    pub fn call(&self, type_tag: &str, request: &[u8]) -> PluginResult<Vec<u8>> {
        // Check memory before accepting request
        if let Some(max_heap) = self.context.config.max_heap_bytes {
            let current = ALLOCATOR.current_usage();
            let threshold = self.context.config.heap_threshold_for_backpressure
                .unwrap_or(0.9);

            if current > (max_heap as f32 * threshold) as usize {
                return Err(PluginError::OutOfMemory);
            }
        }

        // ... proceed with request ...
    }
}
```

**Tasks:**
- [ ] Add memory threshold config options
- [ ] Implement memory checking in call path
- [ ] Add `OutOfMemory` error variant (code 14)
- [ ] Add test for memory-based backpressure
- [ ] Document memory limits configuration

**Priority**: Low (requires memory tracking first)
**Effort**: Small (1-2 days, after 4.1)
**Dependency**: Requires 4.1 (Memory Tracking)

---

## 6. Plugin Reload/Unload Safety

### 6.1 Unload and Reload Support

**Current State:**
- `plugin_shutdown()` exists and cleanly shuts down plugin
- Handle is removed from PluginHandleManager
- Rust runtime is shut down
- No explicit testing of unload/reload cycles

**Potential Issues:**
- **Global state**: If plugin uses `static` or `lazy_static!`, state may persist across reloads
- **Thread-local state**: May not be cleaned up properly
- **FFI callbacks**: Log callback is global, may point to freed memory after host unloads
- **Dynamic library unloading**: Some platforms cache symbols

**Investigation Needed:**
1. Can we unload and reload the same .so/.dll/.dylib file?
2. Does dlclose() actually unload the library or is it cached?
3. Do Rust statics get reinitialized on reload?
4. Are there memory leaks on reload?

**Tasks:**

**Phase 1: Analysis**
- [ ] Document static/global state in rustbridge crates
  - `HANDLE_MANAGER` (OnceCell) in handle.rs
  - `LogCallbackManager` (OnceCell) in logging crate
  - `BINARY_HANDLERS` (thread_local) in exports.rs
- [ ] Review Rust std library behavior on library reload
- [ ] Research platform-specific behavior (Linux, macOS, Windows)
- [ ] Document limitations in ARCHITECTURE.md

**Phase 2: Testing**
- [ ] Add Java test: load → shutdown → reload same plugin
- [ ] Add Java test: load → call → shutdown → reload → call
- [ ] Add Java test: concurrent load/unload cycles
- [ ] Test with Valgrind/AddressSanitizer for leaks
- [ ] Test on Linux, macOS, Windows

**Phase 3: Documentation**
- [ ] Document reload safety guarantees in ARCHITECTURE.md
- [ ] Document known limitations (if any)
- [ ] Provide guidelines for plugin authors:
  - Avoid `static mut`
  - Use `OnceCell` carefully
  - Clean up resources in `on_stop()`
  - Don't rely on static state across reloads

**Priority**: High (critical for long-running applications)
**Effort**: Medium (3-5 days)

### 6.2 Safe Global State Patterns

**Current State:**
- Framework uses `OnceCell` for global managers
- These are NOT reset on plugin unload

**Proposed Enhancement:**
- Ensure all global state is handle-scoped or cleaned up on shutdown
- Provide guidance on avoiding problematic global state

**Design Patterns:**

```rust
// GOOD: State scoped to plugin handle
pub struct MyPlugin {
    state: Arc<RwLock<PluginState>>,
}

// GOOD: Reset global state on shutdown
static MANAGER: OnceCell<Manager> = OnceCell::new();

impl Plugin for MyPlugin {
    async fn on_stop(&self, ctx: &PluginContext) -> PluginResult<()> {
        // Reset global state
        MANAGER.take();  // Clear OnceCell
        Ok(())
    }
}

// BAD: Non-resettable global state
static mut COUNTER: AtomicUsize = AtomicUsize::new(0);  // Persists across reloads!
```

**Tasks:**
- [ ] Audit rustbridge crates for global state
- [ ] Add reset methods where needed (e.g., `LogCallbackManager::reset()`)
- [ ] Call reset methods in `plugin_shutdown` before handle removal
- [ ] Document global state patterns in SKILLS.md
- [ ] Add lints or warnings for `static mut` in plugins

**Priority**: High
**Effort**: Small (1-2 days)

---

## 7. Additional Features to Consider

### 7.1 Async API (`plugin_call_async`)

**Current State:**
- Placeholder FFI function exists (exports.rs:496)
- Returns 0 (not implemented)
- Would allow non-blocking calls from host

**Design Considerations:**
- Callback-based or future-based?
- Cancellation support?
- How to handle completion from different threads?
- Java FFM integration strategy?

**Tasks:**
- [ ] Design async API surface
- [ ] Implement request ID management
- [ ] Implement callback storage and invocation
- [ ] Add cancellation support
- [ ] Integrate with Java CompletableFuture
- [ ] Add tests for async paths
- [ ] Document async API usage

**Priority**: Low (current sync API is sufficient)
**Effort**: Large (1-2 weeks)
**Status**: Deferred pending user requirements

### 7.2 Streaming/Chunked Responses

**Current State:**
- Request/response model is fully buffered
- Large responses must fit in memory

**Use Case:**
- Streaming large files
- Server-sent events
- Incremental results

**Tasks:**
- [ ] Design streaming API (callback-based chunks?)
- [ ] Implement chunk buffering and flow control
- [ ] Add FFI functions for streaming
- [ ] Integrate with Java InputStreams
- [ ] Document streaming patterns

**Priority**: Low (not a current requirement)
**Effort**: Large (1-2 weeks)
**Status**: Deferred pending user requirements

### 7.3 Plugin-to-Plugin Communication

**Current State:**
- Plugins are isolated
- No mechanism for inter-plugin communication

**Use Case:**
- Shared services (caching, auth)
- Event bus between plugins
- Dependency injection

**Tasks:**
- [ ] Design plugin registry and discovery
- [ ] Design inter-plugin messaging protocol
- [ ] Implement isolation/permissions
- [ ] Document plugin composition patterns

**Priority**: Low (single-plugin model is sufficient)
**Effort**: Large (2-3 weeks)
**Status**: Deferred pending user requirements

---

## 8. Pre-Multi-Language Checklist

Before expanding to C#, Python, and other languages, ensure:

### 8.1 Core Framework Stability
- [x] Thread-safe concurrent plugin calls (completed 2026-01-24)
- [ ] Dynamic log level changes verified end-to-end
- [ ] Plugin reload/unload tested and documented
- [ ] Backpressure mechanism implemented and tested
- [ ] Memory/runtime monitoring (if prioritized)

### 8.2 API Stability
- [ ] FFI API considered stable (no breaking changes planned)
- [ ] Error codes finalized and documented
- [ ] PluginConfig schema stable
- [ ] Binary transport API stable (if used)

### 8.3 Developer Experience
- [ ] Plugin development macros improved
- [ ] Getting Started guide completed
- [ ] Error handling patterns documented
- [ ] Debugging guide completed
- [ ] Code generation integrated with build tools

### 8.4 Testing & Quality
- [ ] Integration tests covering all FFI functions
- [ ] Stress tests (high concurrency, memory pressure)
- [ ] Valgrind/ASan clean (no leaks)
- [ ] Cross-platform testing (Linux, macOS, Windows)
- [ ] Performance benchmarks documented

### 8.5 Documentation
- [ ] All public APIs documented
- [ ] Architecture diagrams updated
- [ ] Migration guide (if breaking changes)
- [ ] Known limitations documented
- [ ] Troubleshooting guide complete

---

## 9. Implementation Priority

**Priority 1: Critical for Stability**
1. Plugin reload/unload safety (6.1, 6.2)
2. Backpressure/concurrency limits (5.1)
3. Dynamic log level verification (2.1)

**Priority 2: Developer Experience**
4. Plugin initialization parameters (1.1)
5. Documentation improvements (3.3)
6. Macro improvements (3.1)

**Priority 3: Nice-to-Have**
7. Memory/runtime monitoring (4.1, 4.2)
8. Gradle integration (3.2)
9. Memory-based backpressure (5.2)

**Priority 4: Future Work**
10. Async API (7.1) - Deferred
11. Streaming responses (7.2) - Deferred
12. Plugin-to-plugin communication (7.3) - Deferred

---

## 10. Estimated Timeline

### Phase 1: Critical Stability (1-2 weeks)
- Plugin reload/unload testing and fixes
- Backpressure implementation
- Log level verification

### Phase 2: Ergonomics & Docs (1-2 weeks)
- Init parameters
- Documentation overhaul
- Macro improvements

### Phase 3: Observability (1 week, optional)
- Memory tracking
- Runtime metrics

**Total**: 3-5 weeks before multi-language expansion

---

## Next Steps

1. Review this plan with stakeholders
2. Prioritize based on immediate needs
3. Begin with Phase 1 (Critical Stability)
4. Create GitHub issues for tracking
5. Update TASKS.md with selected work items
