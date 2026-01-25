package com.rustbridge;

import org.jetbrains.annotations.NotNull;

/**
 * Plugin lifecycle states.
 * <p>
 * The lifecycle follows this state machine:
 * <pre>
 * Installed → Starting → Active → Stopping → Stopped
 *                ↑                    │
 *                └────────────────────┘ (restart)
 *            Any state → Failed (on error)
 * </pre>
 */
public enum LifecycleState {
    /**
     * Plugin is installed but not yet initialized.
     */
    INSTALLED(0),

    /**
     * Plugin is starting up.
     */
    STARTING(1),

    /**
     * Plugin is active and ready to handle requests.
     */
    ACTIVE(2),

    /**
     * Plugin is shutting down.
     */
    STOPPING(3),

    /**
     * Plugin has been stopped.
     */
    STOPPED(4),

    /**
     * Plugin has failed.
     */
    FAILED(5);

    private final int code;

    LifecycleState(int code) {
        this.code = code;
    }

    /**
     * Get the state from a numeric code.
     *
     * @param code the state code
     * @return the corresponding state
     * @throws IllegalArgumentException if the code is invalid
     */
    public static @NotNull LifecycleState fromCode(int code) {
        for (LifecycleState state : values()) {
            if (state.code == code) {
                return state;
            }
        }
        throw new IllegalArgumentException("Invalid state code: " + code);
    }

    /**
     * Get the numeric code for this state.
     *
     * @return the state code
     */
    public int getCode() {
        return code;
    }

    /**
     * Check if the plugin can handle requests in this state.
     *
     * @return true if requests can be handled
     */
    public boolean canHandleRequests() {
        return this == ACTIVE;
    }

    /**
     * Check if this is a terminal state.
     *
     * @return true if the plugin cannot transition to other states
     */
    public boolean isTerminal() {
        return this == STOPPED || this == FAILED;
    }
}
