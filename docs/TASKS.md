# rustbridge Tasks & Roadmap

This document tracks incomplete tasks and priorities for the rustbridge project.

## Current Focus: Java/Kotlin Completion

**Objective**: Finalize Java/Kotlin support and address technical debt before expanding to other languages.

---

## Priority 1: Java/Kotlin Blockers

### Technical Debt (Java/Kotlin Related)

| Task | Priority | Notes |
|------|----------|-------|
| ~~FfmPlugin synchronization improvements~~ | ~~High~~ | ✅ **COMPLETED**: Implemented per-call arenas for true thread safety |
| Missing doc comments on public APIs | Medium | Document all Java/Kotlin public APIs |
| Error message quality improvements | Low | Improve actionable error messages |

### Optional Enhancements

| Task | Priority | Notes |
|------|----------|-------|
| JNI native implementation | Low | FFM is preferred, JNI for legacy Java 8 compatibility only |
| Async API (plugin_call_async) | Low | Not critical for current use cases |

---

## Priority 2: Non-JVM Language Support (Deprioritized)

### C# Bindings

| Task | Priority | Notes |
|------|----------|-------|
| C# FFI bindings | Low | Follow Java FFM patterns |
| C# struct mapping for binary transport | Low | Follow Java BinaryStruct pattern |
| Port BundleLoader to C# | Low | After Java refinement |
| Port MinisignVerifier to C# | Low | After Java refinement |

### Python Bindings

| Task | Priority | Notes |
|------|----------|-------|
| Python bindings | Low | User decision needed on priority |

---

## Priority 3: General Improvements

### Code Quality

| Task | Priority | Notes |
|------|----------|-------|
| Clean up unused code warnings | Low | Dead code in runtime, logging crates |

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

## Next Up

1. **Address Java/Kotlin technical debt** (Priority 1)
   - FfmPlugin synchronization improvements
   - Complete public API documentation

2. **User decision needed**: Evaluate need for non-JVM language support

