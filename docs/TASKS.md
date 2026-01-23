# rustbridge Tasks & Roadmap

This document tracks the implementation progress and upcoming tasks for the rustbridge project.

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

### Phase 2: Java Integration 🔄 IN PROGRESS

**Goal**: Working end-to-end Java↔Rust communication

| Task | Status | Notes |
|------|--------|-------|
| Java core interfaces | ✅ Done | Plugin, LifecycleState, LogLevel, PluginConfig |
| FFM implementation | ✅ Done | FfmPluginLoader, FfmPlugin, NativeBindings |
| JNI fallback skeleton | ✅ Done | JniPluginLoader, JniPlugin (needs native impl) |
| Log callback integration | ⬜ Todo | Upcall from Rust to Java for logging |
| FFM integration tests | ⬜ Todo | End-to-end test with hello-plugin |
| JNI native implementation | ⬜ Todo | Rust crate for JNI bridge |
| Gradle build setup | ⬜ Todo | Complete build configuration |
| Java documentation | ⬜ Todo | Javadoc for all public APIs |

### Phase 3: Code Generation

**Goal**: Generate type-safe host language bindings from rustbridge.toml

| Task | Status | Notes |
|------|--------|-------|
| Enhanced rustbridge-macros | ⬜ Todo | Full dispatch generation |
| JSON Schema support | ⬜ Todo | Parse schemas for typed code gen |
| Java code generation | ⬜ Todo | Request/Response records, typed API |
| C# code generation | ⬜ Todo | Strongly-typed bindings |
| Python code generation | ⬜ Todo | Type hints, dataclasses |
| Maven plugin skeleton | ⬜ Todo | Build integration |
| Gradle plugin skeleton | ⬜ Todo | Build integration |

### Phase 4: Async API

**Goal**: Non-blocking calls with CompletableFuture/Promise bridging

| Task | Status | Notes |
|------|--------|-------|
| plugin_call_async impl | ⬜ Todo | Callback-based async FFI |
| plugin_cancel_async impl | ⬜ Todo | Cancellation support |
| Pending request tracking | ⬜ Todo | Request registry with timeouts |
| Java CompletableFuture | ⬜ Todo | Async Java API |
| C# Task bridging | ⬜ Todo | Async C# API |
| Python asyncio bridging | ⬜ Todo | Async Python API |
| Performance benchmarks | ⬜ Todo | Measure async overhead |

### Phase 5: Tier 2 Languages

**Goal**: Functional C# and Python bindings

| Task | Status | Notes |
|------|--------|-------|
| C# P/Invoke bindings | ⬜ Todo | Low-level FFI layer |
| C# high-level API | ⬜ Todo | IPlugin interface, loader |
| Python ctypes bindings | ⬜ Todo | Low-level FFI layer |
| Python high-level API | ⬜ Todo | Plugin class, context manager |
| NuGet package setup | ⬜ Todo | C# distribution |
| PyPI package setup | ⬜ Todo | Python distribution |

### Phase 6: Polish

**Goal**: Production-ready release

| Task | Status | Notes |
|------|--------|-------|
| Comprehensive docs | ⬜ Todo | API documentation, guides |
| Example projects | ⬜ Todo | Real-world usage examples |
| Security review | ⬜ Todo | FFI safety audit |
| CI/CD pipeline | ⬜ Todo | GitHub Actions, release automation |
| Cross-platform testing | ⬜ Todo | Linux, macOS, Windows |
| Performance optimization | ⬜ Todo | Profiling, benchmarks |
| 1.0 release prep | ⬜ Todo | Versioning, changelog |

---

## Current Sprint

### Active Tasks

1. **Java FFM integration testing**
   - Build hello-plugin, load from Java
   - Verify call/response works end-to-end
   - Test error handling paths

2. **Refactor tests to new conventions**
   - Move inline `mod tests` to separate files
   - Apply `subject___condition___expected` naming
   - Add FFI boundary tests

3. **Log callback integration**
   - Implement FFM upcall for log callback
   - Test log forwarding from Rust to Java

### Blocked Tasks

- JNI native implementation (blocked on: JNI design decisions)
- Gradle plugin (blocked on: Java integration complete)

---

## Backlog

### High Priority

- [ ] End-to-end Java integration test
- [ ] Refactor tests to separate files
- [ ] Add CI with GitHub Actions
- [ ] Add ASAN/MSAN testing for FFI

### Medium Priority

- [ ] JSON Schema support for code gen
- [ ] Typed Java API generation
- [ ] Python bindings prototype
- [ ] C# bindings prototype

### Low Priority

- [ ] Go bindings (cgo)
- [ ] Erlang bindings (NIF)
- [ ] MessagePack transport option
- [ ] Binary protocol option (for performance)

---

## Technical Debt

| Issue | Priority | Notes |
|-------|----------|-------|
| Unused code warnings | Low | Clean up dead code in runtime, logging |
| Missing doc comments | Medium | Document all public APIs |
| Inline test modules | Medium | Migrate to separate test files |
| Error message quality | Low | Improve actionable error messages |

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

---

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Memory leaks across FFI | High | ASAN testing, clear ownership docs |
| Java FFM API changes | Medium | Abstraction layer, JNI fallback |
| Platform-specific bugs | Medium | CI matrix, sanitizer testing |
| Performance overhead | Medium | Benchmarks, optional binary protocol |

---

## How to Contribute

1. Pick a task from **Backlog** or **Current Sprint**
2. Create a branch: `feature/task-name` or `fix/issue-name`
3. Follow [SKILLS.md](./SKILLS.md) conventions
4. Follow [TESTING.md](./TESTING.md) for tests
5. Review [ARCHITECTURE.md](./ARCHITECTURE.md) for design context
6. Submit PR with clear description
7. Wait for review

---

## Release Checklist

For each release:

- [ ] All tests pass on Linux, macOS, Windows
- [ ] ASAN/MSAN clean
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in all Cargo.toml
- [ ] Version bumped in Java build.gradle.kts
- [ ] Git tag created
- [ ] Crates published to crates.io
- [ ] Java artifacts published to Maven Central
