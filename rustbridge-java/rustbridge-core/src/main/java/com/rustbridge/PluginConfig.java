package com.rustbridge;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.util.HashMap;
import java.util.Map;

/**
 * Configuration for plugin initialization.
 */
public class PluginConfig {
    private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

    private final Map<String, Object> data;
    private Map<String, Object> initParams;
    private Integer workerThreads;
    private String logLevel = "info";
    private int maxConcurrentOps = 1000;
    private long shutdownTimeoutMs = 5000;

    /**
     * Create a new empty configuration.
     */
    public PluginConfig() {
        this.data = new HashMap<>();
    }

    /**
     * Create a configuration with default settings.
     *
     * @return a new configuration with defaults
     */
    public static PluginConfig defaults() {
        return new PluginConfig();
    }

    /**
     * Set the number of worker threads.
     *
     * @param threads the number of threads
     * @return this config for chaining
     */
    public PluginConfig workerThreads(int threads) {
        this.workerThreads = threads;
        return this;
    }

    /**
     * Set the log level.
     *
     * @param level the log level
     * @return this config for chaining
     */
    public PluginConfig logLevel(LogLevel level) {
        this.logLevel = level.name().toLowerCase();
        return this;
    }

    /**
     * Set the log level from a string.
     * <p>
     * Valid values are: "trace", "debug", "info", "warn", "error", "off".
     * Case-insensitive.
     *
     * @param level the log level as a string
     * @return this config for chaining
     */
    public PluginConfig logLevel(String level) {
        this.logLevel = level.toLowerCase();
        return this;
    }

    /**
     * Set the maximum concurrent operations.
     *
     * @param maxOps the maximum concurrent operations
     * @return this config for chaining
     */
    public PluginConfig maxConcurrentOps(int maxOps) {
        this.maxConcurrentOps = maxOps;
        return this;
    }

    /**
     * Set the shutdown timeout.
     *
     * @param timeoutMs the timeout in milliseconds
     * @return this config for chaining
     */
    public PluginConfig shutdownTimeout(long timeoutMs) {
        this.shutdownTimeoutMs = timeoutMs;
        return this;
    }

    /**
     * Set the shutdown timeout in milliseconds.
     * <p>
     * Alias for {@link #shutdownTimeout(long)} to match the field name.
     *
     * @param timeoutMs the timeout in milliseconds
     * @return this config for chaining
     */
    public PluginConfig shutdownTimeoutMs(long timeoutMs) {
        this.shutdownTimeoutMs = timeoutMs;
        return this;
    }

    /**
     * Set a custom configuration value.
     *
     * @param key   the configuration key
     * @param value the configuration value
     * @return this config for chaining
     */
    public PluginConfig set(String key, Object value) {
        this.data.put(key, value);
        return this;
    }

    /**
     * Set an initialization parameter.
     * <p>
     * Initialization parameters are passed to the plugin during startup and are
     * intended for one-time setup configuration (migrations, seed data, etc.).
     * These are separate from runtime configuration in {@code data}.
     *
     * @param key   the parameter key
     * @param value the parameter value
     * @return this config for chaining
     */
    public PluginConfig initParam(String key, Object value) {
        if (this.initParams == null) {
            this.initParams = new HashMap<>();
        }
        this.initParams.put(key, value);
        return this;
    }

    /**
     * Set all initialization parameters at once.
     * <p>
     * Initialization parameters are passed to the plugin during startup and are
     * intended for one-time setup configuration. This replaces any existing init params.
     *
     * @param params map of initialization parameters
     * @return this config for chaining
     */
    public PluginConfig initParams(Map<String, Object> params) {
        this.initParams = new HashMap<>(params);
        return this;
    }

    /**
     * Serialize the configuration to JSON bytes.
     *
     * @return the JSON bytes
     */
    public byte[] toJsonBytes() {
        ObjectNode json = OBJECT_MAPPER.createObjectNode();
        json.set("data", OBJECT_MAPPER.valueToTree(data));

        if (initParams != null && !initParams.isEmpty()) {
            json.set("init_params", OBJECT_MAPPER.valueToTree(initParams));
        }

        if (workerThreads != null) {
            json.put("worker_threads", workerThreads);
        }

        json.put("log_level", logLevel);
        json.put("max_concurrent_ops", maxConcurrentOps);
        json.put("shutdown_timeout_ms", shutdownTimeoutMs);

        try {
            return OBJECT_MAPPER.writeValueAsBytes(json);
        } catch (JsonProcessingException e) {
            throw new RuntimeException("Failed to serialize config", e);
        }
    }
}
