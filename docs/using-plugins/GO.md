# Getting Started: Go

This guide walks you through using rustbridge plugins from Go using CGo and dlopen.

## Prerequisites

- **Go 1.21 or later** - For generics and modern standard library features
  ```bash
  go version  # Should be >= 1.21
  ```
- **C compiler** - Required for CGo (gcc on Linux, clang on macOS)
- **A rustbridge plugin** - A native shared library built with the rustbridge framework

## Installation

```bash
go get github.com/jrobhoward/rustbridge-go
```

### Local Development

For building against rustbridge source instead of the published module, see the [Development Guide](../DEVELOPMENT.md#go).

## Loading a Plugin

```go
import (
	"log"
	rustbridge "github.com/jrobhoward/rustbridge-go"
)

pluginPath := "target/release/libmyplugin.so"    // Linux
// pluginPath := "target/release/libmyplugin.dylib" // macOS
// pluginPath := "target/release/myplugin.dll"      // Windows

plugin, err := rustbridge.Load(pluginPath)
if err != nil {
	log.Fatal(err)
}
defer plugin.Close()

response, err := plugin.Call("echo", `{"message": "Hello"}`)
if err != nil {
	log.Fatal(err)
}
fmt.Println(response)
```

`Plugin` implements `io.Closer`, so `defer plugin.Close()` is the idiomatic cleanup pattern. It is safe to call `Close` multiple times.

## Making JSON Calls

```go
// Simple string call
response, err := plugin.Call("echo", `{"message": "Hello, World!"}`)
if err != nil {
	log.Fatal(err)
}
fmt.Println(response)

// With maps and json.Marshal/Unmarshal
request := map[string]any{"message": "Hello"}
reqJSON, _ := json.Marshal(request)
response, err = plugin.Call("echo", string(reqJSON))
if err != nil {
	log.Fatal(err)
}

var result map[string]any
json.Unmarshal([]byte(response), &result)
fmt.Printf("Message: %s\n", result["message"])
```

## Type-Safe Calls with Structs

`CallTyped` marshals the request with `json.Marshal`, calls the plugin, and unmarshals the response with `json.Unmarshal`. Struct fields should have `json` tags matching the plugin's expected field names:

```go
type EchoRequest struct {
	Message string `json:"message"`
}
type EchoResponse struct {
	Message string `json:"message"`
	Length  int    `json:"length"`
}

var resp EchoResponse
err := plugin.CallTyped("echo", &EchoRequest{Message: "Hello, Go!"}, &resp)
if err != nil {
	log.Fatal(err)
}
fmt.Printf("Message: %s, Length: %d\n", resp.Message, resp.Length)
```

## Configuration

Configuration uses the functional options pattern:

```go
plugin, err := rustbridge.Load(pluginPath,
	rustbridge.WithLogLevel(rustbridge.LogLevelDebug),
	rustbridge.WithWorkerThreads(4),
	rustbridge.WithMaxConcurrentOps(100),
	rustbridge.WithShutdownTimeout(5000),
	rustbridge.WithData("db_url", "postgres://localhost/mydb"),
	rustbridge.WithInitParam("region", "us-east-1"),
)
```

| Option | Description | Default |
|--------|-------------|---------|
| `WithLogLevel(level)` | Set the log verbosity | `LogLevelInfo` |
| `WithWorkerThreads(n)` | Number of Tokio worker threads | runtime default |
| `WithMaxConcurrentOps(n)` | Max concurrent operations (0 = unlimited) | 1000 |
| `WithShutdownTimeout(ms)` | Shutdown timeout in milliseconds | 5000 |
| `WithLogHandler(handler)` | Callback for receiving log messages | none |
| `WithData(key, value)` | Custom configuration key-value pair | none |
| `WithInitParam(key, value)` | Initialization parameter | none |

## Logging

### Custom Log Handler

```go
handler := func(level rustbridge.LogLevel, target string, message string) {
	fmt.Printf("[%s] %s: %s\n", level, target, message)
}

plugin, err := rustbridge.Load(pluginPath,
	rustbridge.WithLogLevel(rustbridge.LogLevelDebug),
	rustbridge.WithLogHandler(handler),
)
```

### slog Adapter

The package provides `SlogLogHandler` for the standard `log/slog` package. It maps rustbridge levels to slog levels (`LogLevelTrace` to `LevelDebug - 4`, `LogLevelDebug` to `LevelDebug`, and so on):

```go
plugin, err := rustbridge.Load(pluginPath,
	rustbridge.WithLogLevel(rustbridge.LogLevelDebug),
	rustbridge.WithLogHandler(rustbridge.SlogLogHandler(slog.Default())),
)
```

Log level can be changed at runtime with `plugin.SetLogLevel(rustbridge.LogLevelError)`.

## Binary Transport (Advanced)

For performance-critical paths, binary transport avoids JSON serialization overhead by using fixed-layout C structs. The package provides `SmallRequestRaw` (76 bytes) and `SmallResponseRaw` (80 bytes) with compile-time size assertions matching the Rust `#[repr(C)]` layout.

```go
import "unsafe"

// Create request using the convenience constructor
req := rustbridge.NewSmallRequest("cache_key", 0)

// Check support before calling
if !plugin.HasBinaryTransport() {
	log.Fatal("binary transport not available")
}

data, err := plugin.CallRaw(
	rustbridge.MsgBenchSmall,
	unsafe.Pointer(&req),
	int(unsafe.Sizeof(req)),
)
if err != nil {
	log.Fatal(err)
}

// Interpret the response bytes as the expected struct
resp := (*rustbridge.SmallResponseRaw)(unsafe.Pointer(&data[0]))
fmt.Printf("Value: %s\n", resp.ValueString())
fmt.Printf("TTL: %d seconds\n", resp.TtlSeconds)
fmt.Printf("Cache hit: %v\n", resp.CacheHit != 0)
```

## Error Handling

Plugin operations return errors that may be of type `*PluginError`. Use `IsPluginError` to inspect:

```go
_, err := plugin.Call("invalid.type", `{}`)
if err != nil {
	pe, ok := rustbridge.IsPluginError(err)
	if ok {
		fmt.Printf("Error code: %d (%s)\n", pe.Code, pe.Code)
		switch pe.Code {
		case rustbridge.ErrorCodeUnknownMessageType:
			fmt.Println("Unknown message type")
		case rustbridge.ErrorCodeConcurrencyLimit:
			fmt.Println("Too many concurrent requests")
		default:
			fmt.Printf("Unexpected: %s\n", pe.Message)
		}
	} else {
		fmt.Printf("Non-plugin error: %v\n", err)
	}
}
```

### Error Code Reference

| Code | Constant | Description |
|------|----------|-------------|
| 0 | `ErrorCodeSuccess` | No error |
| 1 | `ErrorCodeInvalidState` | Plugin lifecycle state mismatch |
| 2 | `ErrorCodeInitializationFailed` | Plugin initialization failed |
| 3 | `ErrorCodeShutdownFailed` | Failed during shutdown |
| 4 | `ErrorCodeConfigError` | Invalid configuration |
| 5 | `ErrorCodeSerializationError` | JSON marshal/unmarshal failure |
| 6 | `ErrorCodeUnknownMessageType` | Unrecognized type tag |
| 7 | `ErrorCodeHandlerError` | Business logic error |
| 8 | `ErrorCodeRuntimeError` | Async runtime error |
| 9 | `ErrorCodeCancelled` | Request was cancelled |
| 10 | `ErrorCodeTimeout` | Operation timed out |
| 11 | `ErrorCodeInternal` | Internal framework error (or panic) |
| 12 | `ErrorCodeFfiError` | FFI boundary error |
| 13 | `ErrorCodeTooManyRequests` | Concurrency limit exceeded |

## Concurrent Usage

All `Plugin` methods are safe for concurrent use from multiple goroutines (internal `sync.RWMutex`):

```go
var wg sync.WaitGroup
for i := 0; i < 10; i++ {
	wg.Add(1)
	go func(id int) {
		defer wg.Done()
		for j := 0; j < 100; j++ {
			req := fmt.Sprintf(`{"message": "goroutine %d call %d"}`, id, j)
			_, err := plugin.Call("echo", req)
			if err != nil {
				log.Printf("goroutine %d: %v", id, err)
			}
		}
	}(i)
}
wg.Wait()
```

No external synchronization is needed. If the concurrency limit is reached, excess calls return `ErrorCodeConcurrencyLimit` rather than blocking.

## Monitoring

### Lifecycle State

```go
state := plugin.State()
fmt.Printf("State: %s\n", state)    // e.g., "Active"
state.CanHandleRequests()            // true if Active
state.IsTerminal()                   // true if Stopped or Failed
```

| State | Value | Description |
|-------|-------|-------------|
| `StateInstalled` | 0 | Plugin created but not started |
| `StateStarting` | 1 | Plugin is initializing |
| `StateActive` | 2 | Plugin is ready for requests |
| `StateStopping` | 3 | Plugin is shutting down |
| `StateStopped` | 4 | Plugin has stopped cleanly |
| `StateFailed` | 5 | Plugin encountered a fatal error |

### Rejected Request Count

```go
rejected := plugin.RejectedRequestCount()
if rejected > 0 {
	log.Printf("Warning: %d requests rejected due to concurrency limit", rejected)
}
```

A rising count indicates the plugin is under too much load or the concurrency limit should be raised.

## Complete Example

```go
package main

import (
	"fmt"
	"log"
	"log/slog"

	rustbridge "github.com/jrobhoward/rustbridge-go"
)

type AddRequest struct {
	A int `json:"a"`
	B int `json:"b"`
}
type AddResponse struct {
	Result int `json:"result"`
}

func main() {
	plugin, err := rustbridge.Load(
		"target/release/libhello_plugin.so",
		rustbridge.WithLogLevel(rustbridge.LogLevelInfo),
		rustbridge.WithWorkerThreads(4),
		rustbridge.WithMaxConcurrentOps(200),
		rustbridge.WithLogHandler(rustbridge.SlogLogHandler(slog.Default())),
	)
	if err != nil {
		log.Fatal(err)
	}
	defer plugin.Close()

	fmt.Printf("Plugin state: %s\n", plugin.State())

	var resp AddResponse
	err = plugin.CallTyped("math.add", &AddRequest{A: 42, B: 58}, &resp)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("42 + 58 = %d\n", resp.Result)

	response, err := plugin.Call("echo", `{"message": "Hello from Go!"}`)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Echo: %s\n", response)
	fmt.Printf("Rejected requests: %d\n", plugin.RejectedRequestCount())
}
```

## Performance Notes

Go benefits from low CGo overhead and direct in-process FFI via dlopen:

| Transport | Latency (Linux x86-64) |
|-----------|----------------------|
| Binary | 862 ns |
| JSON | 5.68 us |

Binary transport is roughly **6.6x faster** than JSON.

For performance-critical applications, consider:
- Using binary transport (`CallRaw`) for hot paths
- Tuning `WithWorkerThreads` based on CPU core count
- Setting `WithMaxConcurrentOps` high enough to avoid rejected requests under peak load
- Running benchmarks on your hardware with `go test -bench .`

## Testing

Build the hello-plugin before running tests:

```bash
# Build the native library
cargo build --release -p hello-plugin

# Run all tests
cd rustbridge-go
go test -v ./...

# Run tests matching a pattern
go test -v -run "TestCall___EchoMessage" ./...

# Run benchmarks
go test -bench . -benchmem ./...
```

Tests follow the `SubjectUnderTest___Condition___ExpectedResult` naming convention with triple underscores, consistent with the rustbridge project conventions.

## Related Documentation

- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership patterns
- [../ERROR_HANDLING.md](../ERROR_HANDLING.md) - Error codes and handling patterns
- [../ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture and design decisions
