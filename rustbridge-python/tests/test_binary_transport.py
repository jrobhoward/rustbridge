"""Tests for binary transport (call_raw).

Reference: C# BinaryTransportTest.cs, Rust hello-plugin binary_messages.rs
"""

from ctypes import Structure, c_uint8, c_uint32, sizeof
from pathlib import Path

import pytest

from rustbridge import NativePluginLoader, PluginConfig, PluginException


# Message ID for small benchmark (must match Rust MSG_BENCH_SMALL)
MSG_BENCH_SMALL = 1


# ============================================================================
# Binary Struct Types (must match Rust #[repr(C)] layouts exactly)
# ============================================================================


class SmallRequestRaw(Structure):
    """
    Small benchmark request (C struct version).

    Must match Rust SmallRequestRaw layout exactly:
    - version: u8 (1 byte)
    - _reserved: [u8; 3] (3 bytes)
    - key: [u8; 64] (64 bytes)
    - key_len: u32 (4 bytes)
    - flags: u32 (4 bytes)
    Total: 76 bytes
    """

    _pack_ = 1
    _fields_ = [
        ("version", c_uint8),
        ("_reserved", c_uint8 * 3),
        ("key", c_uint8 * 64),
        ("key_len", c_uint32),
        ("flags", c_uint32),
    ]

    CURRENT_VERSION = 1

    @classmethod
    def create(cls, key: str, flags: int) -> "SmallRequestRaw":
        """Create a new request with the given key and flags."""
        request = cls()
        request.version = cls.CURRENT_VERSION

        # Copy key bytes into the fixed-size buffer
        key_bytes = key.encode("utf-8")
        key_len = min(len(key_bytes), 64)
        for i in range(key_len):
            request.key[i] = key_bytes[i]
        request.key_len = key_len
        request.flags = flags

        return request

    def get_key(self) -> str:
        """Get the key as a string."""
        return bytes(self.key[: self.key_len]).decode("utf-8")


class SmallResponseRaw(Structure):
    """
    Small benchmark response (C struct version).

    Must match Rust SmallResponseRaw layout exactly:
    - version: u8 (1 byte)
    - _reserved: [u8; 3] (3 bytes)
    - value: [u8; 64] (64 bytes)
    - value_len: u32 (4 bytes)
    - ttl_seconds: u32 (4 bytes)
    - cache_hit: u8 (1 byte)
    - _padding: [u8; 3] (3 bytes)
    Total: 80 bytes
    """

    _pack_ = 1
    _fields_ = [
        ("version", c_uint8),
        ("_reserved", c_uint8 * 3),
        ("value", c_uint8 * 64),
        ("value_len", c_uint32),
        ("ttl_seconds", c_uint32),
        ("cache_hit", c_uint8),
        ("_padding", c_uint8 * 3),
    ]

    CURRENT_VERSION = 1

    def get_value(self) -> str:
        """Get the value as a string."""
        return bytes(self.value[: self.value_len]).decode("utf-8")


# ============================================================================
# Tests
# ============================================================================


class TestBinaryTransport:
    """Tests for binary transport (call_raw)."""

    def test_struct_sizes___match_rust_layout(self) -> None:
        """Verify struct sizes match Rust #[repr(C)] layouts."""
        assert sizeof(SmallRequestRaw) == 76
        assert sizeof(SmallResponseRaw) == 80

    def test_call_raw___small_benchmark___returns_valid_response(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Test basic binary transport call."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            request = SmallRequestRaw.create("test_key", 0x01)

            response = plugin.call_raw(MSG_BENCH_SMALL, request, SmallResponseRaw)

            assert response.version == SmallResponseRaw.CURRENT_VERSION
            assert response.value_len > 0
            assert response.ttl_seconds == 3600
            assert response.cache_hit == 1  # flags & 1 != 0

    def test_call_raw___with_cache_miss___returns_cache_miss(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Test binary transport with cache miss flag."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            request = SmallRequestRaw.create("another_key", 0x00)  # flags = 0, cache miss

            response = plugin.call_raw(MSG_BENCH_SMALL, request, SmallResponseRaw)

            assert response.cache_hit == 0  # flags & 1 == 0

    def test_call_raw___response_contains_key(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Test that response value contains the key."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            request = SmallRequestRaw.create("my_special_key", 0x01)

            response = plugin.call_raw(MSG_BENCH_SMALL, request, SmallResponseRaw)

            value = response.get_value()
            assert "my_special_key" in value

    def test_call_raw___concurrent_calls___all_succeed(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Test concurrent binary transport calls."""
        import concurrent.futures

        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            concurrent_calls = 100

            def make_call(i: int) -> SmallResponseRaw:
                request = SmallRequestRaw.create(f"key_{i}", i % 2)
                return plugin.call_raw(MSG_BENCH_SMALL, request, SmallResponseRaw)

            with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
                futures = [executor.submit(make_call, i) for i in range(concurrent_calls)]
                results = [f.result() for f in concurrent.futures.as_completed(futures)]

            # Verify all succeeded
            assert len(results) == concurrent_calls
            for response in results:
                assert response.version == SmallResponseRaw.CURRENT_VERSION

    def test_call_raw___unknown_message_id___raises_exception(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Test that unknown message ID raises exception."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            request = SmallRequestRaw.create("test", 0)

            with pytest.raises(PluginException, match="Unknown message ID"):
                plugin.call_raw(999, request, SmallResponseRaw)

    def test_has_binary_transport___returns_true(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Test that has_binary_transport property works."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            assert plugin.has_binary_transport is True


class TestBinaryTransportBenchmark:
    """Benchmark tests comparing JSON vs binary transport."""

    def test_binary_vs_json___small_payload(
        self,
        benchmark,
        hello_plugin_path: Path,
        skip_if_no_plugin: None,
    ) -> None:
        """Benchmark binary transport for small payloads."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            request = SmallRequestRaw.create("benchmark_key", 0x01)

            def call_raw():
                return plugin.call_raw(MSG_BENCH_SMALL, request, SmallResponseRaw)

            result = benchmark(call_raw)
            assert result.version == SmallResponseRaw.CURRENT_VERSION
