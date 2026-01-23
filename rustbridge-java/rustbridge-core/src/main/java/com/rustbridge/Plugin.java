package com.rustbridge;

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
    LifecycleState getState();

    /**
     * Make a synchronous call to the plugin.
     *
     * @param typeTag the message type tag (e.g., "user.create")
     * @param request the JSON request payload
     * @return the JSON response payload
     * @throws PluginException if the call fails
     */
    String call(String typeTag, String request) throws PluginException;

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
    <T, R> R call(String typeTag, T request, Class<R> responseType) throws PluginException;

    /**
     * Set the log level for the plugin.
     *
     * @param level the new log level
     */
    void setLogLevel(LogLevel level);

    /**
     * Shutdown the plugin and release resources.
     * <p>
     * This is called automatically when using try-with-resources.
     */
    @Override
    void close();
}
