# Testing Conventions: Go

## Overview

The Go consumer uses the standard `testing` package with no external dependencies. Tests are split into:
- **Unit tests** for pure Go types (no plugin needed)
- **Integration tests** against the hello-plugin (requires `cargo build --release -p hello-plugin`)
- **Benchmarks** for measuring call latency and allocation overhead

## Running Tests

```bash
# From rustbridge-go/
go test -v ./...                                                # All tests
go test -run 'Test(LogLevel|Config|ResponseEnvelope)' -v ./...  # Unit tests only
go test -bench=. -benchmem -count=3 ./...                       # Benchmarks
make test                                                       # Build plugin + test
make bench                                                      # Build plugin + benchmark
```

Integration tests require the hello-plugin to be built. Tests skip automatically with `t.Skip()` if the plugin is not found.

## Test Naming Convention

Follow the project-wide triple-underscore convention:

```
TestSubjectUnderTest___Condition___ExpectedResult
```

Examples:

```go
func TestCall___EchoMessage___ReturnsResponse(t *testing.T) {
    plugin := loadTestPlugin(t)

    response, err := plugin.Call("echo", `{"message": "Hello from Go!"}`)

    if err != nil {
        t.Fatalf("Call error: %v", err)
    }
    // ... assertions
}
```

Benchmark names follow the same convention:

```go
func BenchmarkCall___SmallEcho(b *testing.B) { ... }
```

## Test Structure

### Unit Tests (no plugin needed)

- `log_level_test.go` — LogLevel String/Parse round-trips
- `lifecycle_state_test.go` — LifecycleState predicates
- `errors_test.go` — PluginError formatting, type assertions
- `config_test.go` — Config defaults, JSON serialization
- `response_envelope_test.go` — ResponseEnvelope parsing

### Integration Tests (require hello-plugin)

- `plugin_test.go` — Full integration against hello-plugin
- `binary_transport_test.go` — Binary transport with C struct FFI

### Benchmarks

- `benchmark_test.go` — JSON and binary call latency

## Test Helpers

```go
// testutil_test.go

// findHelloPlugin walks up from the test file to find target/release/libhello_plugin.so
func findHelloPlugin(t *testing.T) string { ... }

// loadTestPlugin loads the plugin or skips the test if not built
func loadTestPlugin(t *testing.T, opts ...Option) *Plugin { ... }
```

## Arrange-Act-Assert

Use blank lines to separate sections. No inline comments for sections:

```go
func TestCall___MathAdd___ReturnsSum(t *testing.T) {
    plugin := loadTestPlugin(t)

    response, err := plugin.Call("math.add", `{"a": 10, "b": 32}`)

    if err != nil {
        t.Fatalf("Call error: %v", err)
    }
    var result map[string]any
    if err := json.Unmarshal([]byte(response), &result); err != nil {
        t.Fatalf("json.Unmarshal error: %v", err)
    }
    if result["result"] != float64(42) {
        t.Errorf("result = %v, want 42", result["result"])
    }
}
```

## Error Handling Tests

Errors are returned as `*PluginError` implementing the `error` interface:

```go
func TestCall___UnknownType___ReturnsErrorCode6(t *testing.T) {
    plugin := loadTestPlugin(t)

    _, err := plugin.Call("nonexistent", `{}`)

    pe, ok := IsPluginError(err)
    if !ok {
        t.Fatalf("expected PluginError, got %T", err)
    }
    if pe.Code != ErrorCodeUnknownMessageType {
        t.Errorf("Code = %d, want %d", pe.Code, ErrorCodeUnknownMessageType)
    }
}
```

## Dependencies

- Go 1.21+ (for `log/slog`)
- C compiler (for CGo — `gcc` on Linux, `clang` on macOS)
- hello-plugin built in release mode (`cargo build --release -p hello-plugin`)
