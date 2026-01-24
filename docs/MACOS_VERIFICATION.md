# macOS Verification Plan

**Branch**: `chore/check_macos`

**Objective**: Validate that rustbridge builds, tests, and runs correctly on macOS, with particular focus on FFI, binary transport, bundle operations, and both Intel (x86_64) and Apple Silicon (aarch64) architectures.

---

## Prerequisites

### Required Software

- [ ] **Rust toolchain**: 1.85.0 or later (MSRV)
  ```bash
  rustup --version
  rustc --version  # Should be 1.85.0+
  cargo --version
  ```

- [ ] **C/C++ Compiler**: Xcode Command Line Tools or full Xcode
  ```bash
  # Install Xcode Command Line Tools (if not already installed)
  xcode-select --install

  # Verify installation
  clang --version
  ```

- [ ] **Java Development Kit**: JDK 21+ (for FFM support)
  ```bash
  java -version  # Should be 21+
  javac -version
  ```

- [ ] **Git**: For branch management
  ```bash
  git --version
  ```

### Optional Tools

- [ ] **cargo-deny**: For security/license checks
  ```bash
  cargo install cargo-deny
  ```

- [ ] **cargo-msrv**: For MSRV verification
  ```bash
  cargo install cargo-msrv
  ```

- [ ] **Homebrew**: For installing additional tools (optional)
  ```bash
  brew --version
  ```

---

## Phase 1: Environment Verification

### 1.1 Clone and Checkout

```bash
# Navigate to project directory
cd /path/to/rustbridge

# Verify branch
git branch --show-current  # Should show: chore/check_macos

# Ensure clean working tree
git status
```

### 1.2 Verify Rust Toolchain

```bash
# Check default target
rustc -vV | grep host
# Should show one of:
# - aarch64-apple-darwin (Apple Silicon)
# - x86_64-apple-darwin (Intel)

# Ensure latest stable
rustup update stable
rustup default stable
```

### 1.3 Verify C Compiler

```bash
# Verify clang is available (used by rustbridge generate-header --verify)
clang --version

# Check SDK path
xcrun --show-sdk-path
```

### 1.4 Determine Architecture

```bash
# Check current architecture
uname -m
# aarch64 = Apple Silicon (M1/M2/M3)
# x86_64 = Intel

# Check if running under Rosetta 2 (Intel emulation on Apple Silicon)
sysctl -n sysctl.proc_translated 2>/dev/null || echo "Native"
# 0 or "Native" = running natively
# 1 = running under Rosetta 2
```

---

## Phase 2: Build Verification

### 2.1 Clean Build - Debug

```bash
# Clean previous artifacts
cargo clean

# Build all workspace crates (debug mode)
cargo build --workspace

# Expected output: Success, no errors
# Note: Warnings are OK, but document any macOS-specific warnings
```

**Success Criteria**:
- [ ] All crates compile without errors
- [ ] Binaries created in `target/debug/`

**Document**:
- Any macOS-specific compiler warnings
- Build time (for reference)

### 2.2 Clean Build - Release

```bash
# Build release binaries
cargo build --workspace --release

# Build hello-plugin example
cargo build -p hello-plugin --release
```

**Success Criteria**:
- [ ] All crates compile in release mode
- [ ] `target/release/libhello_plugin.dylib` exists (note: macOS uses .dylib)

**Document**:
- Path to hello-plugin dylib
- File size of hello-plugin dylib

### 2.3 Universal Binary (Optional - Apple Silicon only)

```bash
# If on Apple Silicon, optionally test building for Intel target
rustup target add x86_64-apple-darwin

# Build for Intel
cargo build -p hello-plugin --release --target x86_64-apple-darwin

# Verify both architectures work
file target/release/libhello_plugin.dylib
file target/x86_64-apple-darwin/release/libhello_plugin.dylib
```

**Success Criteria**:
- [ ] Native architecture build succeeds
- [ ] Cross-architecture build succeeds (if tested)

---

## Phase 3: Test Suite Execution

### 3.1 Unit Tests

```bash
# Run all unit tests
cargo test --workspace --lib

# Expected: All tests pass
```

**Success Criteria**:
- [ ] All unit tests pass
- [ ] No panics or crashes

**Document**:
- Total number of tests run
- Any test failures (with full output)

### 3.2 Integration Tests

```bash
# Run integration tests
cargo test --workspace --test '*'

# Expected: All integration tests pass
```

**Success Criteria**:
- [ ] All integration tests pass
- [ ] FFI calls work correctly on macOS

**Document**:
- Any failures in FFI-related tests
- Any path-related issues

### 3.3 Doc Tests

```bash
# Run documentation tests
cargo test --workspace --doc

# Expected: All doc tests pass
```

**Success Criteria**:
- [ ] All doc tests pass

### 3.4 Example Tests

```bash
# Run tests in examples
cargo test --workspace --examples
```

**Success Criteria**:
- [ ] Example tests pass

---

## Phase 4: Clippy and Formatting

### 4.1 Clippy Checks

```bash
# Run clippy on all code
cargo clippy --workspace --examples --tests -- -D warnings

# Expected: No warnings or errors
```

**Success Criteria**:
- [ ] Clippy passes with no warnings
- [ ] No macOS-specific lints triggered

**Document**:
- Any macOS-specific clippy warnings

### 4.2 Format Check

```bash
# Check code formatting
cargo fmt --all -- --check

# Expected: All files formatted correctly
```

**Success Criteria**:
- [ ] No formatting issues

---

## Phase 5: Binary Transport Validation

### 5.1 Benchmark Execution

```bash
# Run all transport benchmarks
cargo bench -p rustbridge-transport

# This tests:
# - Binary struct serialization/deserialization
# - C struct alignment on macOS
# - FFI buffer management
```

**Success Criteria**:
- [ ] Benchmarks complete without crashes
- [ ] Binary transport shows similar speedup (~7x) as other platforms

**Document**:
- Benchmark results (full cycle times)
- Any significant performance differences vs Linux/Windows
- macOS system specs (CPU type, RAM, OS version)

### 5.2 Header Generation Test

```bash
# Test C header generation with verification
cargo run -p rustbridge-cli -- generate-header --verify

# This tests:
# - C header generation from Rust structs
# - Compilation with macOS clang
# - Struct layout verification
```

**Success Criteria**:
- [ ] Header generates successfully
- [ ] Test C program compiles and runs
- [ ] No alignment or size mismatches

**Document**:
- C compiler used (clang version)
- Any compilation errors or warnings

### 5.3 Binary Message Tests

```bash
# Run binary transport specific tests
cargo test -p rustbridge-ffi binary_types
cargo test -p rustbridge-transport binary
```

**Success Criteria**:
- [ ] All binary transport tests pass
- [ ] No memory safety issues
- [ ] Struct alignment matches expectations

---

## Phase 6: Bundle Operations

### 6.1 Bundle Creation

```bash
# Determine platform string based on architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    PLATFORM="darwin-aarch64"
else
    PLATFORM="darwin-x86_64"
fi

# Create a test bundle with macOS dylib
cargo run -p rustbridge-cli -- bundle create \
  --name hello-plugin \
  --version 1.0.0-macos-test \
  --lib ${PLATFORM}:target/release/libhello_plugin.dylib \
  --output test-macos.rbp

# Expected: Bundle created successfully
```

**Success Criteria**:
- [ ] Bundle file created
- [ ] File size is reasonable
- [ ] No path-related errors

**Document**:
- Bundle file size
- Platform string used

### 6.2 Bundle Listing

```bash
# List bundle contents
cargo run -p rustbridge-cli -- bundle list test-macos.rbp

# Expected: Shows manifest and library
```

**Success Criteria**:
- [ ] Manifest displayed correctly
- [ ] Library path shown correctly
- [ ] Platform detected correctly (darwin-aarch64 or darwin-x86_64)

**Document**:
- Output of list command

### 6.3 Bundle Extraction

```bash
# Extract bundle
mkdir -p test-extract
cargo run -p rustbridge-cli -- bundle extract test-macos.rbp --output test-extract

# Verify extracted dylib
ls -la test-extract/libhello_plugin.dylib
file test-extract/libhello_plugin.dylib

# Expected: dylib extracted successfully
```

**Success Criteria**:
- [ ] Library extracted to correct path
- [ ] SHA256 checksum verified
- [ ] Extracted dylib is valid

**Document**:
- Extracted file path
- File type verification output

### 6.4 Cleanup

```bash
# Clean up test artifacts
rm test-macos.rbp
rm -rf test-extract
```

---

## Phase 7: Java Integration Tests

### 7.1 Gradle Build

```bash
# Navigate to Java bindings
cd rustbridge-java

# Build all Java modules
./gradlew build

# Expected: All modules build successfully
```

**Success Criteria**:
- [ ] All Java modules compile
- [ ] No macOS-specific build issues

**Document**:
- Gradle version
- Any build warnings

### 7.2 Java Tests

```bash
# Run all Java tests (includes FFM and JNI tests)
./gradlew test

# Expected: All tests pass
```

**Success Criteria**:
- [ ] All Java tests pass
- [ ] FFM bindings work on macOS
- [ ] dylib loading works correctly
- [ ] Binary transport works from Java on macOS

**Document**:
- Number of tests run
- Any test failures
- FFM vs JNI test results

### 7.3 Java FFM Binary Transport Test

```bash
# Run specific FFM tests that exercise binary transport
./gradlew :rustbridge-ffm:test --tests "*Binary*"

# Expected: Binary struct mapping works correctly
```

**Success Criteria**:
- [ ] Binary struct tests pass
- [ ] Memory layout matches between Rust and Java on macOS

### 7.4 Return to Root

```bash
# Navigate back to project root
cd ..
```

---

## Phase 8: Security and Dependency Checks

### 8.1 Cargo Deny

```bash
# Check for security advisories and license issues
cargo deny check

# Expected: No advisories, no license issues
```

**Success Criteria**:
- [ ] No security advisories
- [ ] No license violations
- [ ] No banned dependencies

**Document**:
- Any warnings or errors

### 8.2 MSRV Verification (Optional)

```bash
# Verify minimum supported Rust version (1.85.0)
cargo msrv verify

# Expected: MSRV 1.85.0 is satisfied
```

**Success Criteria**:
- [ ] Project builds on MSRV

---

## Phase 9: Pre-commit Validation

### 9.1 Run Pre-commit Script

```bash
# Run full pre-commit validation
./scripts/pre-commit.sh

# Or run fast mode (skip slower tests)
./scripts/pre-commit.sh --fast
```

**Success Criteria**:
- [ ] All pre-commit checks pass

---

## Phase 10: Platform-Specific Validation

### 10.1 dylib Characteristics

```bash
# Check dylib exports
nm -gU target/release/libhello_plugin.dylib | grep plugin_

# Expected exports:
# - _plugin_init
# - _plugin_call
# - _plugin_call_raw
# - _plugin_shutdown
# - _plugin_free_buffer
# - _plugin_get_state

# Check dylib architecture
file target/release/libhello_plugin.dylib

# Check dylib dependencies
otool -L target/release/libhello_plugin.dylib
```

**Success Criteria**:
- [ ] All required FFI functions exported (with _ prefix on macOS)
- [ ] No unexpected exports
- [ ] Correct architecture (arm64 or x86_64)

**Document**:
- List of exported functions
- Architecture type
- Library dependencies

### 10.2 Code Signing (Optional)

```bash
# Check if dylib is signed (may not be for development builds)
codesign -dv target/release/libhello_plugin.dylib 2>&1

# If unsigned, this is expected for local builds
# For distribution, code signing would be required
```

**Document**:
- Code signing status

### 10.3 Path Handling Test

```bash
# Test bundle with absolute macOS path
ABSOLUTE_PATH=$(pwd)/target/release/libhello_plugin.dylib
cargo run -p rustbridge-cli -- bundle create \
  --name path-test \
  --version 1.0.0 \
  --lib darwin-aarch64:${ABSOLUTE_PATH} \
  --output path-test.rbp

# Cleanup
rm path-test.rbp
```

**Success Criteria**:
- [ ] Absolute paths work
- [ ] Relative paths work
- [ ] Paths with spaces work (if applicable)

### 10.4 Unicode Path Test

```bash
# Create directory with unicode characters
mkdir -p "test-日本語-path"

# Copy dylib to unicode path
cp target/release/libhello_plugin.dylib "test-日本語-path/"

# Test bundle operations with unicode paths
cargo run -p rustbridge-cli -- bundle create \
  --name unicode-test \
  --version 1.0.0 \
  --lib darwin-aarch64:"test-日本語-path/libhello_plugin.dylib" \
  --output unicode-test.rbp

# Cleanup
rm -rf "test-日本語-path"
rm -f unicode-test.rbp
```

**Success Criteria**:
- [ ] Unicode paths handled correctly (or documented as limitation)

### 10.5 Library Loading Paths

```bash
# Verify @rpath and install_name settings
otool -l target/release/libhello_plugin.dylib | grep -A 2 LC_ID_DYLIB

# Check for any absolute paths embedded in the binary
otool -l target/release/libhello_plugin.dylib | grep -A 2 LC_RPATH
```

**Document**:
- Install name configuration
- Any embedded paths that might affect portability

---

## Phase 11: Documentation

### 11.1 Create Verification Report

Create a file `docs/MACOS_VERIFICATION_RESULTS.md` with:

```markdown
# macOS Verification Results

**Date**: [YYYY-MM-DD]
**macOS Version**: [e.g., macOS 14.2 Sonoma]
**Architecture**: [Apple Silicon (aarch64) / Intel (x86_64)]
**Rust Version**: [e.g., 1.85.0]
**Java Version**: [e.g., OpenJDK 21.0.x]
**C Compiler**: [e.g., Apple clang 15.0.0]

## System Specifications

- Mac Model: [e.g., MacBook Pro 14-inch 2023]
- Chip: [e.g., Apple M3 Pro / Intel Core i7]
- RAM: [amount]
- OS: [macOS version]

## Test Results Summary

| Phase | Status | Notes |
|-------|--------|-------|
| Build Verification | ✅/❌ | |
| Unit Tests | ✅/❌ | X/Y tests passed |
| Integration Tests | ✅/❌ | X/Y tests passed |
| Clippy | ✅/❌ | |
| Benchmarks | ✅/❌ | ~Xx speedup |
| Bundle Operations | ✅/❌ | |
| Java Tests | ✅/❌ | X/Y tests passed |
| Cargo Deny | ✅/❌ | |

## Issues Found

[List any macOS-specific issues discovered]

## Performance Comparison

### Binary Transport Benchmarks

| Benchmark | macOS (Apple Silicon) | macOS (Intel) | Linux (reference) |
|-----------|----------------------|---------------|-------------------|
| Full cycle | X ns | X ns | 654 ns |
| Serialize request | X ns | X ns | 48 ns |
| Deserialize request | X ns | X ns | ~1 ns |

## Recommendations

[Any recommendations for macOS users or improvements needed]

## Sign-off

Branch `chore/check_macos` is ready to merge: ✅ / ❌

Tested by: [Name]
Date: [Date]
```

### 11.2 Update CI Documentation (if needed)

If you find macOS-specific requirements, document them in:
- `.github/workflows/` (if CI exists)
- `README.md` (if platform-specific notes needed)
- `docs/SKILLS.md` (if development practices need updates)

---

## Phase 12: Branch Merge Preparation

### 12.1 Final Checks

```bash
# Ensure working directory is clean
git status

# Ensure all tests pass one more time
cargo test --workspace
```

### 12.2 Commit Results (if any changes needed)

```bash
# If you made any macOS-specific fixes, commit them
git add .
git commit -m "docs: Add macOS verification results"
```

### 12.3 Push Branch

```bash
# Push verification results
git push origin chore/check_macos
```

---

## Success Criteria Summary

For the branch to be merge-ready, all of the following must pass:

- [ ] **Build**: All crates build in debug and release modes
- [ ] **Tests**: All unit, integration, doc, and example tests pass
- [ ] **Clippy**: No warnings with `-D warnings`
- [ ] **Formatting**: `cargo fmt` passes
- [ ] **Benchmarks**: Binary transport benchmarks complete without crashes
- [ ] **Header Generation**: `--verify` flag succeeds with macOS clang
- [ ] **Bundles**: Create, list, extract operations work correctly
- [ ] **Java**: All Java tests pass, FFM bindings work on macOS
- [ ] **Cargo Deny**: No security or license issues
- [ ] **FFI**: All exported functions present in dylib
- [ ] **Documentation**: Verification results documented

---

## Common Issues and Solutions

### Issue: Xcode Command Line Tools Not Installed

**Solution**: Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### Issue: Java Tests Fail to Load dylib

**Solution**: Ensure dylib is built before running Java tests:
```bash
cargo build -p hello-plugin --release
```

Also check that `DYLD_LIBRARY_PATH` is set correctly if needed:
```bash
export DYLD_LIBRARY_PATH=$(pwd)/target/release:$DYLD_LIBRARY_PATH
```

### Issue: Code Signing Errors (Gatekeeper)

**Solution**: For local development, you can allow unsigned dylibs:
```bash
# Remove quarantine attribute if downloaded
xattr -d com.apple.quarantine target/release/libhello_plugin.dylib
```

For distribution, sign with a valid Apple Developer certificate.

### Issue: Library Not Found at Runtime

**Solution**: Check `@rpath` settings and library search paths:
```bash
# View library dependencies
otool -L target/release/libhello_plugin.dylib

# Set library path temporarily
export DYLD_LIBRARY_PATH=/path/to/dylib:$DYLD_LIBRARY_PATH
```

### Issue: Rosetta 2 Performance

**Solution**: If running on Apple Silicon under Rosetta 2, benchmarks may be slower. Run native ARM64 builds for accurate performance testing:
```bash
# Check if running under Rosetta
sysctl -n sysctl.proc_translated
# 0 = native, 1 = Rosetta

# Ensure building for native architecture
rustup default stable
cargo build --release  # Builds for native arch by default
```

### Issue: Different Behavior on Intel vs Apple Silicon

**Solution**: Test on both architectures if possible. Key differences to watch for:
- Memory alignment (both are 8-byte aligned for most types)
- Endianness (both are little-endian)
- Performance characteristics (Apple Silicon has unified memory)

---

## macOS-Specific Considerations

### Architecture Support

| Architecture | Target Triple | Notes |
|--------------|---------------|-------|
| Apple Silicon | `aarch64-apple-darwin` | M1/M2/M3 Macs, ARM64 |
| Intel | `x86_64-apple-darwin` | Older Macs, x86-64 |

### Library Naming

- macOS dynamic libraries use `.dylib` extension
- FFI function names have `_` prefix when viewed with `nm`
- Use `@rpath` for relocatable libraries

### Security

- Gatekeeper may block unsigned libraries
- Hardened Runtime may restrict dylib loading
- Consider notarization for distribution

---

## Timeline Estimate

- **Phase 1-2** (Environment + Build): 15-30 minutes
- **Phase 3-4** (Tests + Clippy): 30-60 minutes
- **Phase 5** (Binary Transport): 15-30 minutes
- **Phase 6** (Bundles): 10-15 minutes
- **Phase 7** (Java): 30-45 minutes
- **Phase 8-9** (Security + Pre-commit): 10-20 minutes
- **Phase 10-11** (Platform-specific + Docs): 20-30 minutes
- **Phase 12** (Merge prep): 5-10 minutes

**Total**: ~2.5-4 hours (depending on issues found)

---

## Notes

- Take screenshots of any errors for documentation
- Copy full error messages into the results document
- Compare benchmark results with Linux/Windows baseline
- Test on both Apple Silicon and Intel if possible
- Note any architecture-specific behavior differences
- Verify FFM works with macOS JDK (Temurin, Zulu, or Oracle)

---

**Next Steps After Verification**:

1. Review `MACOS_VERIFICATION_RESULTS.md`
2. Address any critical issues found
3. Merge `chore/check_macos` into `main`
4. Update CI/CD to include macOS testing (if applicable)
5. Proceed with next development phase
