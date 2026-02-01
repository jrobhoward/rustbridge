# Section 1: C# Consumer

In this section, you'll implement synchronized plugin access in C# using `BlockingCollection<T>` and `Task<T>`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new sync-demo --all`
2. Add the sleep handler to `src/lib.rs`
3. Build the plugin and create the bundle
4. Copy the bundle to `consumers/csharp/`

## Verify the Generated Consumer

The scaffolded C# consumer is in `consumers/csharp/`. Let's verify it works:

```bash
cd ~/rustbridge-workspace/sync-demo/consumers/csharp
dotnet run
```

You should see:

```
Response: Hello from C#!
Length: 14
```

## Understanding the Generated Code

Look at `Program.cs`:

```csharp
using System.Text.Json;
using RustBridge;
using RustBridge.Native;

var jsonOptions = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
};

var bundlePath = "sync-demo-0.1.0.rbp";

using var bundleLoader = BundleLoader.Create()
    .WithBundlePath(bundlePath)
    .WithSignatureVerification(false)
    .Build();
var libraryPath = bundleLoader.ExtractLibrary();

using var plugin = NativePluginLoader.Load(libraryPath);

var request = new EchoRequest("Hello from C#!");
var requestJson = JsonSerializer.Serialize(request, jsonOptions);

var responseJson = plugin.Call("echo", requestJson);
var response = JsonSerializer.Deserialize<EchoResponse>(responseJson, jsonOptions);

Console.WriteLine($"Response: {response?.Message}");
Console.WriteLine($"Length: {response?.Length}");

record EchoRequest(string Message);
record EchoResponse(string Message, int Length);
```

Key points:
- `BundleLoader.Create()...Build()` extracts the native library from the bundle
- `NativePluginLoader.Load()` loads the plugin
- `plugin.Call()` makes JSON-based calls
- Records are defined after top-level statements

## Add the SynchronizedPlugin Wrapper

Create a new file `SynchronizedPlugin.cs`:

```csharp
using System.Collections.Concurrent;
using RustBridge;

namespace SyncDemo;

/// <summary>
/// A work item representing a pending plugin call.
/// </summary>
internal sealed class PluginWorkItem
{
    public required string TypeTag { get; init; }
    public required string Request { get; init; }
    public required TaskCompletionSource<string> Completion { get; init; }
}

/// <summary>
/// Thread-safe wrapper that serializes all plugin calls through a single worker thread.
/// </summary>
public sealed class SynchronizedPlugin : IDisposable
{
    private readonly IPlugin _plugin;
    private readonly BlockingCollection<PluginWorkItem> _workQueue;
    private readonly Thread _workerThread;
    private readonly CancellationTokenSource _shutdownCts;
    private volatile bool _disposed;

    public SynchronizedPlugin(IPlugin plugin, int maxQueueSize = 100)
    {
        _plugin = plugin;
        _shutdownCts = new CancellationTokenSource();

        _workQueue = new BlockingCollection<PluginWorkItem>(
            new ConcurrentQueue<PluginWorkItem>(),
            boundedCapacity: maxQueueSize
        );

        _workerThread = new Thread(WorkerLoop)
        {
            Name = "SynchronizedPlugin-Worker",
            IsBackground = true
        };
        _workerThread.Start();
    }

    public int PendingCount => _workQueue.Count;

    public async Task<string> CallAsync(
        string typeTag,
        string request,
        CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        var workItem = new PluginWorkItem
        {
            TypeTag = typeTag,
            Request = request,
            Completion = new TaskCompletionSource<string>(
                TaskCreationOptions.RunContinuationsAsynchronously)
        };

        using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(
            cancellationToken, _shutdownCts.Token);

        try
        {
            _workQueue.Add(workItem, linkedCts.Token);
        }
        catch (OperationCanceledException) when (_shutdownCts.IsCancellationRequested)
        {
            throw new ObjectDisposedException(nameof(SynchronizedPlugin));
        }

        return await workItem.Completion.Task.WaitAsync(cancellationToken);
    }

    public string Call(string typeTag, string request)
    {
        return CallAsync(typeTag, request).GetAwaiter().GetResult();
    }

    public async Task<TResponse> CallAsync<TRequest, TResponse>(
        string typeTag,
        TRequest request,
        CancellationToken cancellationToken = default)
    {
        var requestJson = System.Text.Json.JsonSerializer.Serialize(request, JsonOptions);
        var responseJson = await CallAsync(typeTag, requestJson, cancellationToken);
        return System.Text.Json.JsonSerializer.Deserialize<TResponse>(responseJson, JsonOptions)
            ?? throw new PluginException("Failed to deserialize response");
    }

    // SnakeCaseLower converts "DurationMs" to "duration_ms" to match Rust's serde
    private static readonly System.Text.Json.JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = System.Text.Json.JsonNamingPolicy.SnakeCaseLower
    };

    private void WorkerLoop()
    {
        try
        {
            foreach (var workItem in _workQueue.GetConsumingEnumerable(_shutdownCts.Token))
            {
                try
                {
                    var response = _plugin.Call(workItem.TypeTag, workItem.Request);
                    workItem.Completion.TrySetResult(response);
                }
                catch (OperationCanceledException)
                {
                    workItem.Completion.TrySetCanceled();
                }
                catch (Exception ex)
                {
                    workItem.Completion.TrySetException(ex);
                }
            }
        }
        catch (OperationCanceledException)
        {
            // Shutdown requested
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        _shutdownCts.Cancel();
        _workQueue.CompleteAdding();
        _workerThread.Join(TimeSpan.FromSeconds(5));

        while (_workQueue.TryTake(out var workItem))
        {
            workItem.Completion.TrySetException(
                new ObjectDisposedException(nameof(SynchronizedPlugin)));
        }

        _workQueue.Dispose();
        _shutdownCts.Dispose();
        _plugin.Dispose();
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
        {
            throw new ObjectDisposedException(nameof(SynchronizedPlugin));
        }
    }
}
```

## Update Program.cs

Replace `Program.cs` with the synchronized demo:

```csharp
using RustBridge;
using RustBridge.Native;
using SyncDemo;

var bundlePath = "sync-demo-0.1.0.rbp";

using var bundleLoader = BundleLoader.Create()
    .WithBundlePath(bundlePath)
    .WithSignatureVerification(false)
    .Build();
var libraryPath = bundleLoader.ExtractLibrary();

var plugin = NativePluginLoader.Load(libraryPath);

// Wrap with synchronized access (queue size = 5 for demo)
using var syncPlugin = new SynchronizedPlugin(plugin, maxQueueSize: 5);

Console.WriteLine("=== Synchronized Plugin Demo ===\n");

// Demo 1: Sequential calls
Console.WriteLine("Demo 1: Sequential calls");
for (int i = 0; i < 3; i++)
{
    var response = await syncPlugin.CallAsync<EchoRequest, EchoResponse>(
        "echo",
        new EchoRequest($"Message {i}"));
    Console.WriteLine($"  Echo: {response.Message} (len={response.Length})");
}

// Demo 2: Concurrent calls showing serialization
Console.WriteLine("\nDemo 2: Concurrent calls (observe serialization)");
var tasks = new List<Task>();
var stopwatch = System.Diagnostics.Stopwatch.StartNew();

for (int i = 0; i < 10; i++)
{
    var id = i;
    tasks.Add(Task.Run(async () =>
    {
        Console.WriteLine($"  [{id}] Submitting (queue: {syncPlugin.PendingCount})");

        await syncPlugin.CallAsync<SleepRequest, SleepResponse>(
            "sleep",
            new SleepRequest(100));

        Console.WriteLine($"  [{id}] Completed after {stopwatch.ElapsedMilliseconds}ms");
    }));
}

await Task.WhenAll(tasks);
Console.WriteLine($"\nTotal time: {stopwatch.ElapsedMilliseconds}ms");
Console.WriteLine("  (Expected ~1000ms for 10 x 100ms if serialized)");

// Demo 3: Backpressure
Console.WriteLine("\nDemo 3: Backpressure (queue size = 5)");
stopwatch.Restart();

var pressureTasks = new List<Task>();
for (int i = 0; i < 20; i++)
{
    var id = i;
    pressureTasks.Add(Task.Run(async () =>
    {
        var submitTime = stopwatch.ElapsedMilliseconds;

        await syncPlugin.CallAsync<SleepRequest, SleepResponse>(
            "sleep",
            new SleepRequest(50));

        var completeTime = stopwatch.ElapsedMilliseconds;
        Console.WriteLine($"  [{id:D2}] Submit@{submitTime}ms, Complete@{completeTime}ms");
    }));
}

await Task.WhenAll(pressureTasks);
Console.WriteLine($"\nTotal time: {stopwatch.ElapsedMilliseconds}ms");

Console.WriteLine("\n=== Demo Complete ===");

// Record types (must be after top-level statements)
record EchoRequest(string Message);
record EchoResponse(string Message, int Length);
record SleepRequest(long DurationMs);
record SleepResponse(long SleptMs);
```

## Run the Demo

```bash
dotnet run
```

Expected output:

```
=== Synchronized Plugin Demo ===

Demo 1: Sequential calls
  Echo: Message 0 (len=9)
  Echo: Message 1 (len=9)
  Echo: Message 2 (len=9)

Demo 2: Concurrent calls (observe serialization)
  [0] Submitting (queue: 0)
  [1] Submitting (queue: 1)
  [2] Submitting (queue: 2)
  ...
  [0] Completed after 102ms
  [1] Completed after 204ms
  ...

Total time: 1015ms
  (Expected ~1000ms for 10 x 100ms if serialized)

Demo 3: Backpressure (queue size = 5)
  [00] Submit@0ms, Complete@52ms
  [01] Submit@0ms, Complete@103ms
  ...
  [18] Submit@650ms, Complete@1002ms
  [19] Submit@700ms, Complete@1052ms

Total time: 1052ms

=== Demo Complete ===
```

## Key Observations

### Serialization

Even though 10 requests are submitted concurrently, they complete sequentially (~100ms apart).
Total time is approximately `10 × 100ms = 1000ms`.

### Backpressure

With a queue size of 5:
- First 6 requests submit immediately (1 processing + 5 queued)
- Requests 6+ block until queue has space
- Submit times show delays as callers wait for queue capacity

## Understanding the Implementation

### BlockingCollection&lt;T&gt;

```csharp
_workQueue = new BlockingCollection<PluginWorkItem>(
    new ConcurrentQueue<PluginWorkItem>(),
    boundedCapacity: maxQueueSize
);
```

- **Bounded capacity**: Limits queue size, enabling backpressure
- **Thread-safe**: Safe for multiple producers and single consumer
- **Blocking add**: `Add()` blocks when queue is full

### TaskCompletionSource&lt;T&gt;

```csharp
Completion = new TaskCompletionSource<string>(
    TaskCreationOptions.RunContinuationsAsynchronously)
```

- **Bridge**: Connects the caller's `await` to the worker's completion
- **Async continuations**: Prevents deadlocks from synchronous continuations
- **Error propagation**: `TrySetException` delivers errors to the caller

## Error Handling

Errors in plugin calls are propagated to the caller:

```csharp
try
{
    var response = await syncPlugin.CallAsync("invalid.tag", "{}");
}
catch (PluginException ex)
{
    Console.WriteLine($"Plugin error: {ex.Message}");
}
```

## What's Next?

Continue to the Java/JNI implementation.

[Continue to Section 2: Java/JNI Consumer →](./02-java-jni-consumer.md)
