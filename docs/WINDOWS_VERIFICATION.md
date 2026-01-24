# Windows Verification Plan

**Branch**: `chore/check_windows`

**Objective**: Validate that rustbridge builds, tests, and runs correctly on Windows, with particular focus on FFI, binary transport, and bundle operations.

---

## Prerequisites

### Required Software

- [ ] **Rust toolchain**: 1.85.0 or later (MSRV)
  ```powershell
  rustup --version
  rustc --version  # Should be 1.85.0+
  cargo --version
  ```

- [ ] **C/C++ Compiler**: MSVC Build Tools or MinGW-w64
  ```powershell
  # Option 1: MSVC (recommended)
  # Install Visual Studio 2019+ with "Desktop development with C++" workload
  # Or install standalone "Build Tools for Visual Studio"

  # Option 2: MinGW-w64
  # Install via https://www.mingw-w64.org/
  ```

- [ ] **Java Development Kit**: JDK 21+ (for FFM support)
  ```powershell
  java -version  # Should be 21+
  javac -version
  ```

- [ ] **Git**: For branch management
  ```powershell
  git --version
  ```

### Optional Tools

- [ ] **cargo-deny**: For security/license checks
  ```powershell
  cargo install cargo-deny
  ```

- [ ] **cargo-msrv**: For MSRV verification
  ```powershell
  cargo install cargo-msrv
  ```

---

## Phase 1: Environment Verification

### 1.1 Clone and Checkout

```powershell
# Navigate to project directory
cd path\to\rust_lang_interop

# Verify branch
git branch --show-current  # Should show: chore/check_windows

# Ensure clean working tree
git status
```

### 1.2 Verify Rust Toolchain

```powershell
# Check default target
rustc -vV | Select-String "host"  # Should show: x86_64-pc-windows-msvc or x86_64-pc-windows-gnu

# Ensure latest stable
rustup update stable
rustup default stable
```

### 1.3 Verify C Compiler

```powershell
# Test C compiler availability (used by rustbridge generate-header --verify)
# This will be tested implicitly, but you can verify manually:

# For MSVC:
cl.exe /?

# For MinGW:
gcc --version
```

---

## Phase 2: Build Verification

### 2.1 Clean Build - Debug

```powershell
# Clean previous artifacts
cargo clean

# Build all workspace crates (debug mode)
cargo build --workspace

# Expected output: Success, no errors
# Note: Warnings are OK, but document any Windows-specific warnings
```

**Success Criteria**:
- [x] All crates compile without errors
- [x] Binaries created in `target\debug\`

**Document**:
- Any Windows-specific compiler warnings
- Build time (for reference)

### 2.2 Clean Build - Release

```powershell
# Build release binaries
cargo build --workspace --release

# Build hello-plugin example
cargo build -p hello-plugin --release
```

**Success Criteria**:
- [x] All crates compile in release mode
- [x] `target\release\hello_plugin.dll` exists (note: Windows uses .dll, not .so/.dylib)

**Document**:
- Path to hello-plugin DLL
- File size of hello-plugin DLL

---

## Phase 3: Test Suite Execution

### 3.1 Unit Tests

```powershell
# Run all unit tests
cargo test --workspace --lib

# Expected: All tests pass
```

**Success Criteria**:
- [x] All unit tests pass
- [x] No panics or crashes

**Document**:
- Total number of tests run
- Any test failures (with full output)

### 3.2 Integration Tests

```powershell
# Run integration tests
cargo test --workspace --test '*'

# Expected: All integration tests pass
```

**Success Criteria**:
- [x] All integration tests pass
- [x] FFI calls work correctly on Windows

**Document**:
- Any failures in FFI-related tests
- Any path-related issues (Windows uses backslashes)

### 3.3 Doc Tests

```powershell
# Run documentation tests
cargo test --workspace --doc

# Expected: All doc tests pass
```

**Success Criteria**:
- [x] All doc tests pass

### 3.4 Example Tests

```powershell
# Run tests in examples
cargo test --workspace --examples
```

**Success Criteria**:
- [x] Example tests pass

---

## Phase 4: Clippy and Formatting

### 4.1 Clippy Checks

```powershell
# Run clippy on all code
cargo clippy --workspace --examples --tests -- -D warnings

# Expected: No warnings or errors
```

**Success Criteria**:
- [x] Clippy passes with no warnings
- [x] No Windows-specific lints triggered

**Document**:
- Any Windows-specific clippy warnings

### 4.2 Format Check

```powershell
# Check code formatting
cargo fmt --all -- --check

# Expected: All files formatted correctly
```

**Success Criteria**:
- [x] No formatting issues

---

## Phase 5: Binary Transport Validation

### 5.1 Benchmark Execution

```powershell
# Run all transport benchmarks
cargo bench -p rustbridge-transport

# This tests:
# - Binary struct serialization/deserialization
# - C struct alignment on Windows
# - FFI buffer management
```

**Success Criteria**:
- [x] Benchmarks complete without crashes
- [x] Binary transport shows similar speedup (~7x) as Linux

**Document**:
- Benchmark results (full cycle times)
- Any significant performance differences vs Linux
- Windows system specs (CPU, RAM, OS version)

### 5.2 Header Generation Test

```powershell
# Test C header generation with cross-platform verification
cargo run -p rustbridge-cli -- generate-header --verify

# This tests:
# - C header generation from Rust structs
# - Compilation with Windows C compiler (MSVC or MinGW)
# - Struct layout verification
```

**Success Criteria**:
- [x] Header generates successfully
- [x] Test C program compiles and runs
- [x] No alignment or size mismatches

**Document**:
- C compiler used (MSVC vs MinGW)
- Any compilation errors or warnings

### 5.3 Binary Message Tests

```powershell
# Run binary transport specific tests
cargo test -p rustbridge-ffi binary_types
cargo test -p rustbridge-transport binary
```

**Success Criteria**:
- [x] All binary transport tests pass
- [x] No memory safety issues
- [x] Struct alignment matches expectations

---

## Phase 6: Bundle Operations

### 6.1 Bundle Creation

```powershell
# Create a test bundle with Windows DLL
cargo run -p rustbridge-cli -- bundle create `
  --name hello-plugin `
  --version 1.0.0-windows-test `
  --lib windows-x86_64:target\release\hello_plugin.dll `
  --output test-windows.rbp

# Expected: Bundle created successfully
```

**Success Criteria**:
- [x] Bundle file created
- [x] File size is reasonable
- [x] No path-related errors

**Document**:
- Bundle file size
- Any path handling issues (backslash vs forward slash)

### 6.2 Bundle Listing

```powershell
# List bundle contents
cargo run -p rustbridge-cli -- bundle list test-windows.rbp

# Expected: Shows manifest and library
```

**Success Criteria**:
- [x] Manifest displayed correctly
- [x] Library path shown correctly
- [x] Platform detected as windows-x86_64

**Document**:
- Output of list command

### 6.3 Bundle Extraction

```powershell
# Extract bundle
New-Item -ItemType Directory -Force -Path test-extract
cargo run -p rustbridge-cli -- bundle extract test-windows.rbp --output test-extract

# Verify extracted DLL
Test-Path test-extract\hello_plugin.dll

# Expected: DLL extracted successfully
```

**Success Criteria**:
- [x] Library extracted to correct path
- [x] SHA256 checksum verified
- [x] Extracted DLL is valid

**Document**:
- Extracted file path
- Any path normalization issues

### 6.4 Cleanup

```powershell
# Clean up test artifacts
Remove-Item test-windows.rbp
Remove-Item -Recurse test-extract
```

---

## Phase 7: Java Integration Tests

### 7.1 Gradle Build

```powershell
# Navigate to Java bindings
cd rustbridge-java

# Build all Java modules
.\gradlew build

# Expected: All modules build successfully
```

**Success Criteria**:
- [x] All Java modules compile
- [x] No Windows-specific build issues

**Document**:
- Gradle version
- Any build warnings

### 7.2 Java Tests

```powershell
# Run all Java tests (includes FFM and JNI tests)
.\gradlew test

# Expected: All tests pass
```

**Success Criteria**:
- [x] All Java tests pass
- [x] FFM bindings work on Windows
- [x] DLL loading works correctly
- [x] Binary transport works from Java on Windows

**Document**:
- Number of tests run
- Any test failures
- FFM vs JNI test results

### 7.3 Java FFM Binary Transport Test

```powershell
# Run specific FFM tests that exercise binary transport
.\gradlew :rustbridge-ffm:test --tests "*Binary*"

# Expected: Binary struct mapping works correctly
```

**Success Criteria**:
- [x] Binary struct tests pass
- [x] Memory layout matches between Rust and Java on Windows

### 7.4 Return to Root

```powershell
# Navigate back to project root
cd ..
```

---

## Phase 8: Security and Dependency Checks

### 8.1 Cargo Deny

```powershell
# Check for security advisories and license issues
cargo deny check

# Expected: No advisories, no license issues
```

**Success Criteria**:
- [x] No security advisories
- [x] No license violations
- [x] No banned dependencies

**Document**:
- Any warnings or errors

### 8.2 MSRV Verification (Optional)

```powershell
# Verify minimum supported Rust version (1.85.0)
cargo msrv verify

# Expected: MSRV 1.85.0 is satisfied
```

**Success Criteria**:
- [x] Project builds on MSRV

---

## Phase 9: Pre-commit Validation

### 9.1 Run Pre-commit Script (if available on Windows)

**Note**: The `scripts/pre-commit.sh` is a bash script. On Windows, you can:

**Option A**: Use Git Bash or WSL
```bash
bash scripts/pre-commit.sh --fast
```

**Option B**: Run checks manually
```powershell
# Format check
cargo fmt --all -- --check

# Deny check
cargo deny check

# Tests
cargo test --workspace --lib
cargo test --workspace --test '*'

# Example build
cargo build -p hello-plugin --release

# Clippy
cargo clippy --workspace --examples --tests -- -D warnings
```

**Success Criteria**:
- [x] All pre-commit checks pass

---

## Phase 10: Platform-Specific Validation

### 10.1 DLL Characteristics

```powershell
# Check DLL exports
dumpbin /EXPORTS target\release\hello_plugin.dll

# Expected exports:
# - plugin_init
# - plugin_call
# - plugin_call_raw
# - plugin_shutdown
# - plugin_free_buffer
# - plugin_get_state
```

**Success Criteria**:
- [x] All required FFI functions exported
- [x] No unexpected exports

**Document**:
- List of exported functions

### 10.2 Path Handling Test

Create a test to verify path handling works correctly on Windows:

```powershell
# Test bundle with absolute Windows path
$absolutePath = (Get-Item target\release\hello_plugin.dll).FullName
cargo run -p rustbridge-cli -- bundle create `
  --name path-test `
  --version 1.0.0 `
  --lib windows-x86_64:$absolutePath `
  --output path-test.rbp

# Cleanup
Remove-Item path-test.rbp
```

**Success Criteria**:
- [x] Absolute paths work
- [x] Relative paths work
- [x] UNC paths work (if applicable)

### 10.3 Unicode Path Test

```powershell
# Create directory with unicode characters
New-Item -ItemType Directory -Force -Path "test-中文-path"

# Test bundle operations with unicode paths
# (Skip if not applicable to your use case)
```

**Success Criteria**:
- [x] Unicode paths handled correctly (or documented as limitation)

---

## Phase 11: Documentation

### 11.1 Create Verification Report

Create a file `docs/WINDOWS_VERIFICATION_RESULTS.md` with:

```markdown
# Windows Verification Results

**Date**: [YYYY-MM-DD]
**Windows Version**: [Windows 10/11, build number]
**Rust Version**: [e.g., 1.85.0]
**Java Version**: [e.g., OpenJDK 21.0.x]
**C Compiler**: [MSVC 2022 / MinGW-w64 x.x.x]

## System Specifications

- CPU: [processor model]
- RAM: [amount]
- OS: [Windows version]

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

[List any Windows-specific issues discovered]

## Performance Comparison

### Binary Transport Benchmarks

| Benchmark | Windows | Linux (reference) |
|-----------|---------|-------------------|
| Full cycle | X ns | 92 ns |
| Serialize request | X ns | 48 ns |
| Deserialize request | X ns | ~1 ns |

## Recommendations

[Any recommendations for Windows users or improvements needed]

## Sign-off

Branch `chore/check_windows` is ready to merge: ✅ / ❌

Tested by: [Name]
Date: [Date]
```

### 11.2 Update CI Documentation (if needed)

If you find Windows-specific requirements, document them in:
- `.github/workflows/` (if CI exists)
- `README.md` (if platform-specific notes needed)
- `docs/SKILLS.md` (if development practices need updates)

---

## Phase 12: Branch Merge Preparation

### 12.1 Final Checks

```powershell
# Ensure working directory is clean
git status

# Ensure all tests pass one more time
cargo test --workspace
```

### 12.2 Commit Results (if any changes needed)

```powershell
# If you made any Windows-specific fixes, commit them
git add .
git commit -m "docs: Add Windows verification results"
```

### 12.3 Push Branch

```powershell
# Push verification results
git push origin chore/check_windows
```

---

## Success Criteria Summary

For the branch to be merge-ready, all of the following must pass:

- [x] **Build**: All crates build in debug and release modes
- [x] **Tests**: All unit, integration, doc, and example tests pass
- [x] **Clippy**: No warnings with `-D warnings`
- [x] **Formatting**: `cargo fmt` passes
- [x] **Benchmarks**: Binary transport benchmarks complete without crashes
- [x] **Header Generation**: `--verify` flag succeeds with Windows C compiler
- [x] **Bundles**: Create, list, extract operations work correctly
- [x] **Java**: All Java tests pass, FFM bindings work on Windows
- [x] **Cargo Deny**: No security or license issues
- [x] **FFI**: All exported functions present in DLL
- [x] **Documentation**: Verification results documented

---

## Common Issues and Solutions

### Issue: MSVC Not Found

**Solution**: Install Visual Studio Build Tools or add MSVC to PATH:
```powershell
# Find MSVC installation
$vsPath = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
# Add to PATH temporarily
$env:PATH += ";$vsPath\VC\Tools\MSVC\14.xx.xxxxx\bin\Hostx64\x64"
```

### Issue: Java Tests Fail to Load DLL

**Solution**: Ensure DLL is built before running Java tests:
```powershell
cargo build -p hello-plugin --release
```

### Issue: Path Too Long Errors

**Solution**: Enable long paths in Windows:
```powershell
# Run as Administrator
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### Issue: Permission Denied on DLL

**Solution**: Close any processes using the DLL (Java processes, etc.) and retry.

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
- Compare benchmark results with Linux baseline
- Test on both MSVC and MinGW if possible (note which was used)
- Run tests multiple times if you encounter intermittent failures

---

**Next Steps After Verification**:

1. Review `WINDOWS_VERIFICATION_RESULTS.md`
2. Address any critical issues found
3. Merge `chore/check_windows` into `main`
4. Update CI/CD to include Windows testing (if applicable)
5. Proceed with next development phase (Java bundle loader, etc.)
