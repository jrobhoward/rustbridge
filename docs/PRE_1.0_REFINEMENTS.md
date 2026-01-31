# Pre-1.0 Code Quality Refinements

This document outlines code quality improvements identified during comprehensive review, organized into phases for incremental implementation.

## Overview

| Phase | Focus | Estimated Scope | Languages | Status |
|-------|-------|-----------------|-----------|--------|
| 1 | Critical Resource Safety | 3 fixes | C#, Java | ✅ Complete |
| 2 | Error Handling & Logging | 4 fixes | Java, Rust, C# | ✅ Complete |
| 3 | Performance & Polish | 3 fixes | Rust, C#, Java | ✅ Complete |

---

## Phase 1: Critical Resource Safety

**Goal:** Fix resource management issues that could cause leaks or unexpected behavior.

### 1.1 C# BundleLoader.Dispose() Pattern

**File:** `rustbridge-csharp/RustBridge.Core/BundleLoader.cs:484-491`

**Problem:** If `_zipArchive.Dispose()` throws, `_fileStream` won't be disposed.

**Current code:**
```csharp
public void Dispose()
{
    if (_disposed) return;
    _disposed = true;

    _zipArchive.Dispose();
    _fileStream.Dispose();
}
```

**Fixed code:**
```csharp
public void Dispose()
{
    if (_disposed) return;
    _disposed = true;

    try
    {
        _zipArchive?.Dispose();
    }
    finally
    {
        _fileStream?.Dispose();
    }
}
```

**Validation:** `dotnet test` in rustbridge-csharp/

---

### 1.2 Java ObjectMapper Consistency

**Files:**
- `rustbridge-java/rustbridge-jni/src/main/java/com/rustbridge/jni/JniPlugin.java:17-18`
- `rustbridge-java/rustbridge-core/src/main/java/com/rustbridge/BundleLoader.java:570`

**Problem:** Multiple ObjectMapper instances created instead of using shared instance.

**Fix:** Use `JsonMapper.getInstance()` pattern consistently.

**In JniPlugin.java:**
```java
// Before
private static final ObjectMapper MAPPER = new ObjectMapper();

// After
private static final ObjectMapper MAPPER = JsonMapper.getInstance();
```

**In BundleLoader.java:**
```java
// Before
private static final ObjectMapper MAPPER = new ObjectMapper();

// After
private static final ObjectMapper MAPPER = JsonMapper.getInstance();
```

**Note:** May need to create `JsonMapper` utility class in rustbridge-core if not exists, or use existing pattern from FFM module.

**Validation:** `./gradlew test` in rustbridge-java/

---

### 1.3 Java Callback Exception Handling Documentation

**File:** `rustbridge-java/rustbridge-ffm/src/main/java/com/rustbridge/ffm/FfmPluginLoader.java:289-293`

**Problem:** Exceptions in log callbacks are silently caught and printed to stderr.

**Current code:**
```java
} catch (Exception e) {
    System.err.println("Exception in log callback: " + e.getMessage());
}
```

**Fix options (choose one):**

**Option A - Document behavior (minimal change):**
Add Javadoc to `logCallbackWrapper` explaining that exceptions are caught to prevent native code corruption, and logged to stderr.

**Option B - Add callback error handler (more robust):**
```java
// In FfmPluginLoader or FfmPlugin
private Consumer<Exception> callbackErrorHandler = e ->
    System.err.println("Exception in log callback: " + e.getMessage());

public void setCallbackErrorHandler(Consumer<Exception> handler) {
    this.callbackErrorHandler = handler;
}

// In logCallbackWrapper:
} catch (Exception e) {
    callbackErrorHandler.accept(e);
}
```

**Recommendation:** Option A for 1.0, Option B for post-1.0.

**Validation:** `./gradlew test` in rustbridge-java/

---

## Phase 2: Error Handling & Logging

**Goal:** Improve error handling patterns and replace ad-hoc logging.

### 2.1 Java JNI Static Initializer

**File:** `rustbridge-java/rustbridge-jni/src/main/java/com/rustbridge/jni/JniPlugin.java:20-27`

**Problem:** Static block throws RuntimeException if library not loaded, making class unusable.

**Current behavior:**
```java
static {
    if (!JniPluginLoader.isLibraryLoaded()) {
        throw new RuntimeException("JNI library not loaded...");
    }
}
```

**Fix:** Move validation to instance methods, allow class loading to succeed.

```java
// Remove static block entirely, add validation to methods:

private void ensureLibraryLoaded() {
    if (!JniPluginLoader.isLibraryLoaded()) {
        throw new IllegalStateException(
            "JNI library not loaded. Call JniPluginLoader.load() first.");
    }
}

@Override
public String call(String typeTag, String request) throws PluginException {
    ensureLibraryLoaded();
    // ... existing implementation
}
```

**Validation:** `./gradlew test` in rustbridge-java/

---

### 2.2 Replace System.err with Logging Framework

**Scope:** ~205 instances across rustbridge-java

**Files affected:**
- `rustbridge-java/rustbridge-ffm/src/main/java/com/rustbridge/ffm/FfmPlugin.java`
- `rustbridge-java/rustbridge-ffm/src/main/java/com/rustbridge/ffm/FfmPluginLoader.java`
- `rustbridge-java/rustbridge-jni/src/main/java/com/rustbridge/jni/JniPlugin.java`
- `rustbridge-java/rustbridge-jni/src/main/java/com/rustbridge/jni/JniPluginLoader.java`
- Various test files (can keep System.out in tests)

**Fix:**

1. Add slf4j-api dependency to build.gradle:
```gradle
implementation 'org.slf4j:slf4j-api:2.0.9'
testRuntimeOnly 'org.slf4j:slf4j-simple:2.0.9'
```

2. Replace System.err calls in production code:
```java
// Before
System.err.println("Warning: " + message);

// After
private static final Logger log = LoggerFactory.getLogger(ClassName.class);
log.warn("{}", message);
```

**Scope reduction:** Focus on production code only. Test code can retain System.out/err.

**Validation:** `./gradlew test` in rustbridge-java/

---

### 2.3 Rust Async API Documentation

**File:** `rustbridge-ffi/src/exports.rs:552-567`

**Problem:** `plugin_call_async` and `plugin_cancel_async` are stub implementations without documentation about their status.

**Current code:**
```rust
// TODO: Implement async call support
pub unsafe extern "C" fn plugin_call_async(...) -> u64 {
    0 // Return 0 to indicate not implemented
}
```

**Fix:** Add comprehensive documentation:
```rust
/// Initiates an asynchronous plugin call.
///
/// # Status
/// **Not yet implemented.** This function is reserved for future async API support.
/// Currently returns 0 to indicate the operation is not available.
///
/// # Planned Behavior
/// When implemented, this will:
/// 1. Accept a request and callback function pointer
/// 2. Return a request ID for tracking/cancellation
/// 3. Invoke the callback with the response when complete
///
/// # Safety
/// - All pointer parameters must be valid
/// - Callback must remain valid until invoked or cancelled
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_call_async(...) -> u64 {
    // Reserved for future implementation
    0
}
```

**Validation:** `cargo doc --no-deps` and `cargo test -p rustbridge-ffi`

---

### 2.4 C# MinisignVerifier Null Validation

**File:** `rustbridge-csharp/RustBridge.Core/MinisignVerifier.cs:42`

**Problem:** Constructor doesn't validate null input, will throw cryptic NullReferenceException.

**Fix:**
```csharp
public MinisignVerifier(string publicKeyBase64)
{
    ArgumentNullException.ThrowIfNull(publicKeyBase64);
    if (string.IsNullOrWhiteSpace(publicKeyBase64))
        throw new ArgumentException("Public key cannot be empty", nameof(publicKeyBase64));

    var (publicKeyBytes, keyId) = ParsePublicKey(publicKeyBase64);
    // ...
}
```

**Validation:** `dotnet test` in rustbridge-csharp/

---

## Phase 3: Performance & Polish

**Goal:** Performance optimizations and code deduplication.

### 3.1 Rust JSON Clone Optimization

**File:** `rustbridge-transport/src/envelope.rs:64,164`

**Problem:** `payload_as()` methods clone the entire JSON value before deserializing.

**Current code:**
```rust
pub fn payload_as<T>(&self) -> Result<T, serde_json::Error>
where T: DeserializeOwned
{
    serde_json::from_value(self.payload.clone())
}
```

**Fix:** Use reference-based deserialization where possible:
```rust
pub fn payload_as<T>(&self) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    // Deserialize from reference to avoid clone
    T::deserialize(&self.payload)
}
```

**Note:** This requires `T: Deserialize<'de>` rather than `DeserializeOwned`. Evaluate if this change is API-compatible. If not, add a separate `payload_as_ref()` method.

**Validation:** `cargo test -p rustbridge-transport` and run benchmarks

---

### 3.2 C# MinisignVerifier Span Optimization

**File:** `rustbridge-csharp/RustBridge.Core/MinisignVerifier.cs:203`

**Problem:** Converting `ReadOnlySpan<byte>` to array defeats purpose of using Span.

**Current code:**
```csharp
public bool Verify(ReadOnlySpan<byte> data, string signatureString)
{
    return Verify(data.ToArray(), signatureString);
}
```

**Fix:** Refactor to use Span throughout:
```csharp
public bool Verify(ReadOnlySpan<byte> data, string signatureString)
{
    var signature = ParseSignature(signatureString);

    // Use NSec's span-based API directly
    using var blake2b = new Blake2bMac(Blake2bMac.MaxKeySize, 64);
    var hash = blake2b.Mac(Key.None, data);

    return Ed25519.Ed.Verify(_publicKey, hash, signature.Signature);
}
```

**Note:** Verify NSec API supports Span operations. May need to restructure `ParseSignature` to avoid intermediate allocations.

**Validation:** `dotnet test` and run benchmarks in rustbridge-csharp/

---

### 3.3 Java Platform Detection Deduplication

**Files:**
- `rustbridge-java/rustbridge-ffm/src/main/java/com/rustbridge/ffm/FfmPluginLoader.java:155-166`
- `rustbridge-java/rustbridge-jni/src/main/java/com/rustbridge/jni/JniPluginLoader.java:217-228`

**Problem:** Platform detection logic duplicated between loaders.

**Fix:** Extract to utility class in rustbridge-core:

```java
// New file: rustbridge-core/src/main/java/com/rustbridge/util/PlatformDetector.java
package com.rustbridge.util;

public final class PlatformDetector {
    private PlatformDetector() {}

    public static String getLibraryName(String baseName) {
        String os = System.getProperty("os.name").toLowerCase();
        String arch = System.getProperty("os.arch").toLowerCase();

        String prefix = os.contains("win") ? "" : "lib";
        String suffix = os.contains("win") ? ".dll"
                      : os.contains("mac") ? ".dylib"
                      : ".so";
        String archSuffix = arch.contains("aarch64") || arch.contains("arm64")
                          ? "-aarch64" : "-x86_64";

        return prefix + baseName + archSuffix + suffix;
    }

    public static String getPlatformIdentifier() {
        // Returns "linux-x86_64", "macos-aarch64", "windows-x86_64", etc.
    }
}
```

**Then update both loaders to use `PlatformDetector.getLibraryName()`.**

**Validation:** `./gradlew test` in rustbridge-java/

---

## Optional: Test Code Improvements

These are lower priority and can be addressed post-1.0.

### C# Test Utility Extraction

**Files:**
- `rustbridge-csharp/RustBridge.Tests/HelloPluginIntegrationTest.cs`
- `rustbridge-csharp/RustBridge.Tests/ConcurrencyLimitTest.cs`
- `rustbridge-csharp/RustBridge.Tests/ResourceLeakTest.cs`

**Problem:** Plugin-finding code duplicated ~4 times.

**Fix:** Create `TestPluginHelper` class with shared plugin discovery logic.

---

## Validation Checklist

After completing all phases, run full validation:

```bash
# Rust
cargo fmt --all
cargo clippy --workspace --examples --tests -- -D warnings
cargo test --workspace

# Java
cd rustbridge-java && ./gradlew build && ./gradlew test

# C#
cd rustbridge-csharp && dotnet build && dotnet test

# Python
cd rustbridge-python && python -m pytest tests/ -v

# Full pre-commit
./scripts/pre-commit.sh
```

---

## Timeline Suggestion

| Phase | Suggested Timing |
|-------|------------------|
| Phase 1 | Before any RC release |
| Phase 2 | Before 1.0-beta |
| Phase 3 | Before 1.0 final |
| Optional | Post-1.0 |
