# Section 1: C# Consumer

In this section, you'll implement synchronized plugin access in C# using `BlockingCollection<T>` and `Task<T>`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction.

## Verify the Generated Consumer

```powershell
cd $env:USERPROFILE\rustbridge-workspace\sync-demo\consumers\csharp
dotnet run
```

> **Note**: The generated `.csproj` file references the rustbridge C# projects via `$(HOME)/rustbridge-workspace/rustbridge/...`. If your rustbridge repository is in a different location (e.g., `$(HOME)/git/rustbridge/...`), update the `ProjectReference` paths in the `.csproj` file. Also update the bundle path in `Program.cs` to match the version you created.

You should see:

```
Response: Hello from C#!
Length: 14
```

## Add the SynchronizedPlugin Wrapper

Create `SynchronizedPlugin.cs` with a thread-safe wrapper that:
- Uses `BlockingCollection<T>` for bounded queueing
- Uses `TaskCompletionSource<T>` for async completion
- Has a worker thread that processes requests sequentially

See the [Linux tutorial](../tutorials/07-backpressure-queues/01-csharp-consumer.md) for the complete C# implementation.

## Key Implementation Details

### BlockingCollection

```csharp
_workQueue = new BlockingCollection<PluginWorkItem>(
    new ConcurrentQueue<PluginWorkItem>(),
    boundedCapacity: maxQueueSize
);
```

- **Bounded capacity**: Limits queue size, enabling backpressure
- **Thread-safe**: Safe for multiple producers and single consumer
- **Blocking add**: `Add()` blocks when queue is full

### TaskCompletionSource

```csharp
Completion = new TaskCompletionSource<string>(
    TaskCreationOptions.RunContinuationsAsynchronously)
```

- **Bridge**: Connects the caller's `await` to the worker's completion
- **Async continuations**: Prevents deadlocks

## Run the Demo

```powershell
dotnet run
```

Expected output shows:
- Sequential execution of concurrent calls
- Backpressure when queue fills up
- Proper shutdown handling

## What's Next?

Continue to the Java/JNI implementation.

[Continue to Section 2: Java/JNI Consumer →](./02-java-jni-consumer.md)
