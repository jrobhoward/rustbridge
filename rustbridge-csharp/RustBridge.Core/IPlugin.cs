namespace RustBridge;

/// <summary>
/// Base interface for RustBridge plugins.
/// <para>
/// Plugins are loaded from native shared libraries and provide a JSON-based
/// request/response API. This interface provides the core operations for
/// interacting with the plugin.
/// </para>
/// </summary>
public interface IPlugin : IDisposable
{
    /// <summary>
    /// Get the current lifecycle state of the plugin.
    /// </summary>
    LifecycleState State { get; }

    /// <summary>
    /// Make a synchronous call to the plugin.
    /// </summary>
    /// <param name="typeTag">The message type tag (e.g., "user.create").</param>
    /// <param name="request">The JSON request payload.</param>
    /// <returns>The JSON response payload.</returns>
    /// <exception cref="PluginException">If the call fails.</exception>
    string Call(string typeTag, string request);

    /// <summary>
    /// Make a synchronous call with a typed request and response.
    /// </summary>
    /// <typeparam name="TRequest">The request type.</typeparam>
    /// <typeparam name="TResponse">The response type.</typeparam>
    /// <param name="typeTag">The message type tag.</param>
    /// <param name="request">The request object (will be serialized to JSON).</param>
    /// <returns>The deserialized response.</returns>
    /// <exception cref="PluginException">If the call fails.</exception>
    TResponse Call<TRequest, TResponse>(string typeTag, TRequest request);

    /// <summary>
    /// Set the log level for the plugin.
    /// </summary>
    /// <param name="level">The new log level.</param>
    void SetLogLevel(LogLevel level);

    /// <summary>
    /// Get the count of requests rejected due to concurrency limits.
    /// <para>
    /// When the plugin's <c>MaxConcurrentOps</c> limit is exceeded, requests
    /// are immediately rejected with a TooManyRequests error. This counter tracks
    /// the total number of rejected requests since plugin initialization.
    /// </para>
    /// </summary>
    long RejectedRequestCount { get; }

    /// <summary>
    /// Call the plugin with a binary struct request (raw binary transport).
    /// <para>
    /// This method bypasses JSON serialization for high-performance scenarios.
    /// The request and response are fixed-size C structs.
    /// </para>
    /// </summary>
    /// <typeparam name="TRequest">The request struct type.</typeparam>
    /// <typeparam name="TResponse">The response struct type.</typeparam>
    /// <param name="messageId">The binary message ID (registered with register_binary_handler).</param>
    /// <param name="request">The request struct.</param>
    /// <returns>The response struct.</returns>
    /// <exception cref="PluginException">If the call fails.</exception>
    TResponse CallRaw<TRequest, TResponse>(int messageId, TRequest request)
        where TRequest : unmanaged, IBinaryStruct
        where TResponse : unmanaged, IBinaryStruct;
}
