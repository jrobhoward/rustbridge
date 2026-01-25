"""Plugin configuration with fluent builder pattern."""

from __future__ import annotations

import json
from typing import Any

from rustbridge.core.log_level import LogLevel


class PluginConfig:
    """
    Configuration for plugin initialization.

    Uses a fluent builder pattern for easy configuration:

        config = (PluginConfig.defaults()
            .log_level(LogLevel.DEBUG)
            .worker_threads(4)
            .max_concurrent_ops(500)
            .set("my_key", "my_value"))
    """

    def __init__(self) -> None:
        """Create a new empty configuration."""
        self._data: dict[str, Any] = {}
        self._init_params: dict[str, Any] | None = None
        self._worker_threads: int | None = None
        self._log_level: str = "info"
        self._max_concurrent_ops: int = 1000
        self._shutdown_timeout_ms: int = 5000

    @classmethod
    def defaults(cls) -> PluginConfig:
        """Create a configuration with default settings."""
        return cls()

    def worker_threads(self, threads: int) -> PluginConfig:
        """
        Set the number of worker threads.

        Args:
            threads: The number of threads.

        Returns:
            This config for chaining.
        """
        self._worker_threads = threads
        return self

    def log_level(self, level: LogLevel | str) -> PluginConfig:
        """
        Set the log level.

        Args:
            level: The log level (LogLevel enum or string).

        Returns:
            This config for chaining.
        """
        if isinstance(level, LogLevel):
            self._log_level = level.to_string()
        else:
            self._log_level = level.lower()
        return self

    def max_concurrent_ops(self, max_ops: int) -> PluginConfig:
        """
        Set the maximum concurrent operations.

        Args:
            max_ops: The maximum concurrent operations.

        Returns:
            This config for chaining.
        """
        self._max_concurrent_ops = max_ops
        return self

    def shutdown_timeout_ms(self, timeout_ms: int) -> PluginConfig:
        """
        Set the shutdown timeout.

        Args:
            timeout_ms: The timeout in milliseconds.

        Returns:
            This config for chaining.
        """
        self._shutdown_timeout_ms = timeout_ms
        return self

    def set(self, key: str, value: Any) -> PluginConfig:
        """
        Set a custom configuration value.

        Args:
            key: The configuration key.
            value: The configuration value.

        Returns:
            This config for chaining.
        """
        self._data[key] = value
        return self

    def init_param(self, key: str, value: Any) -> PluginConfig:
        """
        Set an initialization parameter.

        Initialization parameters are passed to the plugin during startup and are
        intended for one-time setup configuration (migrations, seed data, etc.).

        Args:
            key: The parameter key.
            value: The parameter value.

        Returns:
            This config for chaining.
        """
        if self._init_params is None:
            self._init_params = {}
        self._init_params[key] = value
        return self

    def init_params(self, parameters: dict[str, Any]) -> PluginConfig:
        """
        Set all initialization parameters at once.

        This replaces any existing init params.

        Args:
            parameters: Map of initialization parameters.

        Returns:
            This config for chaining.
        """
        self._init_params = dict(parameters)
        return self

    def to_json_bytes(self) -> bytes:
        """
        Serialize the configuration to JSON bytes.

        Returns:
            The JSON bytes.
        """
        config: dict[str, Any] = {
            "data": self._data,
            "log_level": self._log_level,
            "max_concurrent_ops": self._max_concurrent_ops,
            "shutdown_timeout_ms": self._shutdown_timeout_ms,
        }

        if self._init_params:
            config["init_params"] = self._init_params

        if self._worker_threads is not None:
            config["worker_threads"] = self._worker_threads

        return json.dumps(config).encode("utf-8")

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the configuration to a dictionary.

        Returns:
            The configuration as a dictionary.
        """
        config: dict[str, Any] = {
            "data": self._data,
            "log_level": self._log_level,
            "max_concurrent_ops": self._max_concurrent_ops,
            "shutdown_timeout_ms": self._shutdown_timeout_ms,
        }

        if self._init_params:
            config["init_params"] = self._init_params

        if self._worker_threads is not None:
            config["worker_threads"] = self._worker_threads

        return config
