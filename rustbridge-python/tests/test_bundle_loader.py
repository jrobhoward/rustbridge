"""Tests for BundleLoader."""

import hashlib
import io
import json
import platform
import tempfile
import zipfile
from pathlib import Path

import pytest

from rustbridge import BundleLoader, BundleManifest, PlatformInfo, SchemaInfo, PluginException


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

    def test_from_json___with_schemas___parses_schemas(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"name": "test", "version": "1.0.0"},
            "platforms": {},
            "schemas": {
                "messages.h": {
                    "path": "schemas/messages.h",
                    "checksum": "sha256:abc123",
                    "format": "c-header",
                    "description": "C header for binary messages",
                },
                "api.json": {
                    "path": "schemas/api.json",
                    "checksum": "sha256:def456",
                    "format": "json-schema",
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        assert len(manifest.schemas) == 2
        assert "messages.h" in manifest.schemas
        assert manifest.schemas["messages.h"].path == "schemas/messages.h"
        assert manifest.schemas["messages.h"].format == "c-header"
        assert manifest.schemas["api.json"].checksum == "sha256:def456"


def _create_test_bundle_with_schemas(
    schemas: dict[str, str],
) -> tuple[bytes, dict[str, str]]:
    """
    Create a test bundle ZIP with the given schemas.

    Args:
        schemas: Dictionary mapping schema name to schema content.

    Returns:
        Tuple of (zip_bytes, checksums) where checksums maps schema name to its SHA256.
    """
    buffer = io.BytesIO()
    checksums: dict[str, str] = {}

    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as zf:
        # Calculate checksums and add schema files
        schema_manifest: dict[str, dict[str, str]] = {}
        for name, content in schemas.items():
            content_bytes = content.encode("utf-8")
            checksum = hashlib.sha256(content_bytes).hexdigest()
            checksums[name] = checksum
            path = f"schemas/{name}"
            zf.writestr(path, content_bytes)
            schema_manifest[name] = {
                "path": path,
                "checksum": f"sha256:{checksum}",
                "format": "text",
            }

        # Create manifest
        manifest = {
            "bundle_version": "1.0",
            "plugin": {"name": "test-plugin", "version": "1.0.0"},
            "platforms": {},
            "schemas": schema_manifest,
        }
        zf.writestr("manifest.json", json.dumps(manifest))

    return buffer.getvalue(), checksums


class TestBundleLoaderSchemas:
    """Tests for BundleLoader schema extraction."""

    def test_get_schemas___bundle_with_schemas___returns_schema_dict(self) -> None:
        schemas = {
            "messages.h": "// Test header\nstruct TestMessage {};",
            "api.json": '{"type": "object"}',
        }
        bundle_bytes, _ = _create_test_bundle_with_schemas(schemas)

        with tempfile.NamedTemporaryFile(suffix=".rbp", delete=False) as f:
            f.write(bundle_bytes)
            bundle_path = Path(f.name)

        try:
            loader = BundleLoader(verify_signatures=False)

            result = loader.get_schemas(bundle_path)

            assert len(result) == 2
            assert "messages.h" in result
            assert "api.json" in result
            assert result["messages.h"].path == "schemas/messages.h"
        finally:
            bundle_path.unlink()

    def test_get_schemas___bundle_without_schemas___returns_empty_dict(self) -> None:
        bundle_bytes, _ = _create_test_bundle_with_schemas({})

        with tempfile.NamedTemporaryFile(suffix=".rbp", delete=False) as f:
            f.write(bundle_bytes)
            bundle_path = Path(f.name)

        try:
            loader = BundleLoader(verify_signatures=False)

            result = loader.get_schemas(bundle_path)

            assert result == {}
        finally:
            bundle_path.unlink()

    def test_read_schema___existing_schema___returns_content(self) -> None:
        schema_content = "// Test C header\nstruct Message { int32_t id; };"
        schemas = {"messages.h": schema_content}
        bundle_bytes, _ = _create_test_bundle_with_schemas(schemas)

        with tempfile.NamedTemporaryFile(suffix=".rbp", delete=False) as f:
            f.write(bundle_bytes)
            bundle_path = Path(f.name)

        try:
            loader = BundleLoader(verify_signatures=False)

            result = loader.read_schema(bundle_path, "messages.h")

            assert result == schema_content
        finally:
            bundle_path.unlink()

    def test_read_schema___nonexistent_schema___raises_exception(self) -> None:
        bundle_bytes, _ = _create_test_bundle_with_schemas({})

        with tempfile.NamedTemporaryFile(suffix=".rbp", delete=False) as f:
            f.write(bundle_bytes)
            bundle_path = Path(f.name)

        try:
            loader = BundleLoader(verify_signatures=False)

            with pytest.raises(PluginException, match="Schema not found"):
                loader.read_schema(bundle_path, "nonexistent.h")
        finally:
            bundle_path.unlink()

    def test_extract_schema___existing_schema___extracts_to_file(self) -> None:
        schema_content = '{"$schema": "http://json-schema.org/draft-07/schema#"}'
        schemas = {"api.json": schema_content}
        bundle_bytes, _ = _create_test_bundle_with_schemas(schemas)

        with tempfile.NamedTemporaryFile(suffix=".rbp", delete=False) as f:
            f.write(bundle_bytes)
            bundle_path = Path(f.name)

        try:
            loader = BundleLoader(verify_signatures=False)

            with tempfile.TemporaryDirectory() as dest_dir:
                result_path = loader.extract_schema(bundle_path, "api.json", dest_dir)

                assert result_path.exists()
                assert result_path.name == "api.json"
                assert result_path.read_text() == schema_content
        finally:
            bundle_path.unlink()

    def test_extract_schema___nonexistent_schema___raises_exception(self) -> None:
        bundle_bytes, _ = _create_test_bundle_with_schemas({})

        with tempfile.NamedTemporaryFile(suffix=".rbp", delete=False) as f:
            f.write(bundle_bytes)
            bundle_path = Path(f.name)

        try:
            loader = BundleLoader(verify_signatures=False)

            with tempfile.TemporaryDirectory() as dest_dir:
                with pytest.raises(PluginException, match="Schema not found"):
                    loader.extract_schema(bundle_path, "nonexistent.h", dest_dir)
        finally:
            bundle_path.unlink()

    def test_extract_schema___corrupted_checksum___raises_exception(self) -> None:
        schema_content = "test content"
        schemas = {"test.txt": schema_content}
        bundle_bytes, _ = _create_test_bundle_with_schemas(schemas)

        # Corrupt the bundle by modifying the schema content but keeping old checksum
        buffer = io.BytesIO(bundle_bytes)
        with zipfile.ZipFile(buffer, "a") as zf:
            # Overwrite with different content
            zf.writestr("schemas/test.txt", "corrupted content")

        with tempfile.NamedTemporaryFile(suffix=".rbp", delete=False) as f:
            f.write(buffer.getvalue())
            bundle_path = Path(f.name)

        try:
            loader = BundleLoader(verify_signatures=False)

            with pytest.raises(PluginException, match="Checksum verification failed"):
                loader.read_schema(bundle_path, "test.txt")
        finally:
            bundle_path.unlink()
