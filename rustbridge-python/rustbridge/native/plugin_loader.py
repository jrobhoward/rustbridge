"""Plugin loader factory for native plugins."""

from __future__ import annotations

import os
import platform
import sys
from pathlib import Path
from typing import Callable

from rustbridge.core.log_level import LogLevel
from rustbridge.core.plugin_config import PluginConfig
from rustbridge.core.plugin_exception import PluginException
from rustbridge.native.library import NativeLibrary
from rustbridge.native.native_plugin import NativePlugin
from rustbridge.native.structures import LogCallbackFnType

# Type alias for log callback
LogCallbackFn = Callable[[LogLevel, str, str], None]


class NativePluginLoader:
    """
    Factory for loading native plugins.

    Example:
        # Simple loading
        with NativePluginLoader.load("libmyplugin.so") as plugin:
            response = plugin.call("echo", '{"message": "hello"}')

        # With configuration
        config = PluginConfig.defaults().log_level(LogLevel.DEBUG)
        with NativePluginLoader.load_with_config("libmyplugin.so", config) as plugin:
            response = plugin.call("echo", '{"message": "hello"}')

        # With log callback
        def on_log(level: LogLevel, target: str, message: str):
            print(f"[{level.name}] {target}: {message}")

        plugin = NativePluginLoader.load_with_config(
            "libmyplugin.so",
            PluginConfig.defaults(),
            log_callback=on_log
        )
    """

    @staticmethod
    def load(library_path: str | Path) -> NativePlugin:
        """
        Load a plugin from the specified library path.

        Args:
            library_path: Path to the shared library.

        Returns:
            The loaded plugin.

        Raises:
            PluginException: If loading fails.
        """
        return NativePluginLoader.load_with_config(
            library_path, PluginConfig.defaults(), None
        )

    @staticmethod
    def load_with_config(
        library_path: str | Path,
        config: PluginConfig,
        log_callback: LogCallbackFn | None = None,
    ) -> NativePlugin:
        """
        Load a plugin with configuration.

        Args:
            library_path: Path to the shared library.
            config: Plugin configuration.
            log_callback: Optional callback for log messages.

        Returns:
            The loaded plugin.

        Raises:
            PluginException: If loading fails.
        """
        library = NativeLibrary(library_path)

        try:
            # Create the plugin instance
            plugin_ptr = library.plugin_create()
            if not plugin_ptr:
                raise PluginException("plugin_create returned null")

            # Prepare config
            config_bytes = config.to_json_bytes()

            # Prepare log callback
            callback_ref = None
            if log_callback:
                callback_ref = NativePluginLoader._create_log_callback(log_callback)

            # Initialize the plugin
            handle = library.plugin_init(plugin_ptr, config_bytes, callback_ref)
            if not handle:
                raise PluginException("plugin_init returned null handle")

            return NativePlugin(library, handle, log_callback, callback_ref)

        except Exception:
            raise

    @staticmethod
    def load_by_name(library_name: str) -> NativePlugin:
        """
        Load a plugin by name, searching in standard library paths.

        Args:
            library_name: The library name (without lib prefix or extension).

        Returns:
            The loaded plugin.

        Raises:
            PluginException: If loading fails.
        """
        return NativePluginLoader.load_by_name_with_config(
            library_name, PluginConfig.defaults()
        )

    @staticmethod
    def load_by_name_with_config(
        library_name: str, config: PluginConfig
    ) -> NativePlugin:
        """
        Load a plugin by name with configuration.

        Args:
            library_name: The library name (without lib prefix or extension).
            config: Plugin configuration.

        Returns:
            The loaded plugin.

        Raises:
            PluginException: If loading fails.
        """
        library_filename = NativePluginLoader._get_library_filename(library_name)

        search_paths = [
            ".",
            "./target/release",
            "./target/debug",
        ]

        # Add PATH directories
        path_env = os.environ.get("PATH", "")
        if path_env:
            search_paths.extend(path_env.split(os.pathsep))

        for base_path in search_paths:
            if not base_path:
                continue

            full_path = Path(base_path) / library_filename
            if full_path.exists():
                return NativePluginLoader.load_with_config(full_path, config, None)

        raise PluginException(f"Could not find library: {library_filename}")

    @staticmethod
    def _get_library_filename(library_name: str) -> str:
        """Get the platform-specific library filename."""
        system = platform.system()

        if system == "Linux":
            return f"lib{library_name}.so"
        elif system == "Darwin":
            return f"lib{library_name}.dylib"
        elif system == "Windows":
            return f"{library_name}.dll"
        else:
            raise PluginException(f"Unsupported operating system: {system}")

    @staticmethod
    def _create_log_callback(callback: LogCallbackFn) -> LogCallbackFnType:
        """Create a ctypes-compatible log callback wrapper."""

        def wrapper(
            level: int, target: bytes | None, message: bytes | None, message_len: int
        ) -> None:
            try:
                log_level = LogLevel.from_code(level)
                target_str = target.decode("utf-8") if target else ""
                message_str = message[:message_len].decode("utf-8") if message else ""
                callback(log_level, target_str, message_str)
            except Exception as e:
                # Don't let exceptions propagate back to native code
                print(f"Error in log callback: {e}", file=sys.stderr)

        return LogCallbackFnType(wrapper)
