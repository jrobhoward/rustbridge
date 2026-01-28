"""Native library wrapper using ctypes."""

from __future__ import annotations

import ctypes
from ctypes import c_bool, c_char_p, c_size_t, c_uint8, c_uint32, c_uint64, c_void_p, POINTER
from pathlib import Path

from rustbridge.core.plugin_exception import PluginException
from rustbridge.native.structures import FfiBuffer, RbResponse, LogCallbackFnType


class NativeLibrary:
    """
    Wrapper for the native plugin library.

    Loads the shared library and provides typed function wrappers for all FFI functions.
    """

    def __init__(self, library_path: str | Path) -> None:
        """
        Load a native library.

        Args:
            library_path: Path to the shared library (.so, .dylib, .dll).

        Raises:
            PluginException: If the library cannot be loaded.
        """
        self._path = str(library_path)
        try:
            self._lib = ctypes.CDLL(self._path)
        except OSError as e:
            raise PluginException(f"Failed to load library {library_path}: {e}") from e

        self._setup_functions()

    def _setup_functions(self) -> None:
        """Set up function signatures for type safety."""
        # plugin_create() -> *mut c_void
        self._lib.plugin_create.argtypes = []
        self._lib.plugin_create.restype = c_void_p

        # plugin_init(plugin_ptr, config_json, config_len, log_callback) -> handle
        self._lib.plugin_init.argtypes = [
            c_void_p,  # plugin_ptr
            POINTER(c_uint8),  # config_json
            c_size_t,  # config_len
            LogCallbackFnType,  # log_callback (can be None/null)
        ]
        self._lib.plugin_init.restype = c_void_p

        # plugin_call(handle, type_tag, request, request_len) -> FfiBuffer
        self._lib.plugin_call.argtypes = [
            c_void_p,  # handle
            c_char_p,  # type_tag (null-terminated)
            POINTER(c_uint8),  # request
            c_size_t,  # request_len
        ]
        self._lib.plugin_call.restype = FfiBuffer

        # plugin_free_buffer(buffer*)
        self._lib.plugin_free_buffer.argtypes = [POINTER(FfiBuffer)]
        self._lib.plugin_free_buffer.restype = None

        # plugin_shutdown(handle) -> bool
        self._lib.plugin_shutdown.argtypes = [c_void_p]
        self._lib.plugin_shutdown.restype = c_bool

        # plugin_set_log_level(handle, level)
        self._lib.plugin_set_log_level.argtypes = [c_void_p, c_uint8]
        self._lib.plugin_set_log_level.restype = None

        # plugin_get_state(handle) -> u8
        self._lib.plugin_get_state.argtypes = [c_void_p]
        self._lib.plugin_get_state.restype = c_uint8

        # plugin_get_rejected_count(handle) -> u64
        self._lib.plugin_get_rejected_count.argtypes = [c_void_p]
        self._lib.plugin_get_rejected_count.restype = c_uint64

        # Optional: binary transport functions
        try:
            # plugin_call_raw(handle, message_id, request, request_size) -> RbResponse
            self._lib.plugin_call_raw.argtypes = [
                c_void_p,  # handle
                c_uint32,  # message_id
                c_void_p,  # request
                c_size_t,  # request_size
            ]
            self._lib.plugin_call_raw.restype = RbResponse

            # rb_response_free(response*)
            self._lib.rb_response_free.argtypes = [POINTER(RbResponse)]
            self._lib.rb_response_free.restype = None
            self._has_binary_transport = True
        except AttributeError:
            self._has_binary_transport = False

    @property
    def path(self) -> str:
        """Return the library path."""
        return self._path

    @property
    def has_binary_transport(self) -> bool:
        """Check if this library supports binary transport."""
        return self._has_binary_transport

    def plugin_create(self) -> c_void_p:
        """Create a new plugin instance."""
        return self._lib.plugin_create()

    def plugin_init(
        self,
        plugin_ptr: c_void_p,
        config_bytes: bytes | None,
        log_callback: LogCallbackFnType | None,
    ) -> c_void_p:
        """
        Initialize a plugin with configuration.

        Args:
            plugin_ptr: Pointer from plugin_create.
            config_bytes: JSON configuration bytes (or None for defaults).
            log_callback: Optional log callback function.

        Returns:
            Handle to the initialized plugin.
        """
        if config_bytes:
            config_array = (c_uint8 * len(config_bytes))(*config_bytes)
            config_ptr = ctypes.cast(config_array, POINTER(c_uint8))
            config_len = len(config_bytes)
        else:
            config_ptr = None
            config_len = 0

        # Pass None if no callback, otherwise pass the callback
        callback = log_callback if log_callback else LogCallbackFnType(0)

        return self._lib.plugin_init(plugin_ptr, config_ptr, config_len, callback)

    def plugin_call(
        self, handle: c_void_p, type_tag: str, request: bytes
    ) -> FfiBuffer:
        """
        Make a call to the plugin.

        Args:
            handle: Plugin handle from plugin_init.
            type_tag: Message type identifier.
            request: Request payload bytes.

        Returns:
            FfiBuffer containing the response.
        """
        type_tag_bytes = type_tag.encode("utf-8")
        request_array = (c_uint8 * len(request))(*request)
        request_ptr = ctypes.cast(request_array, POINTER(c_uint8))

        return self._lib.plugin_call(
            handle, type_tag_bytes, request_ptr, len(request)
        )

    def plugin_free_buffer(self, buffer: FfiBuffer) -> None:
        """Free a buffer returned by plugin_call."""
        self._lib.plugin_free_buffer(ctypes.byref(buffer))

    def plugin_shutdown(self, handle: c_void_p) -> bool:
        """Shutdown a plugin instance."""
        return self._lib.plugin_shutdown(handle)

    def plugin_set_log_level(self, handle: c_void_p, level: int) -> None:
        """Set the log level for a plugin."""
        self._lib.plugin_set_log_level(handle, level)

    def plugin_get_state(self, handle: c_void_p) -> int:
        """Get the current state of a plugin."""
        return self._lib.plugin_get_state(handle)

    def plugin_get_rejected_count(self, handle: c_void_p) -> int:
        """Get the number of rejected requests."""
        return self._lib.plugin_get_rejected_count(handle)

    def plugin_call_raw(
        self, handle: c_void_p, message_id: int, request_ptr: c_void_p, request_size: int
    ) -> RbResponse:
        """
        Make a binary call to the plugin.

        Args:
            handle: Plugin handle from plugin_init.
            message_id: Numeric message identifier.
            request_ptr: Pointer to the request struct.
            request_size: Size of the request struct in bytes.

        Returns:
            RbResponse containing the binary response.

        Raises:
            PluginException: If binary transport is not supported.
        """
        if not self._has_binary_transport:
            raise PluginException("Binary transport not supported by this library")

        return self._lib.plugin_call_raw(handle, message_id, request_ptr, request_size)

    def rb_response_free(self, response: RbResponse) -> None:
        """Free a binary response."""
        if self._has_binary_transport:
            self._lib.rb_response_free(ctypes.byref(response))
