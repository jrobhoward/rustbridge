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

## Priority 2: Kotlin-First Migration

**Objective**: Migrate core serialization and transport logic to Kotlin with `kotlinx.serialization`, maintaining Java 8+ compatibility via Java facades.

### 2.1 Serialization Layer

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Evaluate kotlinx.serialization viability | High | 1-2 days | Verify Kotlin→Java calling works, validate Java 8+ bytecode compatibility |
| Migrate PluginConfig serialization to Kotlin | High | 2-3 days | Replace Gson with kotlinx.serialization, create Java facade |
| Migrate message transport to Kotlin | High | 2-3 days | Move JSON serialization logic to Kotlin, test with existing Java tests |
| Create Java serialization facades | High | 1-2 days | Wrapper functions making Kotlin serialization ergonomic from Java |

### 2.2 Transport & Utilities

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Migrate BundleLoader to Kotlin | Medium | 2-3 days | Rewrite bundle handling in Kotlin, maintain Java API |
| Migrate MinisignVerifier to Kotlin | Medium | 1-2 days | Rewrite signature verification in Kotlin, maintain Java API |
| Consolidate JSON utilities in Kotlin | Medium | 1-2 days | Move JSON helpers to shared Kotlin module, Java facades |

### 2.3 Build & Integration

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Set up Kotlin compiler in build | Medium | 1 day | Add kotlinx.serialization dependency, Kotlin stdlib, verify MSRV compatibility |
| Integrate Kotlin tests (Kotest/JUnit) | Medium | 1-2 days | Add Kotlin test infrastructure, mirror existing test coverage |
| Document Kotlin-first architecture | Medium | 1-2 days | Update ARCHITECTURE.md explaining Kotlin core + Java facade pattern |

### 2.4 Validation & Compatibility

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Run full Java test suite against Kotlin changes | High | 1 day | Verify all existing Java tests pass without modification |
| Java 8 compatibility verification | High | 1 day | Verify compiled Kotlin bytecode targets Java 8, test on Java 8 runtime |
| Kotlin + Java interop edge cases | Medium | 1-2 days | Test null handling, optional parameters, error propagation |

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

