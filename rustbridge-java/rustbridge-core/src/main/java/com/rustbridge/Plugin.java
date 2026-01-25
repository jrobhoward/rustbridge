package com.rustbridge;

import org.jetbrains.annotations.NotNull;

/**
 * Base interface for rustbridge plugins.
 * <p>
 * Plugins are loaded from native shared libraries and provide a JSON-based
 * request/response API. This interface provides the core operations for
 * interacting with the plugin.
 */
public interface Plugin extends AutoCloseable {

    /**
     * Get the current lifecycle state of the plugin.
     *
     * @return the current state
     */
    @NotNull
    LifecycleState getState();

    /**
     * Make a synchronous call to the plugin.
     *
     * @param typeTag the message type tag (e.g., "user.create")
     * @param request the JSON request payload
     * @return the JSON response payload
     * @throws PluginException if the call fails
     */
    @NotNull
    String call(@NotNull String typeTag, @NotNull String request) throws PluginException;

    /**
     * Make a synchronous call with a typed request and response.
     *
     * @param typeTag      the message type tag
     * @param request      the request object (will be serialized to JSON)
     * @param responseType the response class
     * @param <T>          the request type
     * @param <R>          the response type
     * @return the deserialized response
     * @throws PluginException if the call fails
     */
    @NotNull
    <T, R> R call(@NotNull String typeTag, @NotNull T request, @NotNull Class<R> responseType) throws PluginException;

    /**
     * Set the log level for the plugin.
     *
     * @param level the new log level
     */
    void setLogLevel(@NotNull LogLevel level);

    /**
     * Get the count of requests rejected due to concurrency limits.
     * <p>
     * When the plugin's {@code maxConcurrentOps} limit is exceeded, requests
     * are immediately rejected with a TooManyRequests error. This counter tracks
     * the total number of rejected requests since plugin initialization.
     *
     * @return number of rejected requests since plugin initialization
     */
    long getRejectedRequestCount();

    /**
     * Shutdown the plugin and release resources.
     * <p>
     * This is called automatically when using try-with-resources.
     */
    @Override
    void close();
}
