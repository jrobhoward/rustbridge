package com.rustbridge;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

/**
 * Exception thrown by rustbridge plugin operations.
 */
public class PluginException extends Exception {
    private static final long serialVersionUID = 1L;

    private final int errorCode;

    /**
     * Create a new PluginException.
     *
     * @param message the error message
     */
    public PluginException(@NotNull String message) {
        super(message);
        this.errorCode = 0;
    }

    /**
     * Create a new PluginException with an error code.
     *
     * @param errorCode the error code
     * @param message   the error message
     */
    public PluginException(int errorCode, @NotNull String message) {
        super(message);
        this.errorCode = errorCode;
    }

    /**
     * Create a new PluginException with a cause.
     *
     * @param message the error message
     * @param cause   the underlying cause
     */
    public PluginException(@NotNull String message, @Nullable Throwable cause) {
        super(message, cause);
        this.errorCode = 0;
    }

    /**
     * Get the error code.
     *
     * @return the error code, or 0 if not set
     */
    public int getErrorCode() {
        return errorCode;
    }
}
