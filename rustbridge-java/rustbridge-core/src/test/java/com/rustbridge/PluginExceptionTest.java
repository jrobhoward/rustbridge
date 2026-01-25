package com.rustbridge;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PluginExceptionTest {

    @Test
    void constructor___message_only___sets_message_and_zero_code() {
        PluginException exception = new PluginException("Test error");

        assertEquals("Test error", exception.getMessage());
        assertEquals(0, exception.getErrorCode());
        assertNull(exception.getCause());
    }

    @Test
    void constructor___code_and_message___sets_both() {
        PluginException exception = new PluginException(5, "Serialization error");

        assertEquals("Serialization error", exception.getMessage());
        assertEquals(5, exception.getErrorCode());
        assertNull(exception.getCause());
    }

    @Test
    void constructor___message_and_cause___sets_both() {
        RuntimeException cause = new RuntimeException("Root cause");
        PluginException exception = new PluginException("Wrapper error", cause);

        assertEquals("Wrapper error", exception.getMessage());
        assertEquals(0, exception.getErrorCode());
        assertEquals(cause, exception.getCause());
    }

    @Test
    void getErrorCode___various_codes___returns_correct_value() {
        assertEquals(0, new PluginException(0, "test").getErrorCode());
        assertEquals(1, new PluginException(1, "test").getErrorCode());
        assertEquals(5, new PluginException(5, "test").getErrorCode());
        assertEquals(6, new PluginException(6, "test").getErrorCode());
        assertEquals(11, new PluginException(11, "test").getErrorCode());
    }

    @Test
    void exception___is_checked_exception() {
        PluginException exception = new PluginException("test");

        assertTrue(exception instanceof Exception);
        // PluginException is a checked exception (extends Exception, not RuntimeException)
        assertFalse(RuntimeException.class.isAssignableFrom(PluginException.class));
    }
}
