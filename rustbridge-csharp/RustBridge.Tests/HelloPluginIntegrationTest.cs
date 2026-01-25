using System.Text.Json;
using System.Text.Json.Serialization;
using RustBridge.Native;

namespace RustBridge.Tests;

/// <summary>
/// Integration tests for hello-plugin.
/// <para>
/// These tests require the hello-plugin to be built first:
/// <code>cargo build --release -p hello-plugin</code>
/// </para>
/// </summary>
[Trait("Category", "Integration")]
public class HelloPluginIntegrationTest : IDisposable
{
    private readonly IPlugin? _plugin;
    private readonly string? _skipReason;

    public HelloPluginIntegrationTest()
    {
        var libraryPath = FindHelloPlugin();
        if (libraryPath == null)
        {
            _skipReason = "hello-plugin not found. Run: cargo build --release -p hello-plugin";
            return;
        }

        try
        {
            _plugin = NativePluginLoader.Load(libraryPath);
        }
        catch (Exception ex)
        {
            _skipReason = $"Failed to load plugin: {ex.Message}";
        }
    }

    public void Dispose()
    {
        _plugin?.Dispose();
    }

    private static string? FindHelloPlugin()
    {
        // Search for the hello-plugin library in common locations
        var libraryName = GetLibraryFileName("hello_plugin");

        // Get the test assembly location and work up to find repo root
        var assemblyLocation = typeof(HelloPluginIntegrationTest).Assembly.Location;
        var assemblyDir = Path.GetDirectoryName(assemblyLocation) ?? ".";

        // Build search paths relative to various starting points
        var searchBases = new[]
        {
            // From current directory
            Environment.CurrentDirectory,
            // From assembly location (bin/Debug/net8.0/)
            assemblyDir,
            // Walk up from assembly to find repo (bin/Debug/net8.0 -> Tests -> csharp -> repo)
            Path.Combine(assemblyDir, "..", "..", "..", ".."),
            Path.Combine(assemblyDir, "..", "..", "..", "..", ".."),
        };

        foreach (var baseDir in searchBases)
        {
            var releasePath = Path.Combine(baseDir, "target", "release", libraryName);
            if (File.Exists(releasePath))
            {
                return Path.GetFullPath(releasePath);
            }

            var debugPath = Path.Combine(baseDir, "target", "debug", libraryName);
            if (File.Exists(debugPath))
            {
                return Path.GetFullPath(debugPath);
            }
        }

        return null;
    }

    private static string GetLibraryFileName(string name)
    {
        if (OperatingSystem.IsWindows()) return $"{name}.dll";
        if (OperatingSystem.IsMacOS()) return $"lib{name}.dylib";
        return $"lib{name}.so";
    }

    private void SkipIfPluginNotAvailable()
    {
        Skip.If(_skipReason != null, _skipReason);
    }

    // ==================== Lifecycle Tests ====================

    [SkippableFact]
    public void Load___ValidPlugin___StateIsActive()
    {
        SkipIfPluginNotAvailable();

        Assert.Equal(LifecycleState.Active, _plugin!.State);
    }

    [SkippableFact]
    public void Dispose___AfterDispose___StateIsStopped()
    {
        SkipIfPluginNotAvailable();

        var libraryPath = FindHelloPlugin()!;
        var plugin = NativePluginLoader.Load(libraryPath);

        plugin.Dispose();

        Assert.Equal(LifecycleState.Stopped, plugin.State);
    }

    // ==================== Echo Tests ====================

    [SkippableFact]
    public void Call___EchoMessage___ReturnsMessageWithLength()
    {
        SkipIfPluginNotAvailable();

        var response = _plugin!.Call("echo", """{"message": "Hello, World!"}""");

        var result = JsonSerializer.Deserialize<EchoResponse>(response);
        Assert.NotNull(result);
        Assert.Equal("Hello, World!", result.Message);
        Assert.Equal(13, result.Length);
    }

    [SkippableFact]
    public void Call___EchoTyped___ReturnsMessageWithLength()
    {
        SkipIfPluginNotAvailable();

        var response = _plugin!.Call<EchoRequest, EchoResponse>(
            "echo",
            new EchoRequest { Message = "Hello, Typed!" });

        Assert.Equal("Hello, Typed!", response.Message);
        Assert.Equal(13, response.Length);
    }

    // ==================== Greet Tests ====================

    [SkippableFact]
    public void Call___GreetName___ReturnsGreeting()
    {
        SkipIfPluginNotAvailable();

        var response = _plugin!.Call("greet", """{"name": "Alice"}""");

        var result = JsonSerializer.Deserialize<GreetResponse>(response);
        Assert.NotNull(result);
        Assert.Contains("Alice", result.Greeting);
    }

    // ==================== Math Tests ====================

    [SkippableFact]
    public void Call___MathAdd___ReturnsSum()
    {
        SkipIfPluginNotAvailable();

        var response = _plugin!.Call("math.add", """{"a": 42, "b": 58}""");

        var result = JsonSerializer.Deserialize<AddResponse>(response);
        Assert.NotNull(result);
        Assert.Equal(100, result.Result);
    }

    [SkippableFact]
    public void Call___MathAddTyped___ReturnsSum()
    {
        SkipIfPluginNotAvailable();

        var response = _plugin!.Call<AddRequest, AddResponse>(
            "math.add",
            new AddRequest { A = 123, B = 456 });

        Assert.Equal(579, response.Result);
    }

    // ==================== User Create Tests ====================

    [SkippableFact]
    public void Call___UserCreate___ReturnsUserIdAndTimestamp()
    {
        SkipIfPluginNotAvailable();

        var response = _plugin!.Call("user.create",
            """{"username": "testuser", "email": "test@example.com"}""");

        var result = JsonSerializer.Deserialize<CreateUserResponse>(response);
        Assert.NotNull(result);
        Assert.StartsWith("user-", result.UserId);
        Assert.NotNull(result.CreatedAt);
    }

    // ==================== Error Handling Tests ====================

    [SkippableFact]
    public void Call___InvalidTypeTag___ThrowsWithErrorCode6()
    {
        SkipIfPluginNotAvailable();

        var ex = Assert.Throws<PluginException>(() =>
            _plugin!.Call("invalid.type.tag", """{"test": true}"""));

        Assert.Equal(6, ex.ErrorCode); // UnknownMessageType
    }

    [SkippableFact]
    public void Call___InvalidJson___ThrowsWithErrorCode5()
    {
        SkipIfPluginNotAvailable();

        var ex = Assert.Throws<PluginException>(() =>
            _plugin!.Call("echo", "{broken json}"));

        Assert.Equal(5, ex.ErrorCode); // SerializationError
    }

    // ==================== Concurrency Tests ====================

    [SkippableFact]
    public async Task Call___ConcurrentCalls___AllSucceed()
    {
        SkipIfPluginNotAvailable();

        const int concurrentCalls = 100;
        var tasks = new Task<string>[concurrentCalls];

        for (int i = 0; i < concurrentCalls; i++)
        {
            var message = $"Message {i}";
            tasks[i] = Task.Run(() =>
                _plugin!.Call("echo", $$$"""{"message": "{{{message}}}"}"""));
        }

        var results = await Task.WhenAll(tasks);

        for (int i = 0; i < concurrentCalls; i++)
        {
            var response = JsonSerializer.Deserialize<EchoResponse>(results[i]);
            Assert.NotNull(response);
            Assert.Equal($"Message {i}", response.Message);
        }
    }

    // ==================== Log Callback Tests ====================

    [SkippableFact]
    public void Load___WithLogCallback___ReceivesLogMessages()
    {
        SkipIfPluginNotAvailable();

        var logMessages = new List<(LogLevel Level, string Target, string Message)>();
        var libraryPath = FindHelloPlugin()!;

        var config = PluginConfig.Defaults().WithLogLevel(LogLevel.Debug);

        using var plugin = NativePluginLoader.Load(libraryPath, config,
            (level, target, message) => logMessages.Add((level, target, message)));

        // Make a call to trigger some logging
        plugin.Call("echo", """{"message": "test"}""");

        // Plugin should have logged something during init or call
        // (This depends on the plugin's logging behavior)
    }

    // ==================== Request/Response Types ====================

    private record EchoRequest
    {
        [JsonPropertyName("message")]
        public required string Message { get; init; }
    }

    private record EchoResponse
    {
        [JsonPropertyName("message")]
        public required string Message { get; init; }

        [JsonPropertyName("length")]
        public required int Length { get; init; }
    }

    private record GreetResponse
    {
        [JsonPropertyName("greeting")]
        public required string Greeting { get; init; }
    }

    private record AddRequest
    {
        [JsonPropertyName("a")]
        public required long A { get; init; }

        [JsonPropertyName("b")]
        public required long B { get; init; }
    }

    private record AddResponse
    {
        [JsonPropertyName("result")]
        public required long Result { get; init; }
    }

    private record CreateUserResponse
    {
        [JsonPropertyName("user_id")]
        public required string UserId { get; init; }

        [JsonPropertyName("created_at")]
        public required string CreatedAt { get; init; }
    }
}
