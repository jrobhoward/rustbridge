package com.rustbridge;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.HashMap;
import java.util.Map;

/**
 * Configuration for plugin initialization.
 */
public class PluginConfig {
    private static final ObjectMapper OBJECT_MAPPER = JsonMapper.getInstance();

    private final Map<String, Object> data;
    private @Nullable Map<String, Object> initParams;
    private @Nullable Integer workerThreads;
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
    public static @NotNull PluginConfig defaults() {
        return new PluginConfig();
    }

    /**
     * Set the number of worker threads.
     *
     * @param threads the number of threads
     * @return this config for chaining
     */
    public @NotNull PluginConfig workerThreads(int threads) {
        this.workerThreads = threads;
        return this;
    }

    /**
     * Set the log level.
     *
     * @param level the log level
     * @return this config for chaining
     */
    public @NotNull PluginConfig logLevel(@NotNull LogLevel level) {
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
    public @NotNull PluginConfig logLevel(@NotNull String level) {
        this.logLevel = level.toLowerCase();
        return this;
    }

    /**
     * Set the maximum concurrent operations.
     *
     * @param maxOps the maximum concurrent operations
     * @return this config for chaining
     */
    public @NotNull PluginConfig maxConcurrentOps(int maxOps) {
        this.maxConcurrentOps = maxOps;
        return this;
    }

    /**
     * Set the shutdown timeout.
     *
     * @param timeoutMs the timeout in milliseconds
     * @return this config for chaining
     */
    public @NotNull PluginConfig shutdownTimeout(long timeoutMs) {
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
    public @NotNull PluginConfig shutdownTimeoutMs(long timeoutMs) {
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
    public @NotNull PluginConfig set(@NotNull String key, @Nullable Object value) {
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
    public @NotNull PluginConfig initParam(@NotNull String key, @Nullable Object value) {
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
    public @NotNull PluginConfig initParams(@NotNull Map<String, Object> params) {
        this.initParams = new HashMap<>(params);
        return this;
    }

    /**
     * Serialize the configuration to JSON bytes.
     *
     * @return the JSON bytes
     */
    public byte @NotNull [] toJsonBytes() {
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
