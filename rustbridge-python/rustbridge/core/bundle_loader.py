"""Bundle loader for .rbp plugin bundles."""

from __future__ import annotations

import hashlib
import os
import platform
import stat
import tempfile
import zipfile
from pathlib import Path
from typing import TYPE_CHECKING, Callable

from rustbridge.core.bundle_manifest import BundleManifest, SchemaInfo
from rustbridge.core.minisign_verifier import MinisignVerifier
from rustbridge.core.plugin_exception import PluginException

if TYPE_CHECKING:
    from rustbridge.core.log_level import LogLevel
    from rustbridge.core.plugin_config import PluginConfig
    from rustbridge.native.native_plugin import NativePlugin

# Type alias for log callback
LogCallbackFn = Callable[["LogLevel", str, str], None]


class BundleLoader:
    """
    Loader for RustBridge plugin bundles (.rbp files).

    Provides functionality to:
    - Extract and parse bundle manifests
    - Extract platform-specific libraries
    - Verify SHA256 checksums
    - Verify minisign signatures (optional)

    Example:
        # Load with signature verification (default)
        loader = BundleLoader(verify_signatures=True)
        with loader.load("my-plugin-1.0.0.rbp") as plugin:
            response = plugin.call("echo", '{"message": "hello"}')

        # Load without signature verification (development only)
        loader = BundleLoader(verify_signatures=False)
        with loader.load("my-plugin-1.0.0.rbp") as plugin:
            ...
    """

    def __init__(
        self,
        verify_signatures: bool = True,
        public_key_override: str | None = None,
    ) -> None:
        """
        Create a new BundleLoader.

        Args:
            verify_signatures: Whether to verify minisign signatures (default: True).
            public_key_override: Optional public key to use instead of manifest's key.
        """
        self._verify_signatures = verify_signatures
        self._public_key_override = public_key_override

    def load(self, bundle_path: str | Path) -> "NativePlugin":
        """
        Load plugin from .rbp bundle.

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            The loaded NativePlugin.

        Raises:
            PluginException: If loading fails.
            FileNotFoundError: If bundle file doesn't exist.
        """
        from rustbridge.core.plugin_config import PluginConfig

        return self.load_with_config(bundle_path, PluginConfig.defaults())

    def load_with_config(
        self,
        bundle_path: str | Path,
        config: "PluginConfig",
        log_callback: LogCallbackFn | None = None,
    ) -> "NativePlugin":
        """
        Load plugin from .rbp bundle with configuration.

        Args:
            bundle_path: Path to the .rbp bundle file.
            config: Plugin configuration.
            log_callback: Optional callback for log messages.

        Returns:
            The loaded NativePlugin.

        Raises:
            PluginException: If loading fails.
            FileNotFoundError: If bundle file doesn't exist.
        """
        from rustbridge.native.plugin_loader import NativePluginLoader

        bundle_path = Path(bundle_path)
        if not bundle_path.exists():
            raise FileNotFoundError(f"Bundle not found: {bundle_path}")

        # Extract library to temp directory
        temp_dir = tempfile.mkdtemp(prefix="rustbridge-")
        try:
            lib_path = self.extract_library(bundle_path, Path(temp_dir))
            return NativePluginLoader.load_with_config(
                str(lib_path), config, log_callback
            )
        except Exception:
            # Clean up temp directory on failure
            import shutil

            shutil.rmtree(temp_dir, ignore_errors=True)
            raise

    def extract_library(self, bundle_path: Path, dest_dir: Path) -> Path:
        """
        Extract and verify library from bundle.

        Args:
            bundle_path: Path to the .rbp bundle file.
            dest_dir: Directory to extract the library to.

        Returns:
            Path to the extracted library file.

        Raises:
            PluginException: If extraction or verification fails.
        """
        with zipfile.ZipFile(bundle_path, "r") as zip_file:
            # Load manifest
            manifest = self._load_manifest(zip_file)

            # Verify manifest signature if enabled
            if self._verify_signatures:
                self._verify_manifest_signature(zip_file, manifest)

            # Detect platform
            current_platform = self.get_current_platform()
            platform_info = manifest.get_platform(current_platform)
            if not platform_info:
                raise PluginException(f"Platform not supported: {current_platform}")

            # Read library data
            lib_data = self._read_zip_entry(zip_file, platform_info.library)

            # Verify checksum
            if not self._verify_checksum(lib_data, platform_info.checksum):
                raise PluginException(
                    f"Checksum verification failed for {platform_info.library}"
                )

            # Verify library signature if enabled
            if self._verify_signatures:
                self._verify_library_signature(
                    zip_file, manifest, platform_info.library, lib_data
                )

            # Write to output directory
            lib_filename = Path(platform_info.library).name
            output_path = dest_dir / lib_filename
            output_path.write_bytes(lib_data)

            # Make executable on Unix
            if os.name != "nt":
                current_mode = output_path.stat().st_mode
                output_path.chmod(
                    current_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH
                )

            return output_path

    def get_manifest(self, bundle_path: str | Path) -> BundleManifest:
        """
        Read the manifest from a bundle without extracting.

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            The parsed BundleManifest.

        Raises:
            PluginException: If manifest cannot be read or parsed.
        """
        with zipfile.ZipFile(bundle_path, "r") as zip_file:
            return self._load_manifest(zip_file)

    def list_files(self, bundle_path: str | Path) -> list[str]:
        """
        List all files in the bundle.

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            List of file paths within the bundle.
        """
        with zipfile.ZipFile(bundle_path, "r") as zip_file:
            return zip_file.namelist()

    def get_schemas(self, bundle_path: str | Path) -> dict[str, SchemaInfo]:
        """
        Get all available schemas in the bundle.

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            Dictionary mapping schema names to SchemaInfo objects.
        """
        manifest = self.get_manifest(bundle_path)
        return manifest.schemas

    def extract_schema(
        self, bundle_path: str | Path, schema_name: str, dest_dir: str | Path
    ) -> Path:
        """
        Extract a schema file from the bundle.

        Args:
            bundle_path: Path to the .rbp bundle file.
            schema_name: Name of the schema (e.g., "messages.h").
            dest_dir: Directory to extract the schema to.

        Returns:
            Path to the extracted schema file.

        Raises:
            PluginException: If extraction fails or schema not found.
        """
        bundle_path = Path(bundle_path)
        dest_dir = Path(dest_dir)

        with zipfile.ZipFile(bundle_path, "r") as zip_file:
            manifest = self._load_manifest(zip_file)

            schema_info = manifest.schemas.get(schema_name)
            if not schema_info:
                raise PluginException(f"Schema not found in bundle: {schema_name}")

            # Read schema data
            schema_data = self._read_zip_entry(zip_file, schema_info.path)

            # Verify checksum
            if not self._verify_checksum(schema_data, schema_info.checksum):
                raise PluginException(
                    f"Checksum verification failed for schema {schema_name}"
                )

            # Write to output directory
            output_path = dest_dir / schema_name
            output_path.write_bytes(schema_data)

            return output_path

    def read_schema(self, bundle_path: str | Path, schema_name: str) -> str:
        """
        Read a schema file content as string.

        Args:
            bundle_path: Path to the .rbp bundle file.
            schema_name: Name of the schema (e.g., "messages.h").

        Returns:
            Schema file content as a string.

        Raises:
            PluginException: If reading fails or schema not found.
        """
        with zipfile.ZipFile(bundle_path, "r") as zip_file:
            manifest = self._load_manifest(zip_file)

            schema_info = manifest.schemas.get(schema_name)
            if not schema_info:
                raise PluginException(f"Schema not found in bundle: {schema_name}")

            # Read schema data
            schema_data = self._read_zip_entry(zip_file, schema_info.path)

            # Verify checksum
            if not self._verify_checksum(schema_data, schema_info.checksum):
                raise PluginException(
                    f"Checksum verification failed for schema {schema_name}"
                )

            return schema_data.decode("utf-8")

    @staticmethod
    def get_current_platform() -> str:
        """
        Return platform string for current system.

        Returns:
            Platform string like "linux-x86_64", "darwin-aarch64", etc.
        """
        system = platform.system().lower()
        machine = platform.machine().lower()

        os_map = {"linux": "linux", "darwin": "darwin", "windows": "windows"}
        arch_map = {
            "x86_64": "x86_64",
            "amd64": "x86_64",
            "aarch64": "aarch64",
            "arm64": "aarch64",
        }

        os_name = os_map.get(system, system)
        arch_name = arch_map.get(machine, machine)

        return f"{os_name}-{arch_name}"

    def _load_manifest(self, zip_file: zipfile.ZipFile) -> BundleManifest:
        """Load and parse the manifest from a zip file."""
        manifest_data = self._read_zip_entry(zip_file, "manifest.json")
        return BundleManifest.from_json(manifest_data.decode("utf-8"))

    def _verify_manifest_signature(
        self, zip_file: zipfile.ZipFile, manifest: BundleManifest
    ) -> None:
        """Verify the manifest signature."""
        public_key = self._public_key_override or manifest.public_key

        if not public_key:
            raise PluginException(
                "Signature verification enabled but no public key available. "
                "Bundle must include public_key in manifest, or provide via public_key_override."
            )

        # Read manifest data
        manifest_data = self._read_zip_entry(zip_file, "manifest.json")

        # Read signature
        try:
            sig_data = self._read_zip_entry(zip_file, "manifest.json.minisig")
        except PluginException:
            raise PluginException(
                "Signature verification enabled but manifest.json.minisig not found in bundle"
            )

        signature = sig_data.decode("utf-8")

        # Verify
        verifier = MinisignVerifier(public_key)
        if not verifier.verify(manifest_data, signature):
            raise PluginException("Manifest signature verification failed")

    def _verify_library_signature(
        self,
        zip_file: zipfile.ZipFile,
        manifest: BundleManifest,
        library_path: str,
        library_data: bytes,
    ) -> None:
        """Verify the library signature."""
        public_key = self._public_key_override or manifest.public_key

        if not public_key:
            raise PluginException("No public key available for signature verification")

        # Read signature
        sig_path = library_path + ".minisig"
        try:
            sig_data = self._read_zip_entry(zip_file, sig_path)
        except PluginException:
            raise PluginException(
                f"Signature verification enabled but {sig_path} not found in bundle"
            )

        signature = sig_data.decode("utf-8")

        # Verify
        verifier = MinisignVerifier(public_key)
        if not verifier.verify(library_data, signature):
            raise PluginException(
                f"Library signature verification failed: {library_path}"
            )

    @staticmethod
    def _read_zip_entry(zip_file: zipfile.ZipFile, path: str) -> bytes:
        """Read a file from the zip archive."""
        try:
            return zip_file.read(path)
        except KeyError:
            raise PluginException(f"File not found in bundle: {path}")

    @staticmethod
    def _verify_checksum(data: bytes, expected_checksum: str) -> bool:
        """Verify SHA256 checksum."""
        actual_hash = hashlib.sha256(data).hexdigest()

        # Handle both "sha256:xxx" and raw "xxx" formats
        expected = expected_checksum
        if expected.lower().startswith("sha256:"):
            expected = expected[7:]

        return actual_hash.lower() == expected.lower()
