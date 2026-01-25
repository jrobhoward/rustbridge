# Debugging rustbridge Plugins

This guide covers debugging techniques for rustbridge plugins across the FFI boundary, from Rust implementation through to Java/Kotlin host applications.

## Table of Contents

1. [Quick Start: Enable Debug Logging](#quick-start-enable-debug-logging)
2. [Rust-Side Debugging](#rust-side-debugging)
3. [Java/Kotlin-Side Debugging](#javakotlin-side-debugging)
4. [FFI Boundary Debugging](#ffi-boundary-debugging)
5. [Common Issues and Solutions](#common-issues-and-solutions)
6. [Memory Debugging](#memory-debugging)
7. [Concurrency Debugging](#concurrency-debugging)
8. [Performance Debugging](#performance-debugging)

## Quick Start: Enable Debug Logging

The fastest way to diagnose issues is to enable debug-level logging:

### Rust: Add Logging to Your Plugin

```rust
use tracing::{debug, info, warn, error};

async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    debug!("Received request: type={}, payload_len={}", type_tag, payload.len());

    match type_tag {
        "calculate" => {
            let req: CalculateRequest = serde_json::from_slice(payload)
                .map_err(|e| {
                    error!("Failed to deserialize request: {}", e);
                    PluginError::SerializationError(e.to_string())
                })?;

            debug!("Parsed request: {:?}", req);

            // Your logic...

            info!("Calculation complete: result={}", result);
            Ok(serde_json::to_vec(&response)?)
        }
        _ => {
            warn!("Unknown message type: {}", type_tag);
            Err(PluginError::UnknownMessageType(type_tag.to_string()))
        }
    }
}
```

### Java: Enable Debug Logging with Callback

```java
import com.rustbridge.LogCallback;
import com.rustbridge.PluginConfig;
import com.rustbridge.ffm.FfmPluginLoader;

LogCallback callback = (level, target, message) -> {
    System.out.printf("[%s] %s: %s%n", level, target, message);
};

PluginConfig config = PluginConfig.defaults()
    .logLevel("debug");  // Enable debug logs

try (Plugin plugin = FfmPluginLoader.load("plugin.so", config, callback)) {
    // Your calls here - you'll see all debug logs
}
```

### Kotlin: Enable Debug Logging

```kotlin
val callback = LogCallback { level, target, message ->
    println("[$level] $target: $message")
}

val config = PluginConfig.defaults()
    .logLevel("debug")

FfmPluginLoader.load("plugin.so", config, callback).use { plugin ->
    // Your calls here
}
```

**Log Levels** (from least to most verbose):
- `error` - Only errors
- `warn` - Errors and warnings
- `info` - Errors, warnings, and informational messages (default)
- `debug` - Detailed debugging information
- `trace` - Very verbose, trace-level information

## Rust-Side Debugging

### Building with Debug Symbols

Always build with debug symbols when debugging:

```bash
# Debug build (includes symbols, no optimizations)
cargo build

# Release build with debug symbols
cargo build --release --config profile.release.debug=true
```

Or add to `Cargo.toml`:

```toml
[profile.release]
debug = true  # Include debug symbols in release builds
```

### Using GDB (Linux)

```bash
# Build with debug symbols
cargo build

# Start GDB
gdb --args java -cp ... YourJavaClass

# Or attach to running process
gdb -p <pid>
```

**Useful GDB commands:**

```gdb
# Set breakpoint in Rust function
break calculator_plugin::handle_request

# Set breakpoint on panic
catch throw rust_panic

# Run the program
run

# Continue execution
continue

# Step over next line
next

# Step into function
step

# Print variable
print variable_name

# Print backtrace
backtrace

# Examine memory
x/10x $rsp
```

### Using LLDB (macOS)

```bash
# Build with debug symbols
cargo build

# Start LLDB
lldb -- java -cp ... YourJavaClass

# Or attach to running process
lldb -p <pid>
```

**Useful LLDB commands:**

```lldb
# Set breakpoint
breakpoint set -n calculator_plugin::handle_request

# Run the program
run

# Continue
continue

# Step over
next

# Step into
step

# Print variable
print variable_name

# Backtrace
bt

# Examine memory
memory read -s 10 $rsp
```

### Debugging with rust-lldb/rust-gdb

Rust provides wrappers that improve the debugging experience:

```bash
# Install rust toolchain (includes debugger wrappers)
rustup component add rust-src

# Use rust-gdb (Linux)
rust-gdb --args java -cp ... YourJavaClass

# Use rust-lldb (macOS)
rust-lldb -- java -cp ... YourJavaClass
```

These wrappers provide better Rust type formatting.

### Using println! Debugging (Quick and Dirty)

Sometimes the simplest approach works best:

```rust
async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    eprintln!("DEBUG: type_tag={}, payload={:?}", type_tag,
              String::from_utf8_lossy(payload));

    // Your logic...

    eprintln!("DEBUG: About to return response");
    Ok(response)
}
```

**Note**: Use `eprintln!` instead of `println!` - stderr is unbuffered and won't be lost if the program crashes.

### Using dbg! Macro

The `dbg!` macro is even more convenient:

```rust
let req: CalculateRequest = dbg!(serde_json::from_slice(payload))?;

// Prints:
// [src/lib.rs:42] serde_json::from_slice(payload) = Ok(
//     CalculateRequest {
//         value: 42,
//     },
// )
```

### Debugging Panics

Enable backtraces to see where panics occur:

```bash
RUST_BACKTRACE=1 java -cp ... YourJavaClass

# Or for more detailed backtraces
RUST_BACKTRACE=full java -cp ... YourJavaClass
```

When a panic occurs, you'll see output like:

```
thread 'tokio-runtime-worker' panicked at 'explicit panic', src/lib.rs:42:5
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: calculator_plugin::handle_request::{{closure}}
   ...
```

## Java/Kotlin-Side Debugging

### Using IntelliJ IDEA

1. **Set breakpoints** in your Java/Kotlin code
2. **Run in debug mode** (Shift+F9 or Debug icon)
3. **Step through** plugin calls

```java
try (Plugin plugin = FfmPluginLoader.load("plugin.so")) {
    String request = "{\"value\": 42}";

    // Set breakpoint here
    String response = plugin.call("calculate", request);  // ← Breakpoint

    System.out.println(response);
}
```

4. **Inspect variables** in the debugger
5. **Evaluate expressions** to check plugin state

### Using JDB (Command-Line)

```bash
# Compile with debug info
javac -g YourClass.java

# Run with JDB
jdb -classpath ... YourClass

# Set breakpoint
stop at YourClass:42

# Run
run

# Step over
next

# Step into
step

# Print variable
print variableName

# Continue
cont
```

### Logging Plugin State

```java
try (Plugin plugin = FfmPluginLoader.load("plugin.so")) {
    // Check plugin state
    int state = plugin.getState();
    System.out.println("Plugin state: " + state);  // 2 = Active

    // Change log level dynamically
    plugin.setLogLevel("debug");

    // Make call
    String response = plugin.call("calculate", request);

    // Check rejected request count (if using concurrency limits)
    long rejectedCount = plugin.getRejectedRequestCount();
    System.out.println("Rejected requests: " + rejectedCount);
}
```

### Catching and Logging Exceptions

```java
try {
    String response = plugin.call("calculate", request);
    System.out.println("Response: " + response);
} catch (PluginException e) {
    System.err.println("Plugin error:");
    System.err.println("  Code: " + e.getErrorCode());
    System.err.println("  Message: " + e.getMessage());
    e.printStackTrace();

    // Handle specific error codes
    if (e.getErrorCode() == 5) {
        System.err.println("Serialization error - check JSON format");
    }
}
```

### Kotlin-Specific Debugging

```kotlin
try {
    val response = plugin.call("calculate", request)
    println("Response: $response")
} catch (e: PluginException) {
    println("Error code: ${e.errorCode}")
    println("Message: ${e.message}")

    when (e.errorCode) {
        5 -> println("Serialization error - check JSON")
        6 -> println("Unknown message type")
        7 -> println("Handler error - check plugin logic")
        else -> println("Unexpected error")
    }

    e.printStackTrace()
}
```

## FFI Boundary Debugging

### Inspecting FFI Calls

Add debug output to see what crosses the FFI boundary:

```java
// Wrapper for debugging
class DebugPlugin implements Plugin {
    private final Plugin delegate;

    public DebugPlugin(Plugin delegate) {
        this.delegate = delegate;
    }

    @Override
    public String call(String typeTag, String request) throws PluginException {
        System.out.println("FFI CALL:");
        System.out.println("  Type: " + typeTag);
        System.out.println("  Request: " + request);

        try {
            String response = delegate.call(typeTag, request);
            System.out.println("  Response: " + response);
            return response;
        } catch (PluginException e) {
            System.err.println("  Error: " + e.getMessage());
            throw e;
        }
    }

    // Delegate other methods...
}

// Use wrapper
Plugin plugin = new DebugPlugin(FfmPluginLoader.load("plugin.so"));
```

### Using strace (Linux)

See all system calls, including library loading:

```bash
strace -f -e trace=openat,read,mmap java -cp ... YourClass 2>&1 | grep libcalculator
```

This shows:
- When the library is loaded
- Which symbols are resolved
- Memory mapping operations

### Using dtrace (macOS)

```bash
sudo dtrace -n 'pid$target::plugin_*:entry { printf("%s", probefunc); }' -c 'java -cp ... YourClass'
```

Shows all FFI function calls (plugin_init, plugin_call, etc.).

### Verifying Library Loading

```java
try {
    System.out.println("Loading plugin from: " + pluginPath);
    Plugin plugin = FfmPluginLoader.load(pluginPath);
    System.out.println("Plugin loaded successfully");
    System.out.println("Plugin state: " + plugin.getState());
} catch (UnsatisfiedLinkError e) {
    System.err.println("Failed to load library:");
    System.err.println("  Path: " + pluginPath);
    System.err.println("  Error: " + e.getMessage());

    // Check if file exists
    File lib = new File(pluginPath);
    System.err.println("  File exists: " + lib.exists());
    System.err.println("  Is file: " + lib.isFile());
    System.err.println("  Absolute path: " + lib.getAbsolutePath());
}
```

## Common Issues and Solutions

### Issue 1: Plugin Not Found

**Symptoms:**
```
FileNotFoundException: plugin.so
```

**Diagnosis:**
```java
String pluginPath = "plugin.so";
File f = new File(pluginPath);
System.out.println("Exists: " + f.exists());
System.out.println("Absolute: " + f.getAbsolutePath());
System.out.println("Working dir: " + System.getProperty("user.dir"));
```

**Solutions:**
- Use absolute paths
- Verify working directory
- Check file permissions
- Use bundles instead of raw libraries

### Issue 2: Symbol Not Found

**Symptoms:**
```
UnsatisfiedLinkError: undefined symbol: plugin_init
```

**Diagnosis:**

```bash
# Linux: Check exported symbols
nm -D target/release/libplugin.so | grep plugin_

# macOS: Check exported symbols
nm -gU target/release/libplugin.dylib | grep plugin_
```

**Solutions:**
- Verify `pub use rustbridge_ffi::{...}` in lib.rs
- Check `crate-type = ["cdylib"]` in Cargo.toml
- Rebuild the plugin

### Issue 3: Serialization Errors

**Symptoms:**
```
PluginException: serialization error (code 5)
```

**Diagnosis:**

Enable debug logging and inspect the JSON:

```rust
async fn handle_request(...) -> PluginResult<Vec<u8>> {
    debug!("Raw payload: {}", String::from_utf8_lossy(payload));

    let req: MyRequest = serde_json::from_slice(payload)
        .map_err(|e| {
            error!("Deserialization failed: {}", e);
            error!("Payload was: {}", String::from_utf8_lossy(payload));
            PluginError::SerializationError(e.to_string())
        })?;

    // ...
}
```

**Solutions:**
- Verify JSON field names match struct fields (case-sensitive)
- Check for missing required fields
- Validate JSON with a JSON formatter
- Use `#[serde(rename = "fieldName")]` if needed

### Issue 4: Plugin Stuck/Hanging

**Symptoms:**
- Call never returns
- No error, no response

**Diagnosis:**

```java
// Add timeout
ExecutorService executor = Executors.newSingleThreadExecutor();
Future<String> future = executor.submit(() -> plugin.call("stuck", request));

try {
    String response = future.get(5, TimeUnit.SECONDS);
} catch (TimeoutException e) {
    System.err.println("Plugin call timed out after 5 seconds");
    future.cancel(true);
}
```

**Common Causes:**
- Deadlock in Rust code (holding locks while calling async functions)
- Infinite loop in handler
- Waiting on external resource (network, database)

**Solutions:**
- Review lock usage (see [docs/SKILLS.md](./SKILLS.md) for lock safety)
- Add timeouts to external calls
- Use `tokio::time::timeout` in async code

### Issue 5: Concurrent Access Errors

**Symptoms:**
```
PluginException: too many concurrent requests (code 13)
```

**Diagnosis:**

```java
long rejectedCount = plugin.getRejectedRequestCount();
System.out.println("Rejected: " + rejectedCount);
```

**Solutions:**
- Increase `maxConcurrentOps` in PluginConfig
- Implement retry logic with backoff
- Reduce concurrent request load
- Use 0 for unlimited (if safe)

### Issue 6: Plugin State Errors

**Symptoms:**
```
PluginException: invalid lifecycle state (code 1)
```

**Diagnosis:**

```java
int state = plugin.getState();
System.out.println("Current state: " + state);
// 0=Installed, 1=Starting, 2=Active, 3=Stopping, 4=Stopped, 5=Failed
```

**Solutions:**
- Check if plugin failed during initialization
- Ensure plugin is Active (state=2) before calling
- Check logs for initialization errors

## Memory Debugging

### Checking for Memory Leaks (Valgrind on Linux)

```bash
# Build with debug symbols
cargo build

# Run under valgrind
valgrind --leak-check=full --show-leak-kinds=all \
  java -cp ... YourClass
```

Look for:
- Definitely lost (real leaks)
- Indirectly lost (leaked because parent was leaked)
- Possibly lost (might be false positives)

### Memory Profiling (heaptrack on Linux)

```bash
# Install heaptrack
sudo apt install heaptrack

# Profile your application
heaptrack java -cp ... YourClass

# Analyze results
heaptrack_gui heaptrack.YourClass.12345.gz
```

### Monitoring Memory Usage (Java Side)

```java
Runtime runtime = Runtime.getRuntime();

for (int i = 0; i < 1000; i++) {
    plugin.call("calculate", request);

    if (i % 100 == 0) {
        long used = runtime.totalMemory() - runtime.freeMemory();
        System.out.printf("Memory used: %.2f MB%n", used / 1_000_000.0);
    }
}
```

### Detecting Use-After-Free (AddressSanitizer)

Build with AddressSanitizer:

```bash
# Add to Cargo.toml (only for testing, not production)
RUSTFLAGS="-Z sanitizer=address" cargo build --target x86_64-unknown-linux-gnu

# Run your test
ASAN_OPTIONS=detect_leaks=1 java -cp ... YourTest
```

This catches:
- Use-after-free
- Buffer overflows
- Double-free

## Concurrency Debugging

### Detecting Data Races (ThreadSanitizer)

```bash
# Build with ThreadSanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo build --target x86_64-unknown-linux-gnu

# Run your concurrent test
java -cp ... YourConcurrentTest
```

### Debugging Deadlocks

Add timeout to lock acquisitions:

```rust
use parking_lot::RwLock;
use std::time::Duration;

let lock = RwLock::new(data);

// Try to acquire with timeout
if let Some(guard) = lock.try_write_for(Duration::from_secs(5)) {
    // Got lock
} else {
    error!("Failed to acquire lock after 5 seconds - possible deadlock");
}
```

### Logging Lock Acquisitions

```rust
use tracing::debug;

debug!("Acquiring read lock on cache");
let cache = self.cache.read();
debug!("Acquired read lock on cache");

// Use cache...

drop(cache);
debug!("Released read lock on cache");
```

### Testing Concurrent Scenarios

```java
@Test
void concurrent_calls___no_deadlock() throws Exception {
    ExecutorService executor = Executors.newFixedThreadPool(10);
    List<Future<String>> futures = new ArrayList<>();

    for (int i = 0; i < 100; i++) {
        Future<String> future = executor.submit(() ->
            plugin.call("calculate", "{\"value\": 42}")
        );
        futures.add(future);
    }

    // All should complete within reasonable time
    for (Future<String> future : futures) {
        String result = future.get(10, TimeUnit.SECONDS);
        assertNotNull(result);
    }

    executor.shutdown();
}
```

## Performance Debugging

### Measuring Call Latency

```java
long start = System.nanoTime();
String response = plugin.call("calculate", request);
long duration = System.nanoTime() - start;

System.out.printf("Call took: %.3f ms%n", duration / 1_000_000.0);
```

### Profiling with perf (Linux)

```bash
# Record performance data
perf record -F 99 -g java -cp ... YourClass

# Analyze results
perf report

# Generate flamegraph
perf script | flamegraph.pl > flamegraph.svg
```

### Finding Bottlenecks

Add timing to your Rust code:

```rust
use std::time::Instant;

async fn handle_request(...) -> PluginResult<Vec<u8>> {
    let start = Instant::now();

    let req: MyRequest = serde_json::from_slice(payload)?;
    debug!("Deserialization took: {:?}", start.elapsed());

    let start = Instant::now();
    let result = expensive_calculation(&req).await?;
    debug!("Calculation took: {:?}", start.elapsed());

    let start = Instant::now();
    let response = serde_json::to_vec(&result)?;
    debug!("Serialization took: {:?}", start.elapsed());

    Ok(response)
}
```

### JVM Profiling

Use VisualVM or JProfiler to profile the Java side:

```bash
# Start with JMX enabled
java -Dcom.sun.management.jmxremote \
     -Dcom.sun.management.jmxremote.port=9010 \
     -Dcom.sun.management.jmxremote.authenticate=false \
     -Dcom.sun.management.jmxremote.ssl=false \
     -cp ... YourClass
```

Then connect with VisualVM to see:
- CPU usage
- Memory allocations
- Thread states
- Garbage collection activity

## Tools Reference

### Rust Debugging Tools

| Tool | Platform | Use Case |
|------|----------|----------|
| rust-gdb | Linux | Interactive debugging with Rust types |
| rust-lldb | macOS | Interactive debugging with Rust types |
| cargo-expand | All | Macro expansion debugging |
| cargo-flamegraph | Linux | Performance profiling |
| valgrind | Linux | Memory leak detection |
| heaptrack | Linux | Memory profiling |

### Java Debugging Tools

| Tool | Platform | Use Case |
|------|----------|----------|
| IntelliJ IDEA | All | Interactive debugging |
| jdb | All | Command-line debugging |
| VisualVM | All | Profiling and monitoring |
| JProfiler | All | Advanced profiling |
| Eclipse MAT | All | Heap dump analysis |

### System Tools

| Tool | Platform | Use Case |
|------|----------|----------|
| strace | Linux | System call tracing |
| ltrace | Linux | Library call tracing |
| dtrace | macOS | Dynamic tracing |
| lsof | Unix | Open file debugging |
| nm | Unix | Symbol inspection |

## Best Practices

1. **Start with logging** - Enable debug logs before reaching for a debugger
2. **Use debug builds** - Always include debug symbols when debugging
3. **Isolate the problem** - Create minimal reproducible test cases
4. **Check both sides** - Debug Rust and Java/Kotlin separately
5. **Verify the boundary** - Check data crossing FFI is correct
6. **Test concurrently** - Many bugs only appear under concurrent load
7. **Profile before optimizing** - Measure to find real bottlenecks
8. **Keep logs** - Save logs from failed runs for analysis

## Getting Help

When reporting issues, include:

1. **Environment**: OS, Rust version, Java version
2. **Code**: Minimal reproducible example
3. **Logs**: Debug-level logs from both Rust and Java
4. **Error messages**: Complete error output with stack traces
5. **Steps**: Exact steps to reproduce the issue

See the project's issue tracker for more help.

## Related Documentation

- [docs/ERROR_HANDLING.md](./ERROR_HANDLING.md) - Error handling patterns
- [docs/SKILLS.md](./SKILLS.md) - Lock safety and best practices
- [docs/TESTING.md](./TESTING.md) - Testing strategies
- [docs/ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
