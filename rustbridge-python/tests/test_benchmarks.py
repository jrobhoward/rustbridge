"""Performance benchmarks for rustbridge Python bindings.

These benchmarks measure:
- Call latency (round-trip time for plugin calls)
- Call throughput (calls per second)
- Concurrent call performance
- Payload size impact

Run with: pytest tests/test_benchmarks.py -v --benchmark-only
"""

import json
from pathlib import Path

import pytest

from rustbridge import NativePluginLoader, PluginConfig


# ============================================================================
# Test Fixtures
# ============================================================================


@pytest.fixture(scope="module")
def plugin(hello_plugin_path: Path | None):
    """Create a plugin that stays open for all benchmarks in this module."""
    if hello_plugin_path is None:
        pytest.skip("hello-plugin not built. Run: cargo build -p hello-plugin --release")

    config = PluginConfig.defaults()
    plugin = NativePluginLoader.load_with_config(str(hello_plugin_path), config)
    yield plugin
    plugin.shutdown()


# ============================================================================
# Small Payload Benchmarks (~100 bytes)
# ============================================================================


class TestSmallPayloadBenchmarks:
    """Benchmarks for small payload sizes (~100 bytes)."""

    @pytest.fixture
    def small_request(self) -> str:
        """Small echo request (~50 bytes)."""
        return json.dumps({"message": "hello world from benchmark"})

    def test_call_latency___small_echo(
        self,
        benchmark,
        plugin,
        hello_plugin_path: Path | None,
        small_request: str,
    ) -> None:
        """Measure call latency for small echo requests."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        def call_echo():
            return plugin.call("echo", small_request)

        result = benchmark(call_echo)
        assert "hello world from benchmark" in result

    def test_call_latency___small_greet(
        self,
        benchmark,
        plugin,
        hello_plugin_path: Path | None,
    ) -> None:
        """Measure call latency for small greet requests."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        request = json.dumps({"name": "BenchmarkUser"})

        def call_greet():
            return plugin.call("greet", request)

        result = benchmark(call_greet)
        assert "BenchmarkUser" in result

    def test_call_latency___small_math(
        self,
        benchmark,
        plugin,
        hello_plugin_path: Path | None,
    ) -> None:
        """Measure call latency for small math requests."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        request = json.dumps({"a": 123456789, "b": 987654321})

        def call_math():
            return plugin.call("math.add", request)

        result = benchmark(call_math)
        response = json.loads(result)
        assert response["result"] == 1111111110


# ============================================================================
# Medium Payload Benchmarks (~1KB)
# ============================================================================


class TestMediumPayloadBenchmarks:
    """Benchmarks for medium payload sizes (~1KB)."""

    @pytest.fixture
    def medium_request(self) -> str:
        """Medium request (~1KB)."""
        # Create a message with enough content to be ~1KB
        message = "x" * 900  # ~900 bytes of content
        return json.dumps({"message": message})

    def test_call_latency___medium_echo(
        self,
        benchmark,
        plugin,
        hello_plugin_path: Path | None,
        medium_request: str,
    ) -> None:
        """Measure call latency for medium echo requests."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        def call_echo():
            return plugin.call("echo", medium_request)

        result = benchmark(call_echo)
        assert len(result) > 900  # Response should contain the echoed message


# ============================================================================
# Large Payload Benchmarks (~100KB)
# ============================================================================


class TestLargePayloadBenchmarks:
    """Benchmarks for large payload sizes (~100KB)."""

    @pytest.fixture
    def large_request(self) -> str:
        """Large request (~100KB)."""
        message = "x" * 100000  # 100KB of content
        return json.dumps({"message": message})

    def test_call_latency___large_echo(
        self,
        benchmark,
        plugin,
        hello_plugin_path: Path | None,
        large_request: str,
    ) -> None:
        """Measure call latency for large echo requests."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        def call_echo():
            return plugin.call("echo", large_request)

        result = benchmark(call_echo)
        assert len(result) > 100000  # Response should contain the echoed message


# ============================================================================
# Throughput Benchmarks
# ============================================================================


class TestThroughputBenchmarks:
    """Benchmarks measuring calls per second."""

    def test_throughput___sequential_calls(
        self,
        benchmark,
        plugin,
        hello_plugin_path: Path | None,
    ) -> None:
        """Measure throughput for sequential calls (1000 iterations)."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        request = json.dumps({"message": "throughput test"})

        def call_batch():
            for _ in range(100):
                plugin.call("echo", request)

        benchmark.pedantic(call_batch, iterations=1, rounds=10)

    def test_throughput___concurrent_calls(
        self,
        benchmark,
        plugin,
        hello_plugin_path: Path | None,
    ) -> None:
        """Measure throughput for concurrent calls using threads."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        import concurrent.futures

        request = json.dumps({"message": "concurrent throughput test"})

        def make_call():
            return plugin.call("echo", request)

        def run_concurrent():
            with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
                futures = [executor.submit(make_call) for _ in range(100)]
                for f in concurrent.futures.as_completed(futures):
                    f.result()

        benchmark.pedantic(run_concurrent, iterations=1, rounds=5)


# ============================================================================
# Plugin Lifecycle Benchmarks
# ============================================================================


class TestLifecycleBenchmarks:
    """Benchmarks for plugin lifecycle operations."""

    def test_lifecycle___load_unload_cycle(
        self,
        benchmark,
        hello_plugin_path: Path | None,
    ) -> None:
        """Measure time to load and unload a plugin."""
        if hello_plugin_path is None:
            pytest.skip("hello-plugin not built")

        def load_unload():
            with NativePluginLoader.load(str(hello_plugin_path)) as p:
                # Make one call to ensure plugin is fully initialized
                p.call("echo", '{"message": "init"}')

        benchmark.pedantic(load_unload, iterations=1, rounds=10)
