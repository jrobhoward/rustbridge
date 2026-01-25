"""Tests for PluginConfig."""

import json

import pytest

from rustbridge import PluginConfig, LogLevel


class TestPluginConfig:
    """Tests for PluginConfig."""

    def test_defaults___creates_default_config(self) -> None:
        config = PluginConfig.defaults()

        config_dict = config.to_dict()

        assert config_dict["log_level"] == "info"
        assert config_dict["max_concurrent_ops"] == 1000
        assert config_dict["shutdown_timeout_ms"] == 5000
        assert config_dict["data"] == {}
        assert "worker_threads" not in config_dict
        assert "init_params" not in config_dict

    def test_log_level___with_enum___sets_level(self) -> None:
        config = PluginConfig.defaults().log_level(LogLevel.DEBUG)

        assert config.to_dict()["log_level"] == "debug"

    def test_log_level___with_string___sets_level(self) -> None:
        config = PluginConfig.defaults().log_level("WARN")

        assert config.to_dict()["log_level"] == "warn"

    def test_worker_threads___sets_threads(self) -> None:
        config = PluginConfig.defaults().worker_threads(4)

        assert config.to_dict()["worker_threads"] == 4

    def test_max_concurrent_ops___sets_ops(self) -> None:
        config = PluginConfig.defaults().max_concurrent_ops(500)

        assert config.to_dict()["max_concurrent_ops"] == 500

    def test_shutdown_timeout_ms___sets_timeout(self) -> None:
        config = PluginConfig.defaults().shutdown_timeout_ms(10000)

        assert config.to_dict()["shutdown_timeout_ms"] == 10000

    def test_set___adds_custom_data(self) -> None:
        config = PluginConfig.defaults().set("key1", "value1").set("key2", 42)

        data = config.to_dict()["data"]

        assert data["key1"] == "value1"
        assert data["key2"] == 42

    def test_init_param___adds_init_params(self) -> None:
        config = PluginConfig.defaults().init_param("db_url", "postgres://...")

        assert config.to_dict()["init_params"]["db_url"] == "postgres://..."

    def test_init_params___replaces_all_params(self) -> None:
        config = (
            PluginConfig.defaults()
            .init_param("old_key", "old_value")
            .init_params({"new_key": "new_value"})
        )

        init_params = config.to_dict()["init_params"]

        assert "old_key" not in init_params
        assert init_params["new_key"] == "new_value"

    def test_fluent_chaining___all_methods___returns_self(self) -> None:
        config = (
            PluginConfig.defaults()
            .log_level(LogLevel.DEBUG)
            .worker_threads(4)
            .max_concurrent_ops(500)
            .shutdown_timeout_ms(10000)
            .set("key", "value")
            .init_param("param", "value")
        )

        assert isinstance(config, PluginConfig)

    def test_to_json_bytes___returns_valid_json(self) -> None:
        config = (
            PluginConfig.defaults()
            .log_level(LogLevel.DEBUG)
            .set("custom", "value")
        )

        json_bytes = config.to_json_bytes()

        parsed = json.loads(json_bytes.decode("utf-8"))

        assert parsed["log_level"] == "debug"
        assert parsed["data"]["custom"] == "value"
        assert parsed["max_concurrent_ops"] == 1000
        assert parsed["shutdown_timeout_ms"] == 5000
