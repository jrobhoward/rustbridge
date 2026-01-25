"""Log level enumeration for rustbridge plugins."""

from enum import IntEnum


class LogLevel(IntEnum):
    """
    Log level for plugin logging.

    Values correspond to the FFI codes used by rustbridge:
    - TRACE = 0: Most detailed logging
    - DEBUG = 1: Debug-level messages
    - INFO = 2: Informational messages
    - WARN = 3: Warning messages
    - ERROR = 4: Error messages
    - OFF = 5: Logging disabled
    """

    TRACE = 0
    DEBUG = 1
    INFO = 2
    WARN = 3
    ERROR = 4
    OFF = 5

    @classmethod
    def from_code(cls, code: int) -> "LogLevel":
        """
        Create a LogLevel from its numeric code.

        Args:
            code: The numeric code (0-5).

        Returns:
            The corresponding LogLevel.

        Raises:
            ValueError: If code is not in range 0-5.
        """
        if 0 <= code <= 5:
            return cls(code)
        raise ValueError(f"Invalid log level code: {code}")

    @classmethod
    def from_string(cls, level: str) -> "LogLevel":
        """
        Create a LogLevel from a string name.

        Args:
            level: The level name (case-insensitive).

        Returns:
            The corresponding LogLevel.

        Raises:
            ValueError: If level string is not recognized.
        """
        level_upper = level.upper()
        try:
            return cls[level_upper]
        except KeyError:
            valid = ", ".join(l.name.lower() for l in cls)
            raise ValueError(f"Invalid log level: {level}. Valid values: {valid}") from None

    def to_string(self) -> str:
        """Return the lowercase string representation."""
        return self.name.lower()
