# macOS Verification Results

**Date**: 2026-01-23
**Branch**: `chore/check_windows` (macOS verification on existing branch)

## System Specifications

| Component | Specification |
|-----------|---------------|
| OS | macOS 26.2 (Build 25C56) |
| Architecture | Apple Silicon (arm64 / aarch64-apple-darwin) |
| Mac Model | Apple M1 |
| Rust Version | 1.91.1 (ed61e7d7e 2025-11-07) |
| Rust Target | aarch64-apple-darwin |
| Java Version | OpenJDK 24.0.1+9-30 (Java 21 toolchain auto-provisioned for tests) |
| C Compiler | Apple clang 17.0.0 (clang-1700.6.3.2) |
| Git | 2.50.1 (Apple Git-155) |

## Test Results Summary

| Phase | Status | Notes |
|-------|--------|-------|
| Build Verification | ✅ | Debug: 17.5s, Release: 30.6s |
| Unit Tests | ✅ | 292 tests passed |
| Integration Tests | ⚠️ | No integration tests defined |
| Doc Tests | ✅ | 4 passed, 8 ignored |
| Clippy | ✅ | No warnings |
| Formatting | ✅ | All files formatted |
| Binary Transport Benchmarks | ✅ | ~216ns full cycle (faster than Windows/Linux) |
| Binary Transport Tests | ✅ | 29 tests passed |
| Header Generation | ✅ | Generates and verifies correctly with clang |
| Bundle Operations | ✅ | Create, list, extract all work |
| Java Build | ✅ | All modules compile (requires Java 21 toolchain) |
| Java Tests | ✅ | All tests pass |
| Cargo Deny | ✅ | No advisories, licenses OK, sources OK |
| Pre-commit Script | ✅ | All checks passed |
| dylib Characteristics | ✅ | Valid Mach-O arm64 dylib |
| FFI Exports | ✅ | All 10 required functions exported |
| Unicode Paths | ✅ | Handled correctly |

## Detailed Results

### Build Verification

- **Debug build**: All crates compiled successfully in 17.5s
- **Release build**: All crates compiled successfully in 30.6s
- **libhello_plugin.dylib**: 1,264,128 bytes (~1.2 MB)
- **File type**: Mach-O 64-bit dynamically linked shared library arm64

### Test Suite

**Unit Tests (292 total)**:
- hello-plugin: 24 tests
- rustbridge-bundle: 21 tests
- rustbridge-core: 73 tests
- rustbridge-ffi: 86 tests
- rustbridge-logging: 20 tests
- rustbridge-macros: 6 tests
- rustbridge-runtime: 35 tests
- rustbridge-transport: 27 tests

**Doc Tests**: 4 passed (rustbridge-bundle), 8 ignored (macro examples)

### Binary Transport Performance

| Benchmark | macOS (M1) | Windows (reference) | Linux (reference) | Notes |
|-----------|------------|---------------------|-------------------|-------|
| Full cycle (small) | ~216 ns | ~495 ns | 654 ns | macOS significantly faster |
| Full cycle (medium) | ~2.2 µs | ~5.78 µs | N/A | |
| Full cycle (large 100 records) | ~39 µs | ~97.5 µs | N/A | |

Binary transport tests: 29/29 passed

### Bundle Operations

- **Create**: Successfully created bundle with macOS dylib (490 KB compressed)
- **List**: Correctly displays manifest and platform info
- **Extract**: Successfully extracts dylib with SHA256 verification
- **Platform detection**: Correctly identifies `darwin-aarch64`

### Cargo Deny

- **Advisories**: No security advisories
- **Bans**: No banned dependencies
- **Licenses**: All licenses compliant
- **Sources**: All sources valid

**Duplicate dependency warnings** (informational):
- hashbrown 0.14.5 / 0.16.1
- thiserror 1.0.69 / 2.0.18
- windows-sys 0.60.2 / 0.61.2

### Platform-Specific Notes

1. **FFI Exports**: All required functions exported with `_` prefix (macOS convention):
   - `_plugin_init`
   - `_plugin_call`
   - `_plugin_call_raw`
   - `_plugin_call_async`
   - `_plugin_cancel_async`
   - `_plugin_shutdown`
   - `_plugin_free_buffer`
   - `_plugin_get_state`
   - `_plugin_set_log_level`
   - `_plugin_create`

2. **Library Dependencies**: Minimal dependencies:
   - `/usr/lib/libiconv.2.dylib`
   - `/usr/lib/libSystem.B.dylib`

3. **Code Signing**: Ad-hoc signed by linker (expected for local builds)

4. **Java Toolchain**: Requires Java 21 toolchain. Gradle auto-provisions via foojay-resolver when configured in settings.gradle.kts.

## Issues Found

### Minor Issues

1. **Java 21 Toolchain**: The default settings.gradle.kts does not include toolchain auto-provisioning. Users must either:
   - Install Java 21 locally, or
   - Add the foojay-resolver plugin to settings.gradle.kts for auto-download

2. **Install Name**: The dylib has an absolute install name pointing to the build directory. For distribution, consider using `@rpath` or `@executable_path` relative paths.

## Performance Comparison

Apple Silicon (M1) shows excellent performance, significantly faster than the Windows and Linux reference:

| Metric | macOS (M1) | Windows | Linux | Improvement |
|--------|------------|---------|-------|-------------|
| Full cycle (small) | 216 ns | 495 ns | 654 ns | 2.3x-3x faster |
| Full cycle (medium) | 2.2 µs | 5.78 µs | N/A | 2.6x faster |
| Full cycle (large) | 39 µs | 97.5 µs | N/A | 2.5x faster |

This performance advantage is likely due to:
- Apple Silicon's unified memory architecture
- High single-threaded performance
- Efficient memory subsystem

## Recommendations

1. **Add macOS to CI**: The project builds and tests successfully on macOS, so adding macOS (both arm64 and x86_64) to the CI matrix would be valuable.

2. **Java Toolchain Setup**: Consider adding the foojay-resolver plugin to the default settings.gradle.kts to enable automatic Java 21 provisioning for users without Java 21 installed.

3. **dylib Install Name**: For production distribution, update the build to use `@rpath` or `@loader_path` for the install name to improve portability.

## Sign-off

Branch is verified on macOS: ✅

All functionality works correctly on macOS (Apple Silicon):
- Full build pipeline (debug + release)
- Complete test suite (292 unit tests)
- FFI/dylib loading (verified via Java FFM tests)
- Binary transport (benchmarks + 29 tests)
- Header generation with clang verification
- Bundle operations (create, list, extract)
- Security checks (cargo deny)
- Pre-commit validation

---

Tested by: Claude Code (automated verification)
Date: 2026-01-23
