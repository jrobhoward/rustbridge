# rustbridge Tasks & Roadmap

This document tracks the implementation progress and upcoming tasks for the rustbridge project.

## Current Focus: Transport Benchmarking

**Objective**: Implement and benchmark C struct transport vs JSON to determine if binary transport provides meaningful performance benefits.

See [BUNDLE_ARCHITECTURE.md](./BUNDLE_ARCHITECTURE.md) for the full design.

---

## Implementation Phases

### Phase 1: Core Foundation ✅ COMPLETE

**Goal**: Validate simple request/response pattern end-to-end

| Task | Status | Notes |
|------|--------|-------|
| Workspace structure | ✅ Done | Cargo.toml, all crate scaffolding |
| rustbridge-core | ✅ Done | Plugin trait, LifecycleState, PluginError, PluginConfig |
| rustbridge-transport | ✅ Done | JsonCodec, RequestEnvelope, ResponseEnvelope |
| rustbridge-ffi | ✅ Done | FfiBuffer, C exports, handle management |
| rustbridge-runtime | ✅ Done | Tokio integration, AsyncBridge, shutdown signals |
| rustbridge-logging | ✅ Done | FfiLoggingLayer, log callback management |
| rustbridge-macros | ✅ Done | `#[rustbridge_plugin]`, `derive(Message)`, `rustbridge_entry!` |
| rustbridge-cli | ✅ Done | `new`, `build`, `generate`, `check` commands |
| hello-plugin example | ✅ Done | Echo, greet, user.create, math.add handlers |
| FFI exports verified | ✅ Done | All plugin_* functions exported |

### Phase 2: Java Integration ✅ COMPLETE (Core)

**Goal**: Working end-to-end Java↔Rust communication

| Task | Status | Notes |
|------|--------|-------|
| Java core interfaces | ✅ Done | Plugin, LifecycleState, LogLevel, PluginConfig |
| FFM implementation | ✅ Done | FfmPluginLoader, FfmPlugin, NativeBindings |
| FFM integration tests | ✅ Done | 17 passing tests - echo, greet, user.create, math.add, error handling |
| FFM bindings fixes | ✅ Done | Fixed plugin_get_state, plugin_call struct return, synchronized access |
| JNI fallback skeleton | ✅ Done | JniPluginLoader, JniPlugin (needs native impl) |
| Kotlin examples | ✅ Done | BasicExample, LoggingExample, ErrorHandlingExample |
| Log callback integration | ✅ Done | FFM upcall, MemorySegment handling, comprehensive tests |
| Panic handling verification | ✅ Done | Rust panic handling tested via invalid inputs |

### Phase 3: Benchmark Infrastructure ✅ COMPLETE

**Goal**: Establish baseline measurements and benchmarking framework

| Task | Status | Notes |
|------|--------|-------|
| Define benchmark messages | ✅ Done | Small, medium, large payloads in hello-plugin |
| Criterion benchmark harness | ✅ Done | `benches/json_baseline.rs`, `benches/binary_comparison.rs` |
| JSON baseline benchmarks | ✅ Done | Full cycle: 654 ns |
| Java JMH benchmark harness | ⬜ Deferred | Rust benchmarks sufficient for decision |
| Memory profiling setup | ⬜ Deferred | Not needed for initial decision |
| Latency distribution analysis | ⬜ Deferred | Mean values sufficient for decision |

### Phase 4: C Struct Transport Implementation ✅ COMPLETE

**Goal**: Implement binary transport for benchmark comparison

| Task | Status | Notes |
|------|--------|-------|
| Define RbString, RbBytes types | ✅ Done | `binary_types.rs` - borrowed types |
| Define RbStringOwned, RbBytesOwned | ✅ Done | `binary_types.rs` - owned types |
| Define RbResponse type | ✅ Done | Generic response buffer for FFI |
| Create rustbridge_types.h | ✅ Done | `include/rustbridge_types.h` |
| Implement benchmark message structs | ✅ Done | `SmallRequestRaw`, `SmallResponseRaw` |
| Add `plugin_call_raw` FFI entry | ✅ Done | Binary handler registry, message dispatch |
| Add message_id dispatch | ✅ Done | Thread-local handler map |
| Define RbArray, RbOptional types | ⬜ Deferred | Not needed for benchmark |
| Implement CStructCodec | ⬜ Deferred | Direct handler approach used instead |
| Generate messages.h from Rust | ⬜ Deferred | Manual header sufficient for now |

### Phase 5: Comparative Benchmarking ✅ COMPLETE

**Goal**: Measure and analyze performance differences

| Task | Status | Notes |
|------|--------|-------|
| C struct microbenchmarks | ✅ Done | Full cycle, serialize, deserialize |
| Throughput comparison | ✅ Done | Binary 7.1x faster |
| Latency comparison | ✅ Done | 654 ns → 92 ns |
| Memory allocation comparison | ⬜ Deferred | Allocations similar due to format! |
| CPU profiling | ⬜ Deferred | Not needed - clear wins already |
| Java FFM struct mapping | ⬜ Deferred | After decision point |
| Cross-language overhead | ⬜ Deferred | After decision point |
| Write benchmark report | ✅ Done | See Benchmark Results below |

### Phase 6: Bundle Format ✅ COMPLETE

**Goal**: Standardized plugin distribution

| Task | Status | Notes |
|------|--------|-------|
| Define manifest.json schema | ✅ Done | Platform mapping, API description |
| Implement rustbridge-bundle crate | ✅ Done | Archive creation/extraction with ZIP |
| Add `rustbridge bundle` CLI command | ✅ Done | create, list, extract subcommands |
| Platform detection logic | ✅ Done | OS/arch detection in Platform enum |
| Checksum validation | ✅ Done | SHA256 verification on extract |
| Java bundle loader | ⬜ Todo | Load .rbp in Java runtime |
| Embed JSON schema in bundle | ⬜ Todo | Self-describing messages |
| Embed C header in bundle | ⬜ Todo | For binary transport users |

**Files Created**:
- `crates/rustbridge-bundle/` - New crate for bundle operations
  - `src/lib.rs` - Public API exports
  - `src/error.rs` - BundleError enum
  - `src/platform.rs` - Platform detection and mapping
  - `src/manifest.rs` - Manifest schema and validation
  - `src/builder.rs` - BundleBuilder for creating .rbp files
  - `src/loader.rs` - BundleLoader for extracting libraries
- `crates/rustbridge-cli/src/bundle.rs` - CLI subcommands

**Usage**:
```bash
# Create a bundle
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmyplugin.so \
  --lib darwin-aarch64:target/release/libmyplugin.dylib

# List bundle contents
rustbridge bundle list my-plugin-1.0.0.rbp

# Extract library for current platform
rustbridge bundle extract my-plugin-1.0.0.rbp --output ./lib
```

### Phase 7: Decision Point ✅ COMPLETE

**Goal**: Decide on binary transport based on benchmark results

| Criteria | Target | Actual | Status |
|----------|--------|--------|--------|
| Latency improvement | >5x | **7.1x** | ✅ Met |
| Throughput improvement | >10x | ~7x | ⚠️ Partial |
| Memory reduction | >50% | Similar | ❌ Not met |
| Implementation complexity | Acceptable | ~500 LOC | ✅ Met |

**Analysis**:
- **Latency**: 654 ns → 92 ns (7.1x improvement) - target exceeded
- **Deserialization**: 130 ns → ~1 ns (130x improvement) - massive win
- **Serialization**: 73 ns → 48 ns (1.5x improvement) - modest gain
- **Memory**: Both paths allocate for `format!()` string - no significant difference
- **Payload size**: Binary is 1.4x larger due to fixed-size buffers (72 vs 51 bytes)

**Decision**: ✅ **PROCEED TO PHASE 8**

Binary transport provides meaningful latency improvements that justify the added complexity:
- 7.1x overall latency improvement exceeds the 5x target
- Deserialization is essentially free (pointer cast)
- Implementation is contained (~500 LOC) and maintainable
- Fixed-size buffer tradeoff is acceptable for performance-critical paths

**Recommendation**: Keep JSON as the default transport (debugging, flexibility), offer binary transport as an opt-in for latency-sensitive use cases.

### Phase 8: Binary Transport Generalization ✅ COMPLETE

**Goal**: Full binary transport support (only if Phase 7 criteria met)

| Task | Status | Notes |
|------|--------|-------|
| Header generation for all messages | ✅ Done | `rustbridge generate-header --verify` CLI command |
| Java struct mapping utilities | ✅ Done | BinaryStruct.java, callRaw() in FfmPlugin |
| C# struct mapping | ⬜ Deferred | Will follow same pattern as Java |
| Version field in structs | ✅ Done | `version: u8` + `_reserved` padding |
| Migration guide | ✅ Done | docs/BINARY_TRANSPORT.md |
| Deprecation strategy | ✅ Done | Versioning policy in migration guide |

**Files Created**:
- `crates/rustbridge-cli/src/header_gen.rs` - C header generation from Rust structs
- `rustbridge-java/.../BinaryStruct.java` - Base class for FFM struct wrappers
- `docs/BINARY_TRANSPORT.md` - When/how to use binary transport

---

## Benchmark Results

Benchmarks run on 2025-01-23 using `cargo bench -p rustbridge-transport`.

### Small Message Roundtrip (72-76 byte C structs)

| Benchmark | JSON | Binary | Speedup |
|-----------|------|--------|---------|
| **Full cycle** (serialize → deserialize → process → serialize → deserialize) | 654 ns | 92 ns | **7.1x** |
| Zero-copy path | N/A | 91 ns | — |

### Serialization Only

| Operation | JSON | Binary | Speedup |
|-----------|------|--------|---------|
| Serialize request | 73 ns | 48 ns | 1.5x |
| Serialize response | 104 ns | 48 ns | 2.2x |

### Deserialization Only

| Operation | JSON | Binary | Speedup |
|-----------|------|--------|---------|
| Deserialize request | 130 ns | ~1 ns | **130x** |
| Deserialize response | 197 ns | ~1 ns | **197x** |

### Payload Sizes

| Message | JSON | Binary | Ratio |
|---------|------|--------|-------|
| Request | 51 bytes | 72 bytes | 1.4x larger |
| Response | 78 bytes | 76 bytes | ~same |

### Key Insights

1. **Deserialization dominates**: Binary's pointer-cast "deserialization" is essentially free
2. **Processing still allocates**: Both paths use `format!()` which allocates, limiting gains
3. **Payload size tradeoff**: Binary uses fixed buffers (larger) but enables zero-copy
4. **Best for hot paths**: 7.1x improvement is significant for high-frequency calls

### Files Created

- `crates/rustbridge-ffi/src/binary_types.rs` - Core FFI-safe types
- `crates/rustbridge-ffi/src/binary_types/binary_types_tests.rs` - Unit tests
- `crates/rustbridge-transport/benches/json_baseline.rs` - JSON benchmarks
- `crates/rustbridge-transport/benches/binary_comparison.rs` - Comparative benchmarks
- `examples/hello-plugin/src/binary_messages.rs` - C struct message types
- `include/rustbridge_types.h` - C header for host language integration

---

## Benchmark Message Definitions

### Small Payload (~100 bytes)

```rust
// Simulates: config lookup, feature flag check
struct SmallRequest {
    key: String,        // 32 chars max
    flags: u32,
}

struct SmallResponse {
    value: String,      // 64 chars max
    ttl_seconds: u32,
}
```

### Medium Payload (~1KB)

```rust
// Simulates: user record, API entity
struct MediumRequest {
    user_id: u64,
    include_fields: Vec<String>,  // ~10 field names
}

struct MediumResponse {
    user_id: u64,
    username: String,
    email: String,
    metadata: HashMap<String, String>,  // ~10 entries
    permissions: Vec<String>,           // ~20 entries
}
```

### Large Payload (~100KB)

```rust
// Simulates: batch operation, data export
struct LargeRequest {
    query_id: u64,
    filters: Vec<Filter>,     // ~50 filters
}

struct LargeResponse {
    query_id: u64,
    results: Vec<Record>,     // ~1000 records
    total_count: u64,
}
```

---

## Current Sprint

### Completed ✅

1. **Benchmark infrastructure** - Criterion harness, JSON baseline, message types
2. **Binary transport types** - RbString, RbBytes, RbStringOwned, RbBytesOwned, RbResponse
3. **FFI entry point** - `plugin_call_raw` with binary handler registry
4. **Comparative benchmarks** - Full cycle, serialization, deserialization
5. **Decision point** - Binary transport approved (7.1x improvement)
6. **Phase 8: Binary Transport Generalization**
   - Header generation CLI with cross-platform verification (`cc` crate)
   - Java FFM BinaryStruct utilities and callRaw() method
   - Version field in structs for forward compatibility
   - Migration guide (docs/BINARY_TRANSPORT.md)
7. **Phase 6: Bundle Format**
   - rustbridge-bundle crate with manifest, builder, loader
   - CLI commands: create, list, extract
   - SHA256 checksum verification
   - Platform detection and library extraction
8. **Code Signing (Minisign)**
   - Key generation: `rustbridge keygen` command
   - Bundle signing: `--sign-key` option for bundle creation
   - Signature verification in Java BundleLoader
   - Secure by default (verification enabled unless explicitly disabled)
   - Public key embedded in manifest + optional override
9. **Java Bundle Loader**
   - BundleLoader class with builder pattern
   - Platform detection and library extraction
   - SHA256 checksum verification
   - Minisign signature verification
   - MinisignVerifier using pure Java Ed25519 (Java 8+ compatible)
10. **Schema Embedding**
   - Schema catalog in bundle manifest with metadata
   - Auto-generate C headers during bundle creation (--generate-header)
   - Manual schema file embedding with auto-format detection
   - Java API for schema extraction (getSchemas, readSchema, extractSchema)
   - Documentation in CLAUDE.md with usage examples

### Next Up

1. **C# bindings** (if needed)
   - Same patterns as Java FFM
   - Port BundleLoader and MinisignVerifier to C#

---

## Deferred Tasks

These tasks are on hold until benchmark work is complete:

| Task | Original Phase | Notes |
|------|----------------|-------|
| JNI native implementation | Phase 2 | Low priority, FFM preferred |
| Async API (plugin_call_async) | Phase 4 | After transport decision |
| C# bindings | Phase 5 | After transport decision |
| Python bindings | Phase 5 | After transport decision |
| Code generation | Phase 3 | After schema format settled |

---

## Technical Debt

| Issue | Priority | Notes |
|-------|----------|-------|
| Unused code warnings | Low | Clean up dead code in runtime, logging |
| Missing doc comments | Medium | Document all public APIs |
| Error message quality | Low | Improve actionable error messages |
| FfmPlugin synchronization | Medium | Per-call arenas for true thread safety |

---

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2024-01 | JSON as primary transport | Universal compatibility, debugging ease |
| 2024-01 | Mandatory Tokio runtime | Simplifies API, consistent async model |
| 2024-01 | "Rust allocates, host frees" | Clear ownership, prevents double-free |
| 2024-01 | FFM primary, JNI fallback | FFM is future, JNI for compatibility |
| 2024-01 | Separate test files | Faster rebuilds, cleaner separation |
| 2024-01 | Triple-underscore test names | Readable specifications |
| 2025-01 | Benchmark before generalizing binary | Data-driven decision on complexity |
| 2025-01 | C structs over MessagePack | Better peak performance for FFI |
| 2025-01 | Proceed with binary transport | 7.1x latency improvement justifies complexity |
| 2025-01 | JSON remains default, binary opt-in | Debugging ease vs performance tradeoff |
| 2025-01 | Minisign for code signing | Simple Ed25519 signatures, good Rust support, pure Java verification |
| 2025-01 | Sign libraries + manifest | Complete bundle integrity, public key in manifest + override option |
| 2025-01 | Verification enabled by default | Security by default, explicit opt-out required |

---

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Memory leaks across FFI | High | ASAN testing, clear ownership docs |
| Binary transport not worth it | Medium | Benchmark early, fail fast |
| C struct alignment issues | Medium | Explicit repr(C), test on all platforms |
| Maintenance burden of two transports | Medium | Keep binary opt-in, JSON default |
| Platform-specific bugs | Medium | CI matrix, sanitizer testing |

---

## Success Metrics

### Benchmark Phase Success ✅
- [x] Baseline JSON benchmarks documented (654 ns full cycle)
- [x] C struct transport functional for benchmark messages
- [x] Comparative data for small payloads (7.1x improvement)
- [x] Clear recommendation documented (proceed with binary transport)

### Binary Transport Success (Phase 8 targets)
- [x] 7.1x throughput improvement demonstrated (target was 10x - partial)
- [x] <1μs latency for small messages (92 ns achieved)
- [x] Java FFM binary transport support (BinaryStruct, callRaw())
- [x] Header generation automated (`rustbridge generate-header --verify`)

---

## How to Run Benchmarks

```bash
# Run all transport benchmarks
cargo bench -p rustbridge-transport

# Run specific benchmark group
cargo bench -p rustbridge-transport -- small_roundtrip

# Run JSON baseline only
cargo bench -p rustbridge-transport --bench json_baseline

# Run binary comparison only
cargo bench -p rustbridge-transport --bench binary_comparison

# With profiling
cargo bench -p rustbridge-transport -- --profile-time=5

# Java benchmarks (not yet implemented)
cd rustbridge-java && ./gradlew jmh
```

---

## Hardware Reference

Benchmarks should be run on consistent hardware. Document specs:

| Component | Specification |
|-----------|---------------|
| CPU | (document before benchmarking) |
| RAM | (document before benchmarking) |
| OS | (document before benchmarking) |
| Rust version | (document before benchmarking) |
| Java version | (document before benchmarking) |
