using System.Text.Json;

namespace RustBridge.Tests;

/// <summary>
/// Tests for <see cref="PluginConfig"/>.
/// </summary>
public class PluginConfigTests
{
    [Fact]
    public void Defaults___ReturnsConfigWithDefaultValues()
    {
        var config = PluginConfig.Defaults();

        var json = JsonDocument.Parse(config.ToJsonBytes());
        var root = json.RootElement;

        Assert.Equal("info", root.GetProperty("log_level").GetString());
        Assert.Equal(1000, root.GetProperty("max_concurrent_ops").GetInt32());
        Assert.Equal(5000, root.GetProperty("shutdown_timeout_ms").GetInt64());
    }

    [Fact]
    public void ToJsonBytes___WithWorkerThreads___IncludesWorkerThreadsInJson()
    {
        var config = PluginConfig.Defaults().WorkerThreads(4);

        var json = JsonDocument.Parse(config.ToJsonBytes());
        var root = json.RootElement;

        Assert.Equal(4, root.GetProperty("worker_threads").GetInt32());
    }

    [Fact]
    public void ToJsonBytes___WithLogLevel___IncludesLogLevelInJson()
    {
        var config = PluginConfig.Defaults().WithLogLevel(LogLevel.Debug);

        var json = JsonDocument.Parse(config.ToJsonBytes());
        var root = json.RootElement;

        Assert.Equal("debug", root.GetProperty("log_level").GetString());
    }

    [Fact]
    public void ToJsonBytes___WithCustomData___IncludesDataInJson()
    {
        var config = PluginConfig.Defaults()
            .Set("custom_key", "custom_value")
            .Set("number_key", 42);

        var json = JsonDocument.Parse(config.ToJsonBytes());
        var data = json.RootElement.GetProperty("data");

        Assert.Equal("custom_value", data.GetProperty("custom_key").GetString());
        Assert.Equal(42, data.GetProperty("number_key").GetInt32());
    }

    [Fact]
    public void ToJsonBytes___WithInitParams___IncludesInitParamsInJson()
    {
        var config = PluginConfig.Defaults()
            .InitParam("setup_key", "setup_value");

        var json = JsonDocument.Parse(config.ToJsonBytes());
        var initParams = json.RootElement.GetProperty("init_params");

        Assert.Equal("setup_value", initParams.GetProperty("setup_key").GetString());
    }

    [Fact]
    public void FluentApi___ChainingWorks()
    {
        var config = PluginConfig.Defaults()
            .WorkerThreads(8)
            .WithLogLevel(LogLevel.Warn)
            .MaxConcurrentOps(500)
            .ShutdownTimeoutMs(10000)
            .Set("key", "value")
            .InitParam("init_key", "init_value");

        var json = JsonDocument.Parse(config.ToJsonBytes());
        var root = json.RootElement;

        Assert.Equal(8, root.GetProperty("worker_threads").GetInt32());
        Assert.Equal("warn", root.GetProperty("log_level").GetString());
        Assert.Equal(500, root.GetProperty("max_concurrent_ops").GetInt32());
        Assert.Equal(10000, root.GetProperty("shutdown_timeout_ms").GetInt64());
    }
}
