"""Tests for LogLevel enum."""

import pytest

from rustbridge import LogLevel


class TestLogLevel:
    """Tests for LogLevel enum."""

    def test_from_code___valid_codes___returns_correct_level(self) -> None:
        assert LogLevel.from_code(0) == LogLevel.TRACE
        assert LogLevel.from_code(1) == LogLevel.DEBUG
        assert LogLevel.from_code(2) == LogLevel.INFO
        assert LogLevel.from_code(3) == LogLevel.WARN
        assert LogLevel.from_code(4) == LogLevel.ERROR
        assert LogLevel.from_code(5) == LogLevel.OFF

    def test_from_code___invalid_code___raises_value_error(self) -> None:
        with pytest.raises(ValueError, match="Invalid log level code"):
            LogLevel.from_code(6)

        with pytest.raises(ValueError, match="Invalid log level code"):
            LogLevel.from_code(-1)

    def test_from_string___valid_strings___returns_correct_level(self) -> None:
        assert LogLevel.from_string("trace") == LogLevel.TRACE
        assert LogLevel.from_string("DEBUG") == LogLevel.DEBUG
        assert LogLevel.from_string("Info") == LogLevel.INFO
        assert LogLevel.from_string("WARN") == LogLevel.WARN
        assert LogLevel.from_string("error") == LogLevel.ERROR
        assert LogLevel.from_string("OFF") == LogLevel.OFF

    def test_from_string___invalid_string___raises_value_error(self) -> None:
        with pytest.raises(ValueError, match="Invalid log level"):
            LogLevel.from_string("invalid")

    def test_to_string___all_levels___returns_lowercase(self) -> None:
        assert LogLevel.TRACE.to_string() == "trace"
        assert LogLevel.DEBUG.to_string() == "debug"
        assert LogLevel.INFO.to_string() == "info"
        assert LogLevel.WARN.to_string() == "warn"
        assert LogLevel.ERROR.to_string() == "error"
        assert LogLevel.OFF.to_string() == "off"

    def test_int_value___matches_ffi_codes(self) -> None:
        assert int(LogLevel.TRACE) == 0
        assert int(LogLevel.DEBUG) == 1
        assert int(LogLevel.INFO) == 2
        assert int(LogLevel.WARN) == 3
        assert int(LogLevel.ERROR) == 4
        assert int(LogLevel.OFF) == 5
