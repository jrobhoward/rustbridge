using System.Text.Json;
using System.Text.Json.Nodes;

namespace RustBridge;

/// <summary>
/// Configuration for plugin initialization.
/// </summary>
public class PluginConfig
{
    private readonly Dictionary<string, object?> _data = new();
    private Dictionary<string, object?>? _initParams;
    private int? _workerThreads;
    private string _logLevel = "info";
    private int _maxConcurrentOps = 1000;
    private long _shutdownTimeoutMs = 5000;

    /// <summary>
    /// Create a new empty configuration.
    /// </summary>
    public PluginConfig()
    {
    }

    /// <summary>
    /// Create a configuration with default settings.
    /// </summary>
    /// <returns>A new configuration with defaults.</returns>
    public static PluginConfig Defaults() => new();

    /// <summary>
    /// Set the number of worker threads.
    /// </summary>
    /// <param name="threads">The number of threads.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig WorkerThreads(int threads)
    {
        _workerThreads = threads;
        return this;
    }

    /// <summary>
    /// Set the log level.
    /// </summary>
    /// <param name="level">The log level.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig WithLogLevel(LogLevel level)
    {
        _logLevel = level.ToString().ToLowerInvariant();
        return this;
    }

    /// <summary>
    /// Set the log level from a string.
    /// <para>
    /// Valid values are: "trace", "debug", "info", "warn", "error", "off".
    /// Case-insensitive.
    /// </para>
    /// </summary>
    /// <param name="level">The log level as a string.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig WithLogLevel(string level)
    {
        _logLevel = level.ToLowerInvariant();
        return this;
    }

    /// <summary>
    /// Set the maximum concurrent operations.
    /// </summary>
    /// <param name="maxOps">The maximum concurrent operations.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig MaxConcurrentOps(int maxOps)
    {
        _maxConcurrentOps = maxOps;
        return this;
    }

    /// <summary>
    /// Set the shutdown timeout.
    /// </summary>
    /// <param name="timeoutMs">The timeout in milliseconds.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig ShutdownTimeout(long timeoutMs)
    {
        _shutdownTimeoutMs = timeoutMs;
        return this;
    }

    /// <summary>
    /// Set the shutdown timeout in milliseconds.
    /// Alias for <see cref="ShutdownTimeout"/> to match the field name.
    /// </summary>
    /// <param name="timeoutMs">The timeout in milliseconds.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig ShutdownTimeoutMs(long timeoutMs)
    {
        _shutdownTimeoutMs = timeoutMs;
        return this;
    }

    /// <summary>
    /// Set a custom configuration value.
    /// </summary>
    /// <param name="key">The configuration key.</param>
    /// <param name="value">The configuration value.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig Set(string key, object? value)
    {
        _data[key] = value;
        return this;
    }

    /// <summary>
    /// Set an initialization parameter.
    /// <para>
    /// Initialization parameters are passed to the plugin during startup and are
    /// intended for one-time setup configuration (migrations, seed data, etc.).
    /// These are separate from runtime configuration in <c>data</c>.
    /// </para>
    /// </summary>
    /// <param name="key">The parameter key.</param>
    /// <param name="value">The parameter value.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig InitParam(string key, object? value)
    {
        _initParams ??= new Dictionary<string, object?>();
        _initParams[key] = value;
        return this;
    }

    /// <summary>
    /// Set all initialization parameters at once.
    /// <para>
    /// Initialization parameters are passed to the plugin during startup and are
    /// intended for one-time setup configuration. This replaces any existing init params.
    /// </para>
    /// </summary>
    /// <param name="parameters">Map of initialization parameters.</param>
    /// <returns>This config for chaining.</returns>
    public PluginConfig InitParams(IDictionary<string, object?> parameters)
    {
        _initParams = new Dictionary<string, object?>(parameters);
        return this;
    }

    /// <summary>
    /// Serialize the configuration to JSON bytes.
    /// </summary>
    /// <returns>The JSON bytes.</returns>
    public byte[] ToJsonBytes()
    {
        var json = new JsonObject
        {
            ["data"] = JsonSerializer.SerializeToNode(_data),
            ["log_level"] = _logLevel,
            ["max_concurrent_ops"] = _maxConcurrentOps,
            ["shutdown_timeout_ms"] = _shutdownTimeoutMs
        };

        if (_initParams is { Count: > 0 })
        {
            json["init_params"] = JsonSerializer.SerializeToNode(_initParams);
        }

        if (_workerThreads.HasValue)
        {
            json["worker_threads"] = _workerThreads.Value;
        }

        return JsonSerializer.SerializeToUtf8Bytes(json);
    }
}
