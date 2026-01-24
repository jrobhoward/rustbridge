# rustbridge Tasks & Roadmap

This document tracks incomplete tasks and priorities for the rustbridge project.

## Current Focus: Kotlin-First Migration & Core Stability

**Objective**: Migrate Java/Kotlin ecosystem to Kotlin-first architecture with Java facade, finalize core framework stability, and prepare for multi-language expansion.

---

## Priority 1: Critical Stability

### Plugin Lifecycle & Resource Management

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Plugin reload/unload safety testing | High | 3-5 days | Test unload/reload cycles, verify no leaks, document limitations |
| Safe global state patterns | High | 1-2 days | Audit global state, add reset methods, document patterns in SKILLS.md |
| Request concurrency limits (backpressure) | High | 1-2 days | Implement semaphore-based request limiting with configurable max concurrent |
| Dynamic log level verification | Medium | 1 day | End-to-end test from Java/Kotlin, verify level changes affect subsequent logs |

### Technical Debt (Java/Kotlin Related)

| Task | Priority | Notes |
|------|----------|-------|
| Missing doc comments on public APIs | Medium | Document all Java/Kotlin public APIs |
| Error message quality improvements | Low | Improve actionable error messages |

---

## ✅ Jackson Migration (Completed)

**Status**: COMPLETED
**Date**: 2026-01-24

Migrated rustbridge-java from Gson 2.10.1 to Jackson 2.18.2 with jackson-module-kotlin for better Java/Kotlin interop. This was a drop-in replacement with no API changes.

**Changes**:
- Updated `rustbridge-core` dependency from Gson to Jackson
- Migrated all serialization code in `ResponseEnvelope`, `PluginConfig`, and `BundleLoader`
- Updated `FfmPlugin` and `JniPlugin` implementations
- All tests pass with no API changes
- Maintained Java 8+ compatibility

**Rationale**: Jackson provides better Java/Kotlin interop, active development, and is the industry standard for JSON serialization in Java ecosystems.

---

## Priority 3: Developer Experience & Ergonomics

### Plugin Development Improvements

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Plugin initialization parameters | Medium | 1-2 days | Add `init_params: Option<serde_json::Value>` to PluginConfig (Option C from scope) |
| Automatic handler dispatch macro | Medium | 3-5 days | Enhance `#[rustbridge_handler]` to auto-generate Plugin trait implementation |
| PluginConfig builder pattern | Medium | 1-2 days | Add fluent `PluginConfig::builder()` API for ergonomic testing |
| Gradle code generation integration | Medium | 3-5 days | Create rustbridge-gradle-plugin with auto-generation task |

### Documentation & Guides

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Getting Started guide | Medium | 3-5 days | Step-by-step tutorial: create plugin → add types → test → bundle |
| Error handling patterns guide | Medium | 1-2 days | Document error handling best practices with examples |
| Debugging guide | Medium | 1-2 days | Rust debugging (GDB), Java FFM debugging, troubleshooting |
| Plugin lifecycle best practices | Medium | 1-2 days | Document reload patterns, global state avoidance, resource cleanup |

---

## Priority 4: Observability & Resource Monitoring

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Memory consumption tracking | Low | 3-5 days | Implement TrackingAllocator, expose via FFI and Java API (optional feature) |
| CPU/task metrics from Tokio | Low | 1-2 days | Expose RuntimeMonitor stats via FFI (optional dependency on tokio-metrics) |
| Memory-based backpressure | Low | 1-2 days | Reject requests when heap usage exceeds threshold (requires memory tracking) |

---

## Priority 5: Non-JVM Language Support (Future)

### C# Bindings (Pending: Kotlin-first completion)

| Task | Priority | Notes |
|------|----------|-------|
| C# FFI bindings | Low | Follow Java FFM patterns, implement after Kotlin migration stabilizes |
| C# struct mapping for binary transport | Low | Follow Java BinaryStruct pattern |
| Port BundleLoader to C# | Low | Leverage Kotlin implementation patterns |
| Port MinisignVerifier to C# | Low | Leverage Kotlin implementation patterns |

### Python Bindings (Pending: User Decision)

| Task | Priority | Notes |
|------|----------|-------|
| Python bindings | Low | User decision needed on priority and architecture |

---

## Priority 6: General Improvements

### Code Quality

| Task | Priority | Notes |
|------|----------|-------|
| Clean up unused code warnings | Low | Dead code in runtime, logging crates |
| JNI native implementation | Low | FFM is preferred, JNI for legacy Java 8 compatibility only |
| Async API (plugin_call_async) | Low | Deferred: not critical for current use cases |

---

## Deferred Tasks

These tasks are explicitly deferred pending user requirements:

| Task | Original Context | Status |
|------|------------------|--------|
| Java JMH benchmark harness | Benchmark infrastructure | Rust benchmarks sufficient |
| Memory profiling setup | Benchmark infrastructure | Not needed for current decision |
| Latency distribution analysis | Benchmark infrastructure | Mean values sufficient |
| RbArray, RbOptional types | Binary transport | Not needed yet |
| CStructCodec implementation | Binary transport | Direct handler approach used instead |

---

## Recently Completed

### Log Callback Safety Fixes (2026-01-24)

**What was done:**
- Fixed RwLock deadlock in `LogCallbackManager::unregister_plugin()` when log level was DEBUG
- Fixed use-after-free crash when multiple plugins used callbacks and one shut down
- Updated callback manager to clear callback on ANY plugin shutdown (safety over convenience)
- Updated `SharedCallbackMultiPluginTest` to verify new safer behavior

**Root causes fixed:**
1. **Deadlock**: `tracing::debug!()` was called while holding a write lock, causing the logging layer to try to acquire a read lock on the same RwLock
2. **Use-after-free**: With multiple plugins, the callback from the last-registered plugin could become invalid when its arena closed, but was still stored in the global manager

**Behavioral change:**
- Callbacks are now per-plugin, not shared across multiple plugins
- When any plugin shuts down, the callback is cleared to prevent use-after-free
- Other active plugins continue to work, but without logging until a new callback is registered

**Files changed:**
- `crates/rustbridge-logging/src/callback.rs`
- `rustbridge-java/rustbridge-ffm/src/test/java/com/rustbridge/ffm/SharedCallbackMultiPluginTest.java`

---

### FfmPlugin Thread-Safe Concurrent Access (2026-01-24)

**What was done:**
- Refactored FfmPlugin to use per-call arenas instead of a single plugin-lifetime arena
- Removed `synchronized` keywords from `call()` and `callRaw()` methods
- Changed all arenas from `Arena.ofConfined()` to `Arena.ofShared()` to support concurrent access
- Enabled and verified concurrent test (100/100 calls succeed concurrently)
- Added comprehensive javadoc explaining thread safety and arena architecture

**Impact:**
- True concurrent execution - no more serialization of plugin calls
- Unlocks full performance potential (Rust side is already thread-safe with Arc, DashMap, RwLock)
- Each thread uses its own arena for temporary allocations - no contention
- Plugin handle in shared arena allows safe concurrent access

**Files changed:**
- `rustbridge-java/rustbridge-ffm/src/main/java/com/rustbridge/ffm/FfmPlugin.java`
- `rustbridge-java/rustbridge-ffm/src/main/java/com/rustbridge/ffm/FfmPluginLoader.java`
- `rustbridge-java/rustbridge-ffm/src/test/java/com/rustbridge/ffm/HelloPluginIntegrationTest.java`

---

## Kotlin-First Architecture Decision

**Rationale:**
- `kotlinx.serialization` is more idiomatic and performant than Gson for modern Kotlin
- Kotlin compiles to JVM bytecode compatible with Java 8+ (supporting existing MSRV)
- Java facades enable ergonomic Java API while leveraging Kotlin's advantages
- Reduces boilerplate and improves type safety for core serialization logic
- Positions framework as Kotlin-primary, with Java as a supported secondary consumer

**Migration Strategy:**
1. Start with serialization layer (highest ROI, lowest risk)
2. Add Java facade layer to maintain existing API
3. Verify Java 8+ compatibility
4. Migrate transport and utilities incrementally
5. Maintain green tests throughout (no breaking changes to Java API)

**Key Constraints:**
- MSRV stays 1.85.0 (Rust 2024)
- Java tests must pass without modification (ensures backward compatibility)
- No changes to FFI contract or C ABI
- Gradle build must remain compatible

---

## Next Up (Recommended Priority)

### Phase 1: Critical Stability (weeks 1-2)
1. Plugin reload/unload safety testing
2. Safe global state patterns
3. Request concurrency limits (backpressure)
4. Dynamic log level verification

### Phase 2: Kotlin-First Foundation (weeks 2-4)
5. Evaluate kotlinx.serialization viability
6. Set up Kotlin compiler in build
7. Migrate PluginConfig serialization
8. Create Java facades + test with existing suite

### Phase 3: Developer Experience (weeks 4-5)
9. Plugin initialization parameters
10. Documentation overhaul (Getting Started, Error Handling, Debugging)
11. Macro improvements

