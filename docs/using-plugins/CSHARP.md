# Getting Started: C#

This guide walks you through using rustbridge plugins from C# using P/Invoke.

## Prerequisites

- **.NET 8.0 or later** - Required for latest features
  ```bash
  dotnet --version  # Should be >= 8.0
  ```
- **A rustbridge plugin** - Either a `.rbp` bundle or native library

## Add Package References

### Using Package Reference

```xml
<ItemGroup>
    <PackageReference Include="RustBridge.Core" Version="0.7.0" />
    <PackageReference Include="RustBridge.Native" Version="0.7.0" />
</ItemGroup>
```

### Using .NET CLI

```bash
dotnet add package RustBridge.Core
dotnet add package RustBridge.Native
```

## Local Development

When working with rustbridge source code (not published to NuGet), reference the local projects directly:

```xml
<ItemGroup>
    <ProjectReference Include="../rustbridge-csharp/RustBridge.Core/RustBridge.Core.csproj" />
    <ProjectReference Include="../rustbridge-csharp/RustBridge.Native/RustBridge.Native.csproj" />
</ItemGroup>
```

Or build and create local NuGet packages:

```bash
cd rustbridge-csharp
dotnet build
dotnet pack -o ./packages
```

Then add the local feed to your NuGet.config:

```xml
<configuration>
  <packageSources>
    <add key="local" value="./packages" />
  </packageSources>
</configuration>
```

## Loading a Plugin

### From Bundle (Recommended)

```csharp
using RustBridge.Core;
using RustBridge.Native;

// Load bundle and extract library for current platform
using var bundleLoader = BundleLoader.Create()
    .WithBundlePath("my-plugin-1.0.0.rbp")
    .WithSignatureVerification(false)  // Set true for production
    .Build();

string libraryPath = bundleLoader.ExtractLibrary();

// Load the plugin
using var plugin = NativePluginLoader.Load(libraryPath);

string response = plugin.Call("echo", """{"message": "Hello"}""");
Console.WriteLine(response);
```

### From Raw Library

```csharp
using RustBridge.Native;

// Platform-specific path
string pluginPath = "target/release/libmyplugin.so";  // Linux
// string pluginPath = "target/release/libmyplugin.dylib";  // macOS
// string pluginPath = "target/release/myplugin.dll";  // Windows

using var plugin = NativePluginLoader.Load(pluginPath);

string response = plugin.Call("echo", """{"message": "Hello"}""");
Console.WriteLine(response);
```

## Making JSON Calls

```csharp
using System.Text.Json;

using var plugin = NativePluginLoader.Load(pluginPath);

// Simple call
string response = plugin.Call("echo", """{"message": "Hello, World!"}""");
Console.WriteLine(response);

// With System.Text.Json for type-safe serialization
record EchoRequest(string Message);
record EchoResponse(string Message, int Length);

var request = new EchoRequest("Hello");
string requestJson = JsonSerializer.Serialize(request);

string responseJson = plugin.Call("echo", requestJson);
var parsedResponse = JsonSerializer.Deserialize<EchoResponse>(responseJson);

Console.WriteLine($"Length: {parsedResponse?.Length}");
```

## Configuration

```csharp
using RustBridge.Core;

var config = PluginConfig.Defaults()
    .WithLogLevel(LogLevel.Debug)
    .WorkerThreads(4)
    .MaxConcurrentOps(100)
    .ShutdownTimeoutMs(5000);

using var plugin = NativePluginLoader.Load(pluginPath, config);
```

## Logging

```csharp
using RustBridge.Core;

void LogCallback(LogLevel level, string target, string message)
{
    Console.WriteLine($"[{level}] {target}: {message}");
}

using var plugin = NativePluginLoader.Load(pluginPath, config, LogCallback);

// Change log level dynamically
plugin.SetLogLevel(LogLevel.Debug);
```

## Binary Transport (Advanced)

For performance-critical paths, use binary transport with C# structs.

### Define Structs

```csharp
using System.Runtime.InteropServices;

public const uint MSG_ECHO = 1;

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public unsafe struct EchoRequestRaw : IBinaryStruct
{
    public byte Version;
    private fixed byte _reserved[3];
    private fixed byte _message[256];
    public uint MessageLen;

    public int ByteSize => 264;

    public void SetMessage(string msg)
    {
        var bytes = System.Text.Encoding.UTF8.GetBytes(msg);
        var len = Math.Min(bytes.Length, 256);
        fixed (byte* ptr = _message)
        {
            Marshal.Copy(bytes, 0, (IntPtr)ptr, len);
        }
        MessageLen = (uint)len;
    }
}

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public unsafe struct EchoResponseRaw
{
    public byte Version;
    private fixed byte _reserved[3];
    private fixed byte _message[256];
    public uint MessageLen;
    public uint Length;
}
```

### Make Binary Calls

```csharp
var request = new EchoRequestRaw { Version = 1 };
request.SetMessage("Hello");

var response = plugin.CallRaw<EchoRequestRaw, EchoResponseRaw>(MSG_ECHO, ref request);

Console.WriteLine($"Length: {response.Length}");
```

## Error Handling

```csharp
using RustBridge.Core;

try
{
    string response = plugin.Call("invalid.type", "{}");
}
catch (PluginException ex)
{
    Console.WriteLine($"Error code: {ex.ErrorCode}");
    Console.WriteLine($"Message: {ex.Message}");

    switch (ex.ErrorCode)
    {
        case 6:
            Console.WriteLine("Unknown message type");
            break;
        case 7:
            Console.WriteLine("Handler error");
            break;
        case 13:
            Console.WriteLine("Too many concurrent requests");
            break;
        default:
            Console.WriteLine("Unexpected error");
            break;
    }
}
```

## Async Usage

```csharp
// Wrap synchronous calls for async usage
public static Task<string> CallAsync(
    this IPlugin plugin,
    string typeTag,
    string request)
{
    return Task.Run(() => plugin.Call(typeTag, request));
}

// Usage
string response = await plugin.CallAsync("echo", """{"message": "Hello"}""");
```

### Concurrent Calls

```csharp
var tasks = Enumerable.Range(1, 100)
    .Select(i => plugin.CallAsync("echo", $$$"""{"message": "Message {{{i}}}"}"""))
    .ToArray();

string[] responses = await Task.WhenAll(tasks);
Console.WriteLine($"Completed {responses.Length} calls");
```

## Monitoring

```csharp
// Check plugin state
LifecycleState state = plugin.State;
Console.WriteLine($"State: {state}");  // Active

// Monitor rejected requests
ulong rejectedCount = plugin.RejectedRequestCount;
if (rejectedCount > 0)
{
    Console.WriteLine($"Rejected: {rejectedCount} requests");
}
```

## Dependency Injection

```csharp
using Microsoft.Extensions.DependencyInjection;

// Registration
services.AddSingleton<IPlugin>(sp =>
{
    var config = new PluginConfig { LogLevel = LogLevel.Info };
    return NativePluginLoader.Load("path/to/plugin.so", config);
});

// Usage
public class MyService
{
    private readonly IPlugin _plugin;

    public MyService(IPlugin plugin)
    {
        _plugin = plugin;
    }

    public async Task<EchoResponse> EchoAsync(string message)
    {
        var request = new EchoRequest(message);
        var requestJson = JsonSerializer.Serialize(request);

        var responseJson = await Task.Run(() =>
            _plugin.Call("echo", requestJson));

        return JsonSerializer.Deserialize<EchoResponse>(responseJson)!;
    }
}
```

## Platform-Specific Library Paths

```csharp
using System.Runtime.InteropServices;

string GetPluginPath()
{
    if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
    {
        return "target/release/myplugin.dll";
    }
    else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
    {
        return "target/release/libmyplugin.dylib";
    }
    else
    {
        return "target/release/libmyplugin.so";
    }
}
```

## Complete Example

```csharp
using System.Text.Json;
using RustBridge.Core;
using RustBridge.Native;

record AddRequest(long A, long B);
record AddResponse(long Result);

class Program
{
    static void Main()
    {
        var config = PluginConfig.Defaults()
            .WithLogLevel(LogLevel.Info);

        void LogCallback(LogLevel level, string target, string message)
        {
            Console.WriteLine($"[{level}] {message}");
        }

        using var plugin = NativePluginLoader.Load(
            "target/release/libcalculator_plugin.so",
            config,
            LogCallback);

        // Make typed call
        var request = new AddRequest(42, 58);
        var requestJson = JsonSerializer.Serialize(request);

        var responseJson = plugin.Call("math.add", requestJson);
        var response = JsonSerializer.Deserialize<AddResponse>(responseJson);

        Console.WriteLine($"42 + 58 = {response?.Result}");
    }
}
```

## Performance Notes

C# achieves the lowest latency among all supported languages:

| Transport | Latency (Linux) | Latency (Windows) |
|-----------|-----------------|-------------------|
| Binary | 268 ns | 326 ns |
| JSON | 2.29 μs | 2.55 μs |

Binary transport is **8.5x faster** than JSON on Linux.

## Related Documentation

- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership
- [../BENCHMARK_RESULTS.md](../BENCHMARK_RESULTS.md) - Performance data
