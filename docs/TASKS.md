# rustbridge Tasks & Roadmap

This document tracks incomplete tasks and priorities for the rustbridge project.

## Current Focus: Language Binding Feature Parity

**Objective**: Achieve full feature parity across all language bindings (Java FFM, Java JNI, C#, Python). Python bindings are now complete (JSON transport). Next: fill feature gaps and add comprehensive testing.

---

## Priority 0: Language Binding Feature Parity (Active)

### High Priority (Functionality Gaps)

| # | Task | Language | Status | Notes |
|---|------|----------|--------|-------|
| 1 | Add schema extraction to Python BundleLoader | Python | ✅ Done | Added `extract_schema()`, `read_schema()`, `get_schemas()` + 8 tests |
| 2 | Add concurrency tests to Python | Python | ✅ Done | Added 4 tests in `test_concurrency.py` |
| 3 | Add resource leak tests to Python | Python | ✅ Done | Added 7 tests in `test_resource_leak.py` |
| 4 | Add concurrency tests to C# | C# | ✅ Done | Added 4 tests in `ConcurrencyLimitTest.cs` |
| 5 | Add resource leak tests to C# | C# | ✅ Done | Added 7 tests in `ResourceLeakTest.cs` |

### Medium Priority (Nice to Have)

| # | Task | Language | Status | Notes |
|---|------|----------|--------|-------|
| 6 | Add host-language benchmarks | Java | ✅ Done | JMH benchmarks: latency, throughput, concurrency (FFM Binary: 1.8M ops/s, JNI Binary: 2.6M ops/s) |
| 7 | Add host-language benchmarks | C# | ✅ Done | BenchmarkDotNet: TransportBenchmark, ThroughputBenchmark, ConcurrentBenchmark |
| 8 | Add host-language benchmarks | Python | ✅ Done | Added pytest-benchmark tests for call latency, throughput |
| 9 | Add binary transport to Python | Python | ✅ Done | `call_raw()` with ctypes structs, 8 tests |

### Lower Priority

| # | Task | Language | Status | Notes |
|---|------|----------|--------|-------|
| 10 | Add binary transport to Java JNI | Java JNI | ✅ Done | `callRaw()`, `hasBinaryTransport()`, benchmark shows JNI Binary 20x faster than FFM Binary |
| 11 | Add Kotlin extension APIs | Kotlin | ✅ Done | DSL config, coroutine support, PluginResult, loader extensions (73 tests) |

---

---

## Priority 1: Technical Debt & Documentation

### Java/Kotlin API Quality

| Task | Priority | Notes |
|------|----------|-------|
| Missing doc comments on public APIs | Medium | Document all Java/Kotlin public APIs |
| Error message quality improvements | Low | Improve actionable error messages |

---

## Priority 2: Developer Experience & Ergonomics

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

## Priority 3: Observability & Resource Monitoring

| Task | Priority | Effort | Notes |
|------|----------|--------|-------|
| Memory consumption tracking | Low | 3-5 days | Implement TrackingAllocator, expose via FFI and Java API (optional feature) |
| CPU/task metrics from Tokio | Low | 1-2 days | Expose RuntimeMonitor stats via FFI (optional dependency on tokio-metrics) |
| Memory-based backpressure | Low | 1-2 days | Reject requests when heap usage exceeds threshold (requires memory tracking) |

---

## Priority 4: Non-JVM Language Support (Future)

### C# Bindings

| Task | Priority | Status | Notes |
|------|----------|--------|-------|
| C# FFI bindings | Low | ✅ Done | P/Invoke implementation complete, 76 tests passing |
| C# struct mapping for binary transport | Low | ✅ Done | Binary transport working with thread-safe handler registry |
| Port BundleLoader to C# | Low | ✅ Done | Supports ZIP extraction, checksums, minisign verification (20 tests) |
| Port MinisignVerifier to C# | Low | ✅ Done | Ed25519 verification using NSec.Cryptography (14 tests) |
| C# testing documentation | Low | ✅ Done | docs/TESTING_CSHARP.md |
| C# pre-commit integration | Low | ✅ Done | Added to scripts/pre-commit.sh and scripts/pre-commit.bat |
| C# CI/CD pipeline (GitHub Actions) | Low | Pending | Add .NET build/test to CI workflow, run on ubuntu/windows/macos |
| C# NuGet packaging | Low | Pending | Create .nupkg for RustBridge.Core and RustBridge.Native |

### Rust FFI Fixes Completed

| Task | Priority | Notes |
|------|----------|-------|
| ✅ Fix BINARY_HANDLERS thread-local storage | Medium | Fixed: Replaced `thread_local!` with `OnceCell<DashMap<...>>` in `crates/rustbridge-ffi/src/exports.rs`. Binary transport now works for all host languages. |
| ✅ Java binary transport tests | Medium | Added `BinaryTransportTest.java` with 4 tests for callRaw functionality. |

### Python Bindings (Pending: User Decision)

| Task | Priority | Notes |
|------|----------|-------|
| Python bindings | Low | User decision needed on priority and architecture |

---

## Priority 5: General Improvements

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

### ✅ Phase 1: Critical Stability (2026-01-24)

**Status**: COMPLETED
**Objective**: Finalize core framework stability and production-readiness

All critical stability tasks have been completed, making the framework production-ready for multi-language expansion.

#### Completed Tasks:

**1. Request Concurrency Limits (Backpressure) ✅**
- **Implementation**: Semaphore-based request limiting in `PluginHandle` (handle.rs:74-177)
- **Configuration**: `PluginConfig.max_concurrent_ops` (default: 1000, 0 = unlimited)
- **Error handling**: Returns `PluginError::TooManyRequests` when limit exceeded
- **Metrics**: Tracks rejected request count via `rejected_requests` counter
- **Tests**: Comprehensive test suite in `ConcurrencyLimitTest.java` (4 tests)
  - Verify limit enforcement with concurrent requests
  - Verify unlimited mode (0) allows all requests
  - Verify permits released after completion
  - Verify rejected count tracking

**2. Safe Global State Patterns ✅**
- **Documentation**: Added comprehensive section in `docs/SKILLS.md:470-569`
- **Coverage**: Documents all global state in the framework
  - HANDLE_MANAGER (plugin handles)
  - CALLBACK_MANAGER (FFI log callbacks)
  - BINARY_HANDLERS (binary message handlers)
  - ReloadHandle (tracing filter reload)
- **Patterns**: Provides GOOD/BAD examples with code
- **Guidelines**: Best practices for plugin developers

**3. Plugin Reload/Unload Safety Testing ✅**
- **Tests**: 12 comprehensive tests created across 3 test files
  - `MultiplePluginTest.java` (4 tests) - Multiple plugin scenarios
  - `PluginReloadTest.java` (6 tests) - Reload cycles
  - `ReloadLoggingVerificationTest.java` (2 tests) - Logging after reload
- **Findings**: Reload cycles work perfectly with clean resource cleanup
- **Bug fix**: Fixed critical stale callback crash (SIGSEGV)
- **Documentation**:
  - `RELOAD_TEST_RESULTS.md` - Detailed test results
  - `PLUGIN_RELOAD_STATUS.md` - User-facing status
  - `RELOAD_SAFETY_ANALYSIS.md` - Technical analysis
  - Updated `ARCHITECTURE.md` with limitations

**4. Dynamic Log Level Verification ✅**
- **Implementation**: Added `tracing_subscriber::reload` support in logging crate
- **Tests**: `DynamicLogLevelTest.java` verifies runtime level changes
  - Full level cycle (INFO → DEBUG → ERROR)
  - Immediate effect verification
- **Result**: Log levels can be changed dynamically without restart

#### Impact:

**Production-Ready**: The framework is now stable and ready for:
- ✅ Concurrent plugin calls with configurable backpressure
- ✅ Safe plugin reload cycles
- ✅ Dynamic log level changes at runtime
- ✅ Clean shutdown with proper resource cleanup
- ✅ Well-documented global state patterns

**Multi-Language Ready**: Core FFI API is stable and won't need breaking changes for C#, Python, or other language bindings.

**Documentation**: Comprehensive docs for reload safety, global state patterns, and testing.

See `docs/PHASE1_COMPLETION_SUMMARY.md` for complete details.

---

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

## Next Up (Recommended Priority)

**Phase 1 is complete!** Core framework is production-ready. Choose your next focus area:

### Option A: Developer Experience ✅ COMPLETED
1. ✅ **Documentation overhaul** (High value, approachable)
   - ✅ Getting Started guide
   - ✅ Error handling patterns guide
   - ✅ Debugging guide
   - ✅ Plugin lifecycle best practices
2. ✅ **Plugin initialization parameters** (ergonomics win)
3. ✅ **Missing doc comments on public APIs** (finish polish)

### Option B: Multi-Language Expansion (weeks 3-6)
1. **C# bindings** (Follow Java FFM patterns)
   - C# FFI bindings
   - Struct mapping for binary transport
   - Port BundleLoader to C#
   - Port MinisignVerifier to C#
2. **Python bindings** (High demand)
   - Python FFI bindings (ctypes/cffi)
   - Port BundleLoader to Python

**Recommendation**: Start with **Option B (Multi-Language Expansion)** to expand the framework's reach, or continue with other priorities from Priority 2-5 sections.

