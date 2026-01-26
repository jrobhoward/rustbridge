# RustBridge C# Bindings

C# bindings for the RustBridge plugin framework.

## Requirements

- .NET 8.0 SDK or later
- A RustBridge plugin compiled as a native library (.dll on Windows, .so on Linux, .dylib on macOS)

## Projects

- **RustBridge.Core** - Core interfaces and types (IPlugin, PluginConfig, etc.)
- **RustBridge.Native** - P/Invoke-based native plugin loader
- **RustBridge.Tests** - Unit and integration tests
- **RustBridge.Benchmarks** - BenchmarkDotNet performance benchmarks

## Quick Start

```csharp
using RustBridge;
using RustBridge.Native;

// Load a plugin
using var plugin = NativePluginLoader.Load("path/to/plugin.dll");

// Make a call
var response = plugin.Call("echo", "{\"message\": \"hello\"}");
Console.WriteLine(response);

// Or with typed request/response
var result = plugin.Call<MyRequest, MyResponse>("my.operation", new MyRequest { Value = 42 });
```

## Building

```bash
dotnet build
dotnet test
```

## Benchmarks

Run performance benchmarks using BenchmarkDotNet:

```bash
# Build the hello-plugin first
cargo build --release -p hello-plugin

# Run all benchmarks
cd RustBridge.Benchmarks
dotnet run -c Release

# Run specific benchmark
dotnet run -c Release -- --filter "*TransportBenchmark*"
dotnet run -c Release -- --filter "*ThroughputBenchmark*"
dotnet run -c Release -- --filter "*ConcurrentBenchmark*"
```

Available benchmarks:
- **TransportBenchmark** - Latency comparison between JSON and binary transport
- **ThroughputBenchmark** - Operations per second for sustained load
- **ConcurrentBenchmark** - Multi-threaded scalability (100 concurrent tasks)

## Configuration

```csharp
var config = PluginConfig.Defaults()
    .WorkerThreads(4)
    .WithLogLevel(LogLevel.Debug)
    .MaxConcurrentOps(500)
    .Set("custom_key", "custom_value");

using var plugin = NativePluginLoader.Load("plugin.dll", config);
```

## Log Callback

```csharp
void HandleLog(LogLevel level, string target, string message)
{
    Console.WriteLine($"[{level}] {target}: {message}");
}

using var plugin = NativePluginLoader.Load("plugin.dll", config, HandleLog);
```

## Architecture

The C# bindings follow the same architecture as the Java bindings:

1. **Core abstractions** - Platform-independent interfaces and types
2. **Native bindings** - P/Invoke declarations for the C FFI
3. **Plugin implementation** - Manages native library lifecycle and memory

Memory follows "Rust allocates, host frees" pattern. The native plugin handle is properly disposed when the `IPlugin` is disposed.
