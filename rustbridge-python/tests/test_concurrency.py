"""Tests for concurrency limiting and backpressure.

Reference: Java ConcurrencyLimitTest.java
"""

import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import pytest

from rustbridge import NativePluginLoader, PluginConfig, PluginException


class TestConcurrencyLimit:
    """Tests for concurrency limiting and backpressure."""

    def test_concurrency_limit___exceeded___returns_error(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Concurrency limit exceeded returns error."""
        config = PluginConfig.defaults().max_concurrent_ops(2)

        with NativePluginLoader.load_with_config(str(hello_plugin_path), config) as plugin:
            success_count = 0
            error_count = 0
            lock = threading.Lock()

            def make_call(call_id: int) -> bool:
                """Make a call that holds the permit for 300ms."""
                nonlocal success_count, error_count
                try:
                    # Use sleep handler to hold permits longer (300ms)
                    plugin.call("test.sleep", '{"duration_ms": 300}')
                    with lock:
                        success_count += 1
                    return True
                except PluginException:
                    with lock:
                        error_count += 1
                    return False

            # Submit 15 requests, staggered to ensure we hit the limit
            with ThreadPoolExecutor(max_workers=15) as executor:
                futures = []
                for i in range(15):
                    future = executor.submit(make_call, i)
                    futures.append(future)
                    # Small delay to stagger requests
                    time.sleep(0.01)

                # Wait for all to complete
                for future in as_completed(futures, timeout=10):
                    future.result()

            print(f"Success: {success_count}, Errors: {error_count}")

            # With limit of 2 and 15 requests staggered by 10ms with 300ms sleep each:
            # - First 2 start immediately and hold permits for 300ms
            # - By the time they finish, we've submitted all 15 requests (15 * 10ms = 150ms)
            # - Most of the remaining 13 will be rejected immediately
            # Expected: 2-6 succeed (first batch), 9+ rejected
            total = success_count + error_count
            assert total == 15, f"Total requests should be 15, got {total}"
            assert 2 <= success_count <= 6, f"Expected 2-6 successful requests, got {success_count}"
            assert error_count >= 9, f"Expected at least 9 rejected requests, got {error_count}"

            # Check rejected count
            rejected_count = plugin.rejected_request_count
            assert rejected_count >= 9, f"Rejected count should be at least 9, got {rejected_count}"
            assert (
                error_count == rejected_count
            ), f"Error count ({error_count}) should match rejected count ({rejected_count})"

            print(f"Rejected count: {rejected_count}")

    def test_concurrency_limit___unlimited___all_succeed(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Concurrency limit of zero means unlimited."""
        config = PluginConfig.defaults().max_concurrent_ops(0)  # Unlimited

        with NativePluginLoader.load_with_config(str(hello_plugin_path), config) as plugin:
            success_count = 0
            error_count = 0
            lock = threading.Lock()

            def make_call(call_id: int) -> bool:
                """Make a greet call."""
                nonlocal success_count, error_count
                try:
                    plugin.call("greet", f'{{"name": "User{call_id}"}}')
                    with lock:
                        success_count += 1
                    return True
                except PluginException as e:
                    with lock:
                        error_count += 1
                    raise AssertionError(
                        f"No requests should fail with unlimited concurrency: {e}"
                    )

            # Submit many concurrent requests
            with ThreadPoolExecutor(max_workers=20) as executor:
                futures = [executor.submit(make_call, i) for i in range(20)]

                for future in as_completed(futures, timeout=5):
                    future.result()

            # All should succeed
            assert success_count == 20, f"All requests should succeed, got {success_count}"
            assert error_count == 0, f"No requests should fail, got {error_count}"
            assert (
                plugin.rejected_request_count == 0
            ), f"No requests should be rejected, got {plugin.rejected_request_count}"

    def test_concurrency_limit___permit_released___after_completion(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Permits released after completion allows sequential calls."""
        config = PluginConfig.defaults().max_concurrent_ops(1)

        with NativePluginLoader.load_with_config(str(hello_plugin_path), config) as plugin:
            # Make several sequential calls
            for i in range(10):
                result = plugin.call("greet", f'{{"name": "User{i}"}}')
                assert result is not None

            # All should succeed since they're sequential
            assert plugin.rejected_request_count == 0, (
                f"No requests should be rejected for sequential calls, "
                f"got {plugin.rejected_request_count}"
            )

    def test_rejected_request_count___tracks_rejected_requests(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Rejected request count tracks rejected requests."""
        config = PluginConfig.defaults().max_concurrent_ops(2)

        with NativePluginLoader.load_with_config(str(hello_plugin_path), config) as plugin:
            # Use a barrier to release all threads at once to create contention
            barrier = threading.Barrier(20)

            def make_call(call_id: int) -> str | None:
                """Make a slow call after waiting at barrier to hold permits."""
                try:
                    # Wait for all threads to be ready
                    barrier.wait(timeout=5)
                    # Use sleep handler to hold permits longer (100ms)
                    return plugin.call("test.sleep", '{"duration_ms": 100}')
                except PluginException:
                    return None
                except threading.BrokenBarrierError:
                    return None

            # Submit many concurrent requests simultaneously
            with ThreadPoolExecutor(max_workers=20) as executor:
                futures = [executor.submit(make_call, i) for i in range(20)]

                # Wait for all to complete
                for future in as_completed(futures, timeout=10):
                    try:
                        future.result()
                    except Exception:
                        pass  # Ignore errors for this test

            # Check that some requests were rejected
            rejected_count = plugin.rejected_request_count
            assert rejected_count > 0, (
                f"Some requests should have been rejected with limit of 2 "
                f"and 20 concurrent requests, got {rejected_count}"
            )

            print(f"Rejected count: {rejected_count}")
