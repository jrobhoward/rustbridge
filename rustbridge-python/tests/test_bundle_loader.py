"""Tests for BundleLoader."""

import json
import platform
import pytest

from rustbridge import BundleLoader, BundleManifest, PlatformInfo, PluginException


class TestBundleLoader:
    """Tests for BundleLoader."""

    def test_get_current_platform___linux_x86_64(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setattr(platform, "system", lambda: "Linux")
        monkeypatch.setattr(platform, "machine", lambda: "x86_64")

        result = BundleLoader.get_current_platform()

        assert result == "linux-x86_64"

    def test_get_current_platform___darwin_aarch64(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setattr(platform, "system", lambda: "Darwin")
        monkeypatch.setattr(platform, "machine", lambda: "arm64")

        result = BundleLoader.get_current_platform()

        assert result == "darwin-aarch64"

    def test_get_current_platform___windows_x86_64(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setattr(platform, "system", lambda: "Windows")
        monkeypatch.setattr(platform, "machine", lambda: "AMD64")

        result = BundleLoader.get_current_platform()

        assert result == "windows-x86_64"

    def test_load___file_not_found___raises_file_not_found(self) -> None:
        loader = BundleLoader(verify_signatures=False)

        with pytest.raises(FileNotFoundError):
            loader.load("/nonexistent/path/bundle.rbp")


class TestBundleManifest:
    """Tests for BundleManifest parsing."""

    def test_from_json___valid_manifest___parses_correctly(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {
                "name": "test-plugin",
                "version": "1.0.0",
                "description": "A test plugin",
            },
            "platforms": {
                "linux-x86_64": {
                    "library": "lib/linux-x86_64/libtest.so",
                    "checksum": "sha256:abc123",
                }
            },
            "public_key": "RWS...",
        })

        manifest = BundleManifest.from_json(manifest_json)

        assert manifest.bundle_version == "1.0"
        assert manifest.plugin_name == "test-plugin"
        assert manifest.plugin_version == "1.0.0"
        assert manifest.public_key == "RWS..."
        assert "linux-x86_64" in manifest.platforms
        assert manifest.platforms["linux-x86_64"].library == "lib/linux-x86_64/libtest.so"

    def test_from_json___missing_bundle_version___raises_exception(self) -> None:
        manifest_json = json.dumps({
            "plugin": {"name": "test", "version": "1.0.0"},
            "platforms": {},
        })

        with pytest.raises(PluginException, match="bundle_version"):
            BundleManifest.from_json(manifest_json)

    def test_from_json___missing_plugin_name___raises_exception(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"version": "1.0.0"},
            "platforms": {},
        })

        with pytest.raises(PluginException, match="plugin.name"):
            BundleManifest.from_json(manifest_json)

    def test_from_json___missing_plugin_version___raises_exception(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"name": "test"},
            "platforms": {},
        })

        with pytest.raises(PluginException, match="plugin.version"):
            BundleManifest.from_json(manifest_json)

    def test_from_json___invalid_json___raises_exception(self) -> None:
        with pytest.raises(PluginException, match="Failed to parse"):
            BundleManifest.from_json("not valid json")

    def test_get_platform___existing___returns_platform_info(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"name": "test", "version": "1.0.0"},
            "platforms": {
                "linux-x86_64": {
                    "library": "lib/linux-x86_64/libtest.so",
                    "checksum": "sha256:abc123",
                }
            },
        })

        manifest = BundleManifest.from_json(manifest_json)
        platform_info = manifest.get_platform("linux-x86_64")

        assert platform_info is not None
        assert platform_info.library == "lib/linux-x86_64/libtest.so"
        assert platform_info.checksum == "sha256:abc123"

    def test_get_platform___non_existing___returns_none(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"name": "test", "version": "1.0.0"},
            "platforms": {},
        })

        manifest = BundleManifest.from_json(manifest_json)

        assert manifest.get_platform("linux-x86_64") is None
