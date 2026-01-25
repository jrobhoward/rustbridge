"""Native plugin implementation using ctypes."""

from __future__ import annotations

import json
from ctypes import c_void_p
from typing import Any, Callable, TypeVar

from rustbridge.core.lifecycle_state import LifecycleState
from rustbridge.core.log_level import LogLevel
from rustbridge.core.plugin_exception import PluginException
from rustbridge.core.response_envelope import ResponseEnvelope
from rustbridge.native.library import NativeLibrary
from rustbridge.native.structures import LogCallbackFnType

T = TypeVar("T")
R = TypeVar("R")

# Type alias for log callback
LogCallbackFn = Callable[[LogLevel, str, str], None]


class NativePlugin:
    """
    Native plugin implementation using ctypes.

    This implementation uses Python's ctypes to call native plugin functions directly.

    Thread Safety: This class delegates to the Rust plugin implementation which is
    thread-safe (Send + Sync), allowing concurrent execution.

    Example:
        with NativePluginLoader.load("libmyplugin.so") as plugin:
            response = plugin.call("echo", '{"message": "hello"}')
            print(response)
    """

    def __init__(
        self,
        library: NativeLibrary,
        handle: c_void_p,
        log_callback: LogCallbackFn | None = None,
        _callback_ref: LogCallbackFnType | None = None,
    ) -> None:
        """
        Create a new NativePlugin.

        This should not be called directly. Use NativePluginLoader.load() instead.

        Args:
            library: The loaded native library.
            handle: Plugin handle from plugin_init.
            log_callback: Python log callback (kept for reference).
            _callback_ref: ctypes callback reference (prevents GC).
        """
        self._library = library
        self._handle = handle
        self._log_callback = log_callback
        self._callback_ref = _callback_ref
        self._disposed = False

    @property
    def state(self) -> LifecycleState:
        """
        Get the current lifecycle state of the plugin.

        Returns:
            The current LifecycleState.

        Raises:
            PluginException: If the handle is invalid.
        """
        if self._disposed:
            return LifecycleState.STOPPED

        state_code = self._library.plugin_get_state(self._handle)
        if state_code == 255:
            raise PluginException("Invalid plugin handle")
        return LifecycleState.from_code(state_code)

    @property
    def rejected_request_count(self) -> int:
        """
        Get the number of requests rejected due to concurrency limits.

        Returns:
            Number of rejected requests.
        """
        self._throw_if_disposed()
        return self._library.plugin_get_rejected_count(self._handle)

    def call(self, type_tag: str, request: str) -> str:
        """
        Make a call to the plugin with JSON request/response.

        Args:
            type_tag: Message type identifier (e.g., "echo", "user.create").
            request: JSON request payload.

        Returns:
            JSON response payload.

        Raises:
            PluginException: If the call fails or plugin is disposed.
        """
        self._throw_if_disposed()

        request_bytes = request.encode("utf-8")
        buffer = self._library.plugin_call(self._handle, type_tag, request_bytes)

        try:
            return self._parse_result_buffer(buffer)
        finally:
            self._library.plugin_free_buffer(buffer)

    def call_typed(
        self, type_tag: str, request: Any, response_type: type[T] | None = None
    ) -> T | Any:
        """
        Make a typed call to the plugin.

        Args:
            type_tag: Message type identifier.
            request: Request object (will be JSON serialized).
            response_type: Optional response type for deserialization.

        Returns:
            Response object (deserialized from JSON).

        Raises:
            PluginException: If the call fails.
        """
        request_json = json.dumps(request)
        response_json = self.call(type_tag, request_json)
        return json.loads(response_json)

    def set_log_level(self, level: LogLevel) -> None:
        """
        Set the log level for the plugin.

        Args:
            level: The new log level.
        """
        self._throw_if_disposed()
        self._library.plugin_set_log_level(self._handle, int(level))

    def shutdown(self) -> bool:
        """
        Shutdown the plugin.

        Returns:
            True if shutdown was successful.
        """
        if self._disposed:
            return True

        self._disposed = True
        return self._library.plugin_shutdown(self._handle)

    def close(self) -> None:
        """Close the plugin (alias for context manager exit)."""
        self.shutdown()

    def _parse_result_buffer(self, buffer: Any) -> str:
        """Parse the result buffer and extract the payload."""
        if buffer.is_error():
            error_message = "Unknown error"
            if not buffer.is_empty():
                error_message = buffer.get_string()
            raise PluginException(error_message, buffer.error_code)

        if buffer.is_empty():
            return "null"

        response_json = buffer.get_string()
        envelope = ResponseEnvelope.from_json(response_json)

        if not envelope.is_success:
            raise envelope.to_exception()

        return envelope.get_payload_json()

    def _throw_if_disposed(self) -> None:
        """Raise an exception if the plugin has been disposed."""
        if self._disposed:
            raise PluginException("Plugin has been closed")

    def __enter__(self) -> "NativePlugin":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        """Context manager exit - shutdown the plugin."""
        self.shutdown()

    def __del__(self) -> None:
        """Destructor - ensure plugin is shutdown."""
        if not self._disposed:
            try:
                self.shutdown()
            except Exception:
                pass
