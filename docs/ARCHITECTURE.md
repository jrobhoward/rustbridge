# rustbridge Architecture

This document describes the architecture of rustbridge, the design decisions made, and the tradeoffs considered.

## Overview

rustbridge is a framework for building Rust shared libraries that can be called from other programming languages. It provides a high-level abstraction over the C ABI, handling memory management, async runtime integration, lifecycle management, and logging.

```mermaid
flowchart TB
    subgraph Host["Host Language (Java/C#/Python)"]
        HL[Host Application]
        HB[Language Bindings]
    end

    subgraph FFI["FFI Boundary (C ABI)"]
        PI[plugin_init]
        PC[plugin_call]
        PF[plugin_free_buffer]
        PS[plugin_shutdown]
    end

    subgraph Rust["Rust Plugin"]
        FH[FFI Handler]
        RT[Tokio Runtime]
        PL[Plugin Implementation]
        LC[Lifecycle Manager]
        LG[Logging Layer]
    end

    HL --> HB
    HB --> PI & PC & PF & PS
    PI & PC & PF & PS --> FH
    FH --> RT
    RT --> PL
    FH --> LC
    FH --> LG
    LG -.->|callback| HB
```

## Crate Architecture

The framework is organized into focused crates with clear responsibilities:

```mermaid
flowchart TD
    subgraph Core["Core Layer"]
        RC[rustbridge-core]
        RT[rustbridge-transport]
    end

    subgraph Runtime["Runtime Layer"]
        RR[rustbridge-runtime]
        RL[rustbridge-logging]
    end

    subgraph FFI["FFI Layer"]
        RF[rustbridge-ffi]
    end

    subgraph Tools["Tooling"]
        RM[rustbridge-macros]
        CLI[rustbridge-cli]
    end

    subgraph Plugin["User Plugin"]
        UP[hello-plugin]
    end

    RC --> RT
    RC --> RR
    RC --> RL
    RT --> RF
    RR --> RF
    RL --> RF
    RC --> RM
    RC --> CLI
    RT --> CLI

    RF --> UP
    RM --> UP
    RC --> UP
```

### Crate Responsibilities

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| **rustbridge-core** | Core abstractions | `Plugin`, `LifecycleState`, `PluginError`, `PluginConfig` |
| **rustbridge-transport** | Serialization | `JsonCodec`, `RequestEnvelope`, `ResponseEnvelope` |
| **rustbridge-runtime** | Async execution | `AsyncRuntime`, `AsyncBridge`, `ShutdownSignal` |
| **rustbridge-logging** | Log forwarding | `FfiLoggingLayer`, `LogCallbackManager` |
| **rustbridge-ffi** | C ABI exports | `FfiBuffer`, `PluginHandle`, FFI functions |
| **rustbridge-macros** | Code generation | `#[rustbridge_plugin]`, `derive(Message)` |
| **rustbridge-cli** | Build tooling | `new`, `build`, `generate`, `check` commands |

## Plugin Lifecycle

Plugins follow an OSGI-inspired lifecycle state machine:

```mermaid
stateDiagram-v2
    [*] --> Installed: plugin_create()

    Installed --> Starting: plugin_init()
    Starting --> Active: on_start() success
    Starting --> Failed: on_start() error

    Active --> Stopping: plugin_shutdown()
    Stopping --> Stopped: on_stop() complete
    Stopping --> Failed: on_stop() error

    Stopped --> Starting: restart
    Failed --> [*]: cleanup

    Active --> Active: plugin_call()
```

### State Descriptions

| State | Description | Allowed Operations |
|-------|-------------|-------------------|
| **Installed** | Plugin created, not initialized | `plugin_init()` |
| **Starting** | Initializing runtime and resources | Wait |
| **Active** | Ready to handle requests | `plugin_call()`, `plugin_shutdown()` |
| **Stopping** | Graceful shutdown in progress | Wait |
| **Stopped** | Shutdown complete | Restart or cleanup |
| **Failed** | Error occurred | Cleanup only |

## Request/Response Flow

```mermaid
sequenceDiagram
    participant H as Host
    participant F as FFI Layer
    participant R as Runtime
    participant P as Plugin

    H->>F: plugin_call(handle, "user.create", json)
    F->>F: Validate inputs
    F->>R: AsyncBridge::call_sync()
    R->>P: handle_request("user.create", payload)
    P->>P: Deserialize, process, serialize
    P-->>R: Result<Vec<u8>>
    R-->>F: Result<Vec<u8>>
    F->>F: Create FfiBuffer
    F-->>H: FfiBuffer { data, len, error_code }
    H->>H: Copy data to managed heap
    H->>F: plugin_free_buffer(buffer)
    F->>F: Deallocate
```

## Concurrency Limits and Backpressure

rustbridge provides built-in protection against memory exhaustion under high concurrent request loads through configurable concurrency limits.

### Configuration

The `max_concurrent_ops` field in `PluginConfig` controls the maximum number of concurrent requests:

- **Limited mode (`max_concurrent_ops > 0`)**: Enforces a hard limit on concurrent requests
- **Unlimited mode (`max_concurrent_ops = 0`)**: No limit enforced (default: 1000)

```rust
let config = PluginConfig {
    max_concurrent_ops: 100,  // Allow up to 100 concurrent requests
    ..Default::default()
};
```

### Implementation

The concurrency limit is enforced using a Tokio semaphore in `PluginHandle`:

```rust
pub struct PluginHandle {
    // ... other fields
    request_limiter: Option<Arc<tokio::sync::Semaphore>>,
    rejected_requests: AtomicU64,
}
```

**Request flow with concurrency limiting:**

1. `PluginHandle::call()` attempts to acquire a permit using `try_acquire()`
2. If successful, the request proceeds and the permit is held during execution
3. If failed (limit exceeded), returns `PluginError::TooManyRequests` immediately
4. Permit is automatically released when dropped (success or error path)

**Key design decisions:**

- **Non-blocking (`try_acquire`)**: Provides immediate backpressure instead of queuing/blocking
- **Fail-fast**: Callers receive errors immediately when capacity is exceeded
- **No queuing**: Prevents unbounded memory growth from queued requests
- **RAII permits**: Automatic cleanup ensures permits are always released

### Error Handling

When the concurrency limit is exceeded:

1. **Rust**: Returns `PluginError::TooManyRequests` (error code 13)
2. **FFI**: Returns error envelope with code 13
3. **Java**: Throws `PluginException` with error code 13
4. **Metrics**: `rejected_requests` counter is incremented

Callers should implement retry with backoff or load shedding strategies.

### Monitoring

Query rejected request count to monitor system health and tune limits:

**Rust:**
```rust
let count = handle.rejected_request_count();
```

**Java:**
```java
long count = plugin.getRejectedRequestCount();
```

### Trade-offs

**Why try_acquire() instead of blocking acquire():**
- Immediate backpressure (fail fast)
- No thread starvation from waiting
- Caller decides retry strategy
- Simpler reasoning about resource usage

**Why semaphore instead of atomic counter:**
- Tokio semaphore is well-tested and efficient
- RAII permit automatically releases
- Handles edge cases (overflow, fairness)
- Lock-free implementation

### Tuning Recommendations

1. **Start with defaults**: The default limit of 1000 is suitable for most applications
2. **Monitor rejection rate**: High rejection rates indicate undersized limits or overload
3. **Consider request latency**: Slower requests need lower limits to prevent memory exhaustion
4. **Profile memory usage**: Set limits based on measured per-request memory consumption
5. **Use unlimited mode (0) only**: When you have external rate limiting or for low-traffic scenarios

## Memory Management

rustbridge uses a **"Rust allocates, host frees"** pattern for memory safety:

```mermaid
flowchart LR
    subgraph Rust["Rust (Native Heap)"]
        A[Allocate FfiBuffer]
        D[Deallocate on free]
    end

    subgraph FFI["FFI Boundary"]
        B[Return FfiBuffer pointer]
        F[plugin_free_buffer]
    end

    subgraph Host["Host (Managed Heap)"]
        C[Copy data]
        M[Managed memory]
    end

    A --> B
    B --> C
    C --> M
    M --> F
    F --> D
```

### FfiBuffer Structure

```rust
#[repr(C)]
pub struct FfiBuffer {
    pub data: *mut u8,      // Pointer to Rust-allocated data
    pub len: usize,         // Data length in bytes
    pub capacity: usize,    // Allocation capacity
    pub error_code: u32,    // 0 = success, non-zero = error
}
```

### Memory Safety Guarantees

1. **Single ownership**: Rust owns native memory until `plugin_free_buffer()` is called
2. **No double-free**: Buffer tracks whether it's been freed
3. **No use-after-free**: Host must copy data before freeing
4. **Thread safety**: Buffer operations are not thread-safe; host must synchronize

## Async Runtime Integration

All plugins include a mandatory Tokio runtime:

```mermaid
flowchart TB
    subgraph FFI["FFI Thread"]
        FC[plugin_call]
        AB[AsyncBridge]
    end

    subgraph Tokio["Tokio Runtime"]
        RT[Runtime]
        W1[Worker 1]
        W2[Worker 2]
        W3[Worker N]
    end

    subgraph Plugin["Plugin"]
        HR[handle_request]
        AS[async operations]
    end

    FC --> AB
    AB -->|block_on| RT
    RT --> W1 & W2 & W3
    W1 --> HR
    HR --> AS
    AS -.->|await| HR
    HR -.->|result| AB
```

### Design Decision: Mandatory Async

**Tradeoff considered**: Optional vs mandatory async runtime

| Approach | Pros | Cons |
|----------|------|------|
| **Optional async** | Smaller binary for sync-only plugins | Complex API, two code paths |
| **Mandatory async** | Simpler API, consistent behavior | ~2MB binary overhead |

**Decision**: Mandatory async. The consistency and simplicity outweigh the binary size cost. Modern applications typically need async I/O anyway.

## Logging Architecture

Logs flow from Rust's `tracing` ecosystem to the host language via callbacks:

```mermaid
flowchart LR
    subgraph Rust["Rust"]
        T[tracing macros]
        L[FfiLoggingLayer]
        M[LogCallbackManager]
    end

    subgraph FFI["FFI"]
        CB[Log Callback]
    end

    subgraph Host["Host"]
        HL[Host Logger]
    end

    T --> L
    L --> M
    M --> CB
    CB --> HL
```

### Log Level Mapping

| Rust Level | Numeric | Description |
|------------|---------|-------------|
| Trace | 0 | Very detailed debugging |
| Debug | 1 | Debugging information |
| Info | 2 | General information |
| Warn | 3 | Warnings |
| Error | 4 | Errors |
| Off | 5 | Logging disabled |

### Design Decision: Callback vs Queue

**Tradeoff considered**: Synchronous callbacks vs async log queue

| Approach | Pros | Cons |
|----------|------|------|
| **Callbacks** | Immediate delivery, simple | Can block Rust if host is slow |
| **Queue** | Non-blocking, buffered | Complexity, potential log loss |

**Decision**: Synchronous callbacks. Logging should be immediate for debugging. Hosts can implement buffering if needed.

## Transport Layer

### JSON-Based Protocol

```mermaid
flowchart LR
    subgraph Request
        RQ[RequestEnvelope]
        TT[type_tag: string]
        PL[payload: JSON]
        ID[request_id: optional]
    end

    subgraph Response
        RS[ResponseEnvelope]
        ST[status: success/error]
        RP[payload: optional JSON]
        EC[error_code: optional]
        EM[error_message: optional]
    end

    RQ --> TT & PL & ID
    RS --> ST & RP & EC & EM
```

### Design Decision: JSON vs Binary

**Tradeoff considered**: JSON vs MessagePack vs Protocol Buffers

| Format | Pros | Cons |
|--------|------|------|
| **JSON** | Universal, debuggable, no schema | Larger, slower |
| **MessagePack** | Compact, fast, schema-optional | Less debuggable |
| **Protobuf** | Very compact, typed | Requires schema, complex |

**Decision**: JSON as primary format. Debuggability and universal support are critical for a framework targeting multiple languages. Binary formats can be added as optional features later.

## Host Language Integration

### Java Integration Strategy

```mermaid
flowchart TB
    subgraph Java["Java Application"]
        APP[Application Code]
        PI[Plugin Interface]
    end

    subgraph FFM["FFM (Java 21+)"]
        FL[FfmPluginLoader]
        FP[FfmPlugin]
        NB[NativeBindings]
    end

    subgraph JNI["JNI (Java 8+)"]
        JL[JniPluginLoader]
        JP[JniPlugin]
        NC[Native C Code]
    end

    APP --> PI
    PI --> FL
    PI --> JL
    FL --> FP --> NB
    JL --> JP --> NC
```

### Design Decision: FFM Primary, JNI Fallback

**Tradeoff considered**: FFM-only vs JNI-only vs both

| Approach | Pros | Cons |
|----------|------|------|
| **FFM only** | Modern, pure Java, better perf | Requires Java 21+ |
| **JNI only** | Wide compatibility | Complex, error-prone |
| **Both** | Best of both worlds | More code to maintain |

**Decision**: Support both with FFM as primary. FFM is the future, but JNI provides backward compatibility for enterprise environments on older JVMs.

## Error Handling Strategy

### Error Code Design

```mermaid
flowchart TD
    PE[PluginError] --> IC[InvalidState: 1]
    PE --> IF[InitFailed: 2]
    PE --> SF[ShutdownFailed: 3]
    PE --> CE[ConfigError: 4]
    PE --> SE[SerializationError: 5]
    PE --> UM[UnknownMessageType: 6]
    PE --> HE[HandlerError: 7]
    PE --> RE[RuntimeError: 8]
    PE --> CA[Cancelled: 9]
    PE --> TO[Timeout: 10]
    PE --> IE[InternalError: 11]
    PE --> FE[FfiError: 12]
```

### Design Decision: Stable Error Codes

**Rationale**: Error codes must be stable across versions for host language error handling. Using an enum with explicit numeric mapping ensures backward compatibility.

## Security Considerations

### FFI Boundary Safety

1. **Input validation**: All FFI functions validate pointers before dereferencing
2. **Null handling**: Null pointers return error buffers, never crash
3. **No panics across FFI**: All panics are caught and converted to errors
4. **Bounded operations**: No unbounded allocations from untrusted input

### Memory Safety

1. **No raw pointer arithmetic**: Use safe Rust abstractions
2. **Clear ownership**: "Rust allocates, host frees" pattern
3. **ASAN/MSAN testing**: Regular sanitizer runs in CI

## Performance Considerations

### Overhead Sources

| Operation | Overhead | Mitigation |
|-----------|----------|------------|
| JSON serialization | ~1-10μs | Optional binary formats |
| FFI call | ~10-100ns | Batch operations if needed |
| Tokio runtime | ~2MB memory | Shared across requests |
| Log callbacks | ~1μs | Level filtering |

### Optimization Opportunities

1. **Batch requests**: Combine multiple operations in one call
2. **Binary transport**: Optional MessagePack for hot paths
3. **Arena allocation**: Reuse buffers for repeated calls
4. **Log level filtering**: Filter in Rust before callback

## Current Limitations

### Plugin Reload and Multiple Instances

**Plugin Reload**: ✅ **Fully Supported**
- Plugins can be loaded, shut down, and reloaded in the same process
- All functionality works correctly after reload
- Clean shutdown with proper resource cleanup

**Multiple Plugin Instances**: ⚠️ **Single Plugin Per Process Recommended**

While the framework can technically load multiple plugin instances, they share global logging infrastructure:

**What's Shared:**
- Log callback function pointer (global `LogCallbackManager`)
- Tracing subscriber (process-global via `set_global_default`)
- Dynamic log level filtering (changes affect all plugins)

**Impact:**
- Multiple plugins will use the same log callback
- Log level changes in one plugin affect all plugins
- The last plugin to shut down clears the callback for all plugins

**Recommended Usage:**
```java
// ✅ RECOMMENDED: One plugin per process
try (Plugin plugin = FfmPluginLoader.load("libmyplugin.so")) {
    plugin.call("operation", request);
    plugin.setLogLevel(LogLevel.DEBUG);  // Works great!
}

// ✅ SUPPORTED: Reload in same process
plugin.close();
Plugin reloaded = FfmPluginLoader.load("libmyplugin.so");  // Works!

// ⚠️ WORKS BUT SHARES LOGGING: Multiple plugins
try (Plugin p1 = FfmPluginLoader.load("lib1.so");
     Plugin p2 = FfmPluginLoader.load("lib2.so")) {
    // Both work, but share log callback and level
}

// ✅ ALTERNATIVE: One plugin per process
// Use separate processes or containers for full isolation
```

**Why This Design?**

The shared logging state is an intentional trade-off:
- ✅ Simpler implementation for the common case (single plugin)
- ✅ Better performance (no per-call overhead for scoped logging)
- ✅ Reliable reload support (global state doesn't prevent reinitialization)
- ⚠️ Multi-plugin scenarios require awareness of shared state

**Future Enhancement**: If multi-plugin with isolated logging becomes a requirement, we can implement per-handle logging state. See [PLUGIN_RELOAD_STATUS.md](./PLUGIN_RELOAD_STATUS.md) for details.

**Related Documentation**:
- [Plugin Reload Status](./PLUGIN_RELOAD_STATUS.md) - Test results and detailed analysis
- [Reload Safety Analysis](./RELOAD_SAFETY_ANALYSIS.md) - Global state inventory and considerations

---

## Future Considerations

### Planned Extensions

1. **Async FFI API**: `plugin_call_async()` with completion callbacks
2. **Streaming**: Bidirectional streaming for large data
3. **Metrics**: Built-in performance metrics export
4. **Hot reload**: Plugin update without process restart

### Not Planned

1. **Shared memory transport**: Complexity outweighs benefits
2. **Custom allocators**: Standard allocator is sufficient
3. **Multiple runtime support**: Tokio-only simplifies API
