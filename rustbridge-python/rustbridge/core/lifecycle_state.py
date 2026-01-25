"""Lifecycle state enumeration for rustbridge plugins."""

from enum import IntEnum


class LifecycleState(IntEnum):
    """
    Plugin lifecycle state.

    Values correspond to the FFI codes used by rustbridge:
    - INSTALLED = 0: Plugin created but not started
    - STARTING = 1: Plugin is starting up
    - ACTIVE = 2: Plugin is ready to handle requests
    - STOPPING = 3: Plugin is shutting down
    - STOPPED = 4: Plugin has stopped
    - FAILED = 5: Plugin failed to start or crashed
    """

    INSTALLED = 0
    STARTING = 1
    ACTIVE = 2
    STOPPING = 3
    STOPPED = 4
    FAILED = 5

    @classmethod
    def from_code(cls, code: int) -> "LifecycleState":
        """
        Create a LifecycleState from its numeric code.

        Args:
            code: The numeric code (0-5).

        Returns:
            The corresponding LifecycleState.

        Raises:
            ValueError: If code is not in range 0-5.
        """
        if 0 <= code <= 5:
            return cls(code)
        raise ValueError(f"Invalid lifecycle state code: {code}")

    def can_handle_requests(self) -> bool:
        """Check if this state can handle requests."""
        return self == LifecycleState.ACTIVE

    def is_terminal(self) -> bool:
        """Check if this is a terminal state (stopped or failed)."""
        return self in (LifecycleState.STOPPED, LifecycleState.FAILED)
