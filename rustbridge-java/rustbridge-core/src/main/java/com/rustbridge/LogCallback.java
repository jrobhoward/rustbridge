package com.rustbridge;

import org.jetbrains.annotations.NotNull;

/**
 * Callback interface for receiving log messages from plugins.
 */
@FunctionalInterface
public interface LogCallback {
    /**
     * Called when the plugin emits a log message.
     *
     * @param level   the log level
     * @param target  the log target (usually module path)
     * @param message the log message
     */
    void log(@NotNull LogLevel level, @NotNull String target, @NotNull String message);
}
