"""Plugin exception with error code support."""


class PluginException(Exception):
    """
    Exception raised by plugin operations.

    Attributes:
        error_code: Numeric error code from FFI (0 = success).
        message: Human-readable error message.
    """

    def __init__(self, message: str, error_code: int = 1) -> None:
        """
        Create a new PluginException.

        Args:
            message: Human-readable error message.
            error_code: Numeric error code (default: 1).
        """
        super().__init__(message)
        self.error_code = error_code
        self.message = message

    def __str__(self) -> str:
        return f"[{self.error_code}] {self.message}"

    def __repr__(self) -> str:
        return f"PluginException(message={self.message!r}, error_code={self.error_code})"
