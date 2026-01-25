package com.rustbridge;

import org.jetbrains.annotations.NotNull;

/**
 * Log levels for plugin logging.
 */
public enum LogLevel {
    /**
     * Trace level - very detailed debugging information.
     */
    TRACE(0),

    /**
     * Debug level - debugging information.
     */
    DEBUG(1),

    /**
     * Info level - general information.
     */
    INFO(2),

    /**
     * Warn level - warning messages.
     */
    WARN(3),

    /**
     * Error level - error messages.
     */
    ERROR(4),

    /**
     * Off - disable logging.
     */
    OFF(5);

    private final int code;

    LogLevel(int code) {
        this.code = code;
    }

    /**
     * Get the level from a numeric code.
     *
     * @param code the level code
     * @return the corresponding level
     */
    public static @NotNull LogLevel fromCode(int code) {
        for (LogLevel level : values()) {
            if (level.code == code) {
                return level;
            }
        }
        return OFF;
    }

    /**
     * Get the numeric code for this level.
     *
     * @return the level code
     */
    public int getCode() {
        return code;
    }
}
