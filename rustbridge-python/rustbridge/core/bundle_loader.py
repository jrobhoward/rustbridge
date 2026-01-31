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

from rustbridge.core.bundle_manifest import BundleManifest, BridgeInfo, BuildInfo, SchemaInfo
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

        # Extract library to unique temp directory
        temp_dir = tempfile.mkdtemp(prefix="rustbridge-", dir=tempfile.gettempdir())
        try:
            lib_path = self._extract_library_internal(
                bundle_path, Path(temp_dir), fail_if_exists=False
            )
            return NativePluginLoader.load_with_config(
                str(lib_path), config, log_callback
            )
        except Exception:
            # Clean up temp directory on failure
            import shutil

            shutil.rmtree(temp_dir, ignore_errors=True)
            raise

    def extract_library_to_temp(self, bundle_path: str | Path) -> Path:
        """
        Extract and verify library from bundle to a unique temporary directory.

        The library is extracted to a unique subdirectory under the system temp
        directory, ensuring no conflicts with other extractions. The caller is
        responsible for cleaning up the temporary directory when done.

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            Path to the extracted library file.

        Raises:
            PluginException: If extraction or verification fails.
        """
        bundle_path = Path(bundle_path)
        if not bundle_path.exists():
            raise FileNotFoundError(f"Bundle not found: {bundle_path}")

        # Create unique temp directory under system temp path
        temp_dir = tempfile.mkdtemp(prefix="rustbridge-", dir=tempfile.gettempdir())
        return self._extract_library_internal(
            bundle_path, Path(temp_dir), fail_if_exists=False
        )

    def extract_library(self, bundle_path: str | Path, dest_dir: str | Path) -> Path:
        """
        Extract and verify library from bundle to the specified directory.

        This method will fail if the library file already exists at the target path.
        This prevents accidental overwrites and ensures the caller has explicit
        control over file lifecycle.

        Args:
            bundle_path: Path to the .rbp bundle file.
            dest_dir: Directory to extract the library to.

        Returns:
            Path to the extracted library file.

        Raises:
            PluginException: If extraction or verification fails.
            FileExistsError: If the library file already exists at the target path.
        """
        bundle_path = Path(bundle_path)
        dest_dir = Path(dest_dir)

        if not bundle_path.exists():
            raise FileNotFoundError(f"Bundle not found: {bundle_path}")

        return self._extract_library_internal(bundle_path, dest_dir, fail_if_exists=True)

    def _extract_library_internal(
        self,
        bundle_path: Path,
        dest_dir: Path,
        *,
        fail_if_exists: bool,
        variant: str | None = None,
    ) -> Path:
        """
        Internal method to extract the library with configurable overwrite behavior.

        Args:
            bundle_path: Path to the .rbp bundle file.
            dest_dir: Directory to extract the library to.
            fail_if_exists: If True, raise FileExistsError if file already exists.
            variant: Variant to extract (defaults to platform's default variant).

        Returns:
            Path to the extracted library file.
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

            # Get effective variant
            effective_variant = variant or platform_info.get_default_variant()

            # Get library path and checksum for the variant
            library_path = platform_info.get_library(effective_variant)
            checksum = platform_info.get_checksum(effective_variant)

            if not library_path:
                raise PluginException(
                    f"Variant '{effective_variant}' not found for platform '{current_platform}'"
                )

            # Read library data
            lib_data = self._read_zip_entry(zip_file, library_path)

            # Verify checksum
            if not self._verify_checksum(lib_data, checksum):
                raise PluginException(f"Checksum verification failed for {library_path}")

            # Verify library signature if enabled
            if self._verify_signatures:
                self._verify_library_signature(zip_file, manifest, library_path, lib_data)

            # Determine output path
            lib_filename = Path(library_path).name
            output_path = dest_dir / lib_filename

            # Check if file already exists when user specifies path
            if fail_if_exists and output_path.exists():
                raise FileExistsError(
                    f"Library already exists at target path: {output_path}. "
                    "Remove the existing file or use extract_library_to_temp() "
                    "for automatic temp directory."
                )

            # Ensure output directory exists
            dest_dir.mkdir(parents=True, exist_ok=True)

            # Write the library
            output_path.write_bytes(lib_data)

            # Make executable on Unix
            if os.name != "nt":
                current_mode = output_path.stat().st_mode
                output_path.chmod(
                    current_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH
                )

            return output_path

    def extract_library_variant(
        self,
        bundle_path: str | Path,
        dest_dir: str | Path,
        variant: str,
    ) -> Path:
        """
        Extract a specific variant of the library from bundle to the specified directory.

        This method will fail if the library file already exists at the target path.

        Args:
            bundle_path: Path to the .rbp bundle file.
            dest_dir: Directory to extract the library to.
            variant: Variant name (e.g., "release", "debug").

        Returns:
            Path to the extracted library file.

        Raises:
            PluginException: If extraction or verification fails, or variant not found.
            FileExistsError: If the library file already exists at the target path.
        """
        bundle_path = Path(bundle_path)
        dest_dir = Path(dest_dir)

        if not bundle_path.exists():
            raise FileNotFoundError(f"Bundle not found: {bundle_path}")

        return self._extract_library_internal(
            bundle_path, dest_dir, fail_if_exists=True, variant=variant
        )

    def list_variants(self, bundle_path: str | Path, platform: str | None = None) -> list[str]:
        """
        List available variants for a platform.

        Args:
            bundle_path: Path to the .rbp bundle file.
            platform: Platform string (e.g., "linux-x86_64"). Defaults to current platform.

        Returns:
            List of available variant names.

        Raises:
            PluginException: If platform is not supported.
        """
        manifest = self.get_manifest(bundle_path)
        platform = platform or self.get_current_platform()
        platform_info = manifest.get_platform(platform)
        if not platform_info:
            raise PluginException(f"Platform not supported: {platform}")
        return platform_info.list_variants()

    def get_default_variant(self, bundle_path: str | Path, platform: str | None = None) -> str:
        """
        Get the default variant for a platform.

        Args:
            bundle_path: Path to the .rbp bundle file.
            platform: Platform string (e.g., "linux-x86_64"). Defaults to current platform.

        Returns:
            Default variant name (typically "release").
        """
        manifest = self.get_manifest(bundle_path)
        platform = platform or self.get_current_platform()
        platform_info = manifest.get_platform(platform)
        if not platform_info:
            return "release"
        return platform_info.get_default_variant()

    def get_build_info(self, bundle_path: str | Path) -> BuildInfo | None:
        """
        Get build info from the manifest (v2.0+ bundles only).

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            BuildInfo if present, None otherwise.
        """
        manifest = self.get_manifest(bundle_path)
        return manifest.build_info

    def has_jni_bridge(self, bundle_path: str | Path) -> bool:
        """
        Check if the bundle includes a JNI bridge library.

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            True if the bundle contains at least one JNI bridge library.
        """
        manifest = self.get_manifest(bundle_path)
        return (
            manifest.bridges is not None
            and manifest.bridges.jni is not None
            and len(manifest.bridges.jni) > 0
        )

    def extract_jni_bridge_to_temp(self, bundle_path: str | Path) -> Path:
        """
        Extract and verify JNI bridge library from bundle to a unique temp directory.

        The library is extracted to a unique subdirectory under the system temp
        directory, ensuring no conflicts with other extractions. The caller is
        responsible for cleaning up the temporary directory when done.

        Args:
            bundle_path: Path to the .rbp bundle file.

        Returns:
            Path to the extracted library file.

        Raises:
            PluginException: If extraction or verification fails.
        """
        bundle_path = Path(bundle_path)
        if not bundle_path.exists():
            raise FileNotFoundError(f"Bundle not found: {bundle_path}")

        # Create unique temp directory under system temp path
        temp_dir = tempfile.mkdtemp(prefix="rustbridge-", dir=tempfile.gettempdir())
        return self._extract_jni_bridge_internal(
            bundle_path, Path(temp_dir), fail_if_exists=False
        )

    def extract_jni_bridge(self, bundle_path: str | Path, dest_dir: str | Path) -> Path:
        """
        Extract and verify JNI bridge library from bundle to the specified directory.

        This method will fail if the library file already exists at the target path.

        Args:
            bundle_path: Path to the .rbp bundle file.
            dest_dir: Directory to extract the library to.

        Returns:
            Path to the extracted library file.

        Raises:
            PluginException: If extraction or verification fails.
            FileExistsError: If the library file already exists at the target path.
        """
        bundle_path = Path(bundle_path)
        dest_dir = Path(dest_dir)

        if not bundle_path.exists():
            raise FileNotFoundError(f"Bundle not found: {bundle_path}")

        return self._extract_jni_bridge_internal(bundle_path, dest_dir, fail_if_exists=True)

    def _extract_jni_bridge_internal(
        self,
        bundle_path: Path,
        dest_dir: Path,
        *,
        fail_if_exists: bool,
        variant: str | None = None,
    ) -> Path:
        """
        Internal method to extract the JNI bridge library.

        Args:
            bundle_path: Path to the .rbp bundle file.
            dest_dir: Directory to extract the library to.
            fail_if_exists: If True, raise FileExistsError if file already exists.
            variant: Variant to extract (defaults to platform's default variant).

        Returns:
            Path to the extracted library file.
        """
        with zipfile.ZipFile(bundle_path, "r") as zip_file:
            # Load manifest
            manifest = self._load_manifest(zip_file)

            # Check if JNI bridge is available
            if manifest.bridges is None or not manifest.bridges.jni:
                raise PluginException("Bundle does not contain a JNI bridge library")

            # Verify manifest signature if enabled
            if self._verify_signatures:
                self._verify_manifest_signature(zip_file, manifest)

            # Detect platform
            current_platform = self.get_current_platform()
            platform_info = manifest.bridges.jni.get(current_platform)
            if not platform_info:
                raise PluginException(
                    f"JNI bridge not available for platform: {current_platform}"
                )

            # Get effective variant
            effective_variant = variant or platform_info.get_default_variant()

            # Get library path and checksum for the variant
            library_path = platform_info.get_library(effective_variant)
            checksum = platform_info.get_checksum(effective_variant)

            if not library_path:
                raise PluginException(
                    f"JNI bridge variant '{effective_variant}' not found for platform '{current_platform}'"
                )

            # Read library data
            lib_data = self._read_zip_entry(zip_file, library_path)

            # Verify checksum
            if not self._verify_checksum(lib_data, checksum):
                raise PluginException(f"Checksum verification failed for JNI bridge: {library_path}")

            # Verify library signature if enabled
            if self._verify_signatures:
                self._verify_library_signature(zip_file, manifest, library_path, lib_data)

            # Determine output path
            lib_filename = Path(library_path).name
            output_path = dest_dir / lib_filename

            # Check if file already exists when user specifies path
            if fail_if_exists and output_path.exists():
                raise FileExistsError(
                    f"JNI bridge already exists at target path: {output_path}. "
                    "Remove the existing file or use extract_jni_bridge_to_temp() "
                    "for automatic temp directory."
                )

            # Ensure output directory exists
            dest_dir.mkdir(parents=True, exist_ok=True)

            # Write the library
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
