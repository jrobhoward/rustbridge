package com.rustbridge;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class LogLevelTest {

    @ParameterizedTest
    @CsvSource({
            "0, TRACE",
            "1, DEBUG",
            "2, INFO",
            "3, WARN",
            "4, ERROR",
            "5, OFF"
    })
    void fromCode___valid_code___returns_correct_level(int code, LogLevel expected) {
        LogLevel level = LogLevel.fromCode(code);

        assertEquals(expected, level);
    }

    @Test
    void fromCode___invalid_code___returns_off() {
        assertEquals(LogLevel.OFF, LogLevel.fromCode(-1));
        assertEquals(LogLevel.OFF, LogLevel.fromCode(6));
        assertEquals(LogLevel.OFF, LogLevel.fromCode(100));
        assertEquals(LogLevel.OFF, LogLevel.fromCode(255));
    }

    @Test
    void getCode___returns_correct_code() {
        assertEquals(0, LogLevel.TRACE.getCode());
        assertEquals(1, LogLevel.DEBUG.getCode());
        assertEquals(2, LogLevel.INFO.getCode());
        assertEquals(3, LogLevel.WARN.getCode());
        assertEquals(4, LogLevel.ERROR.getCode());
        assertEquals(5, LogLevel.OFF.getCode());
    }

    @Test
    void roundTrip___code_to_level_and_back() {
        for (LogLevel level : LogLevel.values()) {
            int code = level.getCode();
            LogLevel recovered = LogLevel.fromCode(code);

            assertEquals(level, recovered);
        }
    }

    @Test
    void ordering___codes_increase_with_severity() {
        assertTrue(LogLevel.TRACE.getCode() < LogLevel.DEBUG.getCode());
        assertTrue(LogLevel.DEBUG.getCode() < LogLevel.INFO.getCode());
        assertTrue(LogLevel.INFO.getCode() < LogLevel.WARN.getCode());
        assertTrue(LogLevel.WARN.getCode() < LogLevel.ERROR.getCode());
        assertTrue(LogLevel.ERROR.getCode() < LogLevel.OFF.getCode());
    }
}
