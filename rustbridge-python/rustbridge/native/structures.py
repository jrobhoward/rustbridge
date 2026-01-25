"""ctypes structures for FFI interop."""

from ctypes import Structure, c_uint8, c_uint32, c_uint64, c_size_t, POINTER


class FfiBuffer(Structure):
    """
    Buffer for passing data across FFI boundary.

    This structure follows the "Rust allocates, host frees" pattern:
    - Rust creates the buffer and populates it with data
    - Host copies the data to its managed heap
    - Host calls `plugin_free_buffer` to release the memory

    Attributes:
        data: Pointer to the data bytes.
        len: Length of valid data in bytes.
        capacity: Total capacity of the allocation.
        error_code: Error code (0 = success).
    """

    _fields_ = [
        ("data", POINTER(c_uint8)),
        ("len", c_size_t),
        ("capacity", c_size_t),
        ("error_code", c_uint32),
    ]

    def is_error(self) -> bool:
        """Check if this buffer represents an error."""
        return self.error_code != 0

    def is_empty(self) -> bool:
        """Check if this buffer is empty."""
        return not self.data or self.len == 0

    def get_bytes(self) -> bytes:
        """
        Get the buffer data as bytes.

        Returns:
            The buffer contents as bytes, or empty bytes if buffer is empty.
        """
        if self.is_empty():
            return b""
        # Create bytes from the pointer and length
        return bytes(self.data[: self.len])

    def get_string(self, encoding: str = "utf-8") -> str:
        """
        Get the buffer data as a string.

        Args:
            encoding: The string encoding to use.

        Returns:
            The buffer contents as a string.
        """
        return self.get_bytes().decode(encoding)


class RbResponse(Structure):
    """
    Response structure for binary transport.

    Used by plugin_call_raw for high-performance binary communication.

    Attributes:
        error_code: Error code (0 = success).
        len: Length of valid data.
        capacity: Total capacity of the allocation.
        data: Pointer to the data.
    """

    _fields_ = [
        ("error_code", c_uint32),
        ("len", c_uint32),
        ("capacity", c_uint32),
        ("data", POINTER(c_uint8)),
    ]

    def is_error(self) -> bool:
        """Check if this response represents an error."""
        return self.error_code != 0

    def get_bytes(self) -> bytes:
        """Get the response data as bytes."""
        if not self.data or self.len == 0:
            return b""
        return bytes(self.data[: self.len])


# Log callback function type
# void (*)(uint8_t level, const char* target, const char* message, size_t message_len)
from ctypes import CFUNCTYPE, c_char_p

LogCallbackFnType = CFUNCTYPE(None, c_uint8, c_char_p, c_char_p, c_size_t)
