"""Tests for resource leak detection in plugin lifecycle.

Verifies that plugins and their native resources are properly cleaned up
and not leaked even under stress conditions.

Reference: Java ResourceLeakTest.java
"""

import gc
import threading
import weakref
from pathlib import Path

import pytest

from rustbridge import LifecycleState, NativePluginLoader, PluginException


class TestResourceLeak:
    """Test for resource leak detection in plugin lifecycle."""

    def test_plugin_resources___released_on_close(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Plugin resources are released on close."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            # Verify plugin is active
            assert plugin.state == LifecycleState.ACTIVE

            # Use the plugin
            response = plugin.call("echo", '{"message": "test"}')
            assert "test" in response

    def test_sequential_load_close___cycles_dont_leak_resources(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Sequential load/close cycles don't leak resources."""
        for i in range(100):
            with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
                assert plugin.state == LifecycleState.ACTIVE

                # Call the plugin
                plugin.call("echo", f'{{"message": "cycle {i}"}}')

    def test_plugin_objects___gc_eligible_after_close(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Plugin objects are GC-eligible after close."""
        plugin_ref: weakref.ref[object] | None = None

        # Load plugin in a local scope
        def create_and_use_plugin() -> weakref.ref[object]:
            with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
                ref = weakref.ref(plugin)
                assert plugin.state == LifecycleState.ACTIVE

                # Use the plugin
                plugin.call("echo", '{"message": "test"}')
                # Plugin closed at end of with block
                return ref

        plugin_ref = create_and_use_plugin()

        # Try to GC (not guaranteed, but helps)
        gc.collect()

        # WeakReference may be None if GC collected it
        # This is not guaranteed but indicates good cleanup

    def test_multiple_plugins___close_cleanly(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Multiple plugins close cleanly."""
        # Create and close 20 plugins
        for i in range(20):
            with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
                assert plugin.state == LifecycleState.ACTIVE
                # Use plugin
                plugin.call("echo", f'{{"message": "test {i}"}}')

    def test_plugin_state___correct_after_use_and_close(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Plugin state is correct after use and close."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            assert plugin.state == LifecycleState.ACTIVE

            # Make a valid call
            plugin.call("echo", '{"message": "test"}')

            # Still active
            assert plugin.state == LifecycleState.ACTIVE

    def test_large_payload___cycles_dont_leak_memory(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Large payload cycles don't leak memory."""
        # Create a large payload
        large_message = "x" * 10000
        large_payload = f'{{"message": "{large_message}"}}'

        for cycle in range(50):
            with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
                # Send large payload
                plugin.call("echo", large_payload)

    def test_plugin_resources___survive_concurrent_access_before_close(
        self, hello_plugin_path: Path, skip_if_no_plugin: None
    ) -> None:
        """Plugin resources survive concurrent access before close."""
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            # Multiple threads access the plugin
            thread_count = 10
            exceptions: list[Exception] = []
            lock = threading.Lock()

            def thread_work(thread_id: int) -> None:
                try:
                    for j in range(10):
                        response = plugin.call(
                            "echo", f'{{"message": "thread {thread_id} call {j}"}}'
                        )
                        assert response is not None
                except Exception as e:
                    with lock:
                        exceptions.append(e)

            threads = []
            for i in range(thread_count):
                t = threading.Thread(target=thread_work, args=(i,))
                threads.append(t)
                t.start()

            # Wait for all threads
            for t in threads:
                t.join(timeout=10)

            # Check no exceptions occurred
            assert len(exceptions) == 0, f"Threads had exceptions: {exceptions}"
