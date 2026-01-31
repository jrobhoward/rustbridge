"""Bundle manifest parsing for .rbp files."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Any

from rustbridge.core.plugin_exception import PluginException


@dataclass
class VariantInfo:
    """Variant-specific library information."""

    library: str
    """Path to the library file within the bundle."""

    checksum: str
    """SHA256 checksum of the library."""

    build: Any | None = None
    """Optional build metadata (profile, opt_level, features, etc.)."""


@dataclass
class PlatformInfo:
    """Platform-specific library information with variant support."""

    library: str
    """Path to the library file within the bundle (backward compat / default variant)."""

    checksum: str
    """SHA256 checksum of the library (backward compat / default variant)."""

    default_variant: str | None = None
    """The default variant name (usually "release")."""

    variants: dict[str, VariantInfo] = field(default_factory=dict)
    """Map of variant name to VariantInfo (v2.0+)."""

    def get_library(self, variant: str) -> str:
        """Get the effective library path for a variant."""
        if self.variants and variant in self.variants:
            return self.variants[variant].library
        return self.library

    def get_checksum(self, variant: str) -> str:
        """Get the effective checksum for a variant."""
        if self.variants and variant in self.variants:
            return self.variants[variant].checksum
        return self.checksum

    def get_default_variant(self) -> str:
        """Get the default variant name."""
        return self.default_variant if self.default_variant else "release"

    def list_variants(self) -> list[str]:
        """List available variants for this platform."""
        if not self.variants:
            return ["release"]
        return list(self.variants.keys())


@dataclass
class PluginInfo:
    """Plugin metadata information."""

    name: str
    version: str
    description: str | None = None
    authors: list[str] = field(default_factory=list)
    license: str | None = None
    repository: str | None = None


@dataclass
class SchemaInfo:
    """Schema file information."""

    path: str
    """Path to the schema file within the bundle."""

    checksum: str
    """SHA256 checksum of the schema file."""

    format: str | None = None
    """Schema format (e.g., "json-schema", "c-header")."""

    description: str | None = None
    """Schema description."""


@dataclass
class GitInfo:
    """Git repository information."""

    commit: str | None = None
    """Full commit hash (required if git section present)."""

    branch: str | None = None
    """Branch name."""

    tag: str | None = None
    """Tag name (if on a tagged commit)."""

    dirty: bool | None = None
    """Whether working directory had uncommitted changes."""


@dataclass
class BuildInfo:
    """Build metadata information."""

    built_by: str | None = None
    """Who/what built the bundle (e.g., "CI/CD", "developer")."""

    built_at: str | None = None
    """ISO 8601 timestamp."""

    host: str | None = None
    """Host triple (e.g., "x86_64-unknown-linux-gnu")."""

    compiler: str | None = None
    """Compiler version (e.g., "rustc 1.90.0")."""

    rustbridge_version: str | None = None
    """rustbridge CLI version."""

    git: GitInfo | None = None
    """Git repository info."""


@dataclass
class Sbom:
    """Software Bill of Materials (SBOM) paths."""

    cyclonedx: str | None = None
    """Path to CycloneDX SBOM file (e.g., "sbom/sbom.cdx.json")."""

    spdx: str | None = None
    """Path to SPDX SBOM file (e.g., "sbom/sbom.spdx.json")."""


@dataclass
class BridgeInfo:
    """Bridge libraries bundled with the plugin.

    Allows bundling bridge libraries (like the JNI bridge) alongside
    the plugin for self-contained distribution.
    """

    jni: dict[str, PlatformInfo] = field(default_factory=dict)
    """JNI bridge libraries by platform."""


@dataclass
class BundleManifest:
    """
    Bundle manifest structure.

    Parsed from manifest.json in .rbp bundle files.
    """

    bundle_version: str
    """Bundle format version."""

    plugin_name: str
    """Plugin name."""

    plugin_version: str
    """Plugin version."""

    platforms: dict[str, PlatformInfo]
    """Platform-specific libraries."""

    public_key: str | None = None
    """Minisign public key (base64)."""

    plugin_info: PluginInfo | None = None
    """Full plugin metadata."""

    schemas: dict[str, SchemaInfo] = field(default_factory=dict)
    """Schema files in the bundle."""

    build_info: BuildInfo | None = None
    """Build metadata (v2.0+)."""

    sbom: Sbom | None = None
    """SBOM information (v2.0+)."""

    schema_checksum: str | None = None
    """Combined schema checksum for validation (v2.0+)."""

    notices: str | None = None
    """Path to license notices file in bundle (v2.0+)."""

    bridges: BridgeInfo | None = None
    """Bridge libraries bundled with the plugin (e.g., JNI bridge)."""

    @classmethod
    def from_json(cls, json_str: str) -> BundleManifest:
        """
        Parse a BundleManifest from JSON string.

        Args:
            json_str: The JSON string.

        Returns:
            The parsed BundleManifest.

        Raises:
            PluginException: If parsing fails.
        """
        try:
            data = json.loads(json_str)
        except json.JSONDecodeError as e:
            raise PluginException(f"Failed to parse manifest JSON: {e}") from e

        return cls.from_dict(data)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> BundleManifest:
        """
        Create a BundleManifest from a dictionary.

        Args:
            data: The manifest data as a dictionary.

        Returns:
            The parsed BundleManifest.

        Raises:
            PluginException: If required fields are missing.
        """
        bundle_version = data.get("bundle_version")
        if not bundle_version:
            raise PluginException("Missing required field: bundle_version")

        plugin_data = data.get("plugin", {})
        plugin_name = plugin_data.get("name")
        plugin_version = plugin_data.get("version")

        if not plugin_name:
            raise PluginException("Missing required field: plugin.name")
        if not plugin_version:
            raise PluginException("Missing required field: plugin.version")

        # Parse platforms
        platforms_data = data.get("platforms", {})
        platforms: dict[str, PlatformInfo] = {}
        for platform_key, platform_value in platforms_data.items():
            # Parse variants if present (v2.0+)
            variants: dict[str, VariantInfo] = {}
            variants_data = platform_value.get("variants", {})
            for variant_name, variant_value in variants_data.items():
                variants[variant_name] = VariantInfo(
                    library=variant_value.get("library", ""),
                    checksum=variant_value.get("checksum", ""),
                    build=variant_value.get("build"),
                )

            platforms[platform_key] = PlatformInfo(
                library=platform_value.get("library", ""),
                checksum=platform_value.get("checksum", ""),
                default_variant=platform_value.get("default_variant"),
                variants=variants,
            )

        # Parse plugin info
        plugin_info = None
        if plugin_data:
            plugin_info = PluginInfo(
                name=plugin_name,
                version=plugin_version,
                description=plugin_data.get("description"),
                authors=plugin_data.get("authors", []),
                license=plugin_data.get("license"),
                repository=plugin_data.get("repository"),
            )

        # Parse schemas
        schemas: dict[str, SchemaInfo] = {}
        schemas_data = data.get("schemas", {})
        for schema_name, schema_value in schemas_data.items():
            schemas[schema_name] = SchemaInfo(
                path=schema_value.get("path", ""),
                checksum=schema_value.get("checksum", ""),
                format=schema_value.get("format"),
                description=schema_value.get("description"),
            )

        # Parse build info
        build_info: BuildInfo | None = None
        build_info_data = data.get("build_info")
        if build_info_data:
            git_info: GitInfo | None = None
            git_data = build_info_data.get("git")
            if git_data:
                git_info = GitInfo(
                    commit=git_data.get("commit"),
                    branch=git_data.get("branch"),
                    tag=git_data.get("tag"),
                    dirty=git_data.get("dirty"),
                )
            build_info = BuildInfo(
                built_by=build_info_data.get("built_by"),
                built_at=build_info_data.get("built_at"),
                host=build_info_data.get("host"),
                compiler=build_info_data.get("compiler"),
                rustbridge_version=build_info_data.get("rustbridge_version"),
                git=git_info,
            )

        # Parse SBOM
        sbom: Sbom | None = None
        sbom_data = data.get("sbom")
        if sbom_data:
            sbom = Sbom(
                cyclonedx=sbom_data.get("cyclonedx"),
                spdx=sbom_data.get("spdx"),
            )

        # Parse bridges
        bridges: BridgeInfo | None = None
        bridges_data = data.get("bridges")
        if bridges_data:
            jni_platforms: dict[str, PlatformInfo] = {}
            jni_data = bridges_data.get("jni", {})
            for platform_key, platform_value in jni_data.items():
                # Parse variants if present
                variants: dict[str, VariantInfo] = {}
                variants_data = platform_value.get("variants", {})
                for variant_name, variant_value in variants_data.items():
                    variants[variant_name] = VariantInfo(
                        library=variant_value.get("library", ""),
                        checksum=variant_value.get("checksum", ""),
                        build=variant_value.get("build"),
                    )

                jni_platforms[platform_key] = PlatformInfo(
                    library=platform_value.get("library", ""),
                    checksum=platform_value.get("checksum", ""),
                    default_variant=platform_value.get("default_variant"),
                    variants=variants,
                )
            bridges = BridgeInfo(jni=jni_platforms)

        return cls(
            bundle_version=bundle_version,
            plugin_name=plugin_name,
            plugin_version=plugin_version,
            platforms=platforms,
            public_key=data.get("public_key"),
            plugin_info=plugin_info,
            schemas=schemas,
            build_info=build_info,
            sbom=sbom,
            schema_checksum=data.get("schema_checksum"),
            notices=data.get("notices"),
            bridges=bridges,
        )

    def get_platform(self, platform: str) -> PlatformInfo | None:
        """
        Get platform info for a specific platform.

        Args:
            platform: Platform string (e.g., "linux-x86_64").

        Returns:
            PlatformInfo if found, None otherwise.
        """
        return self.platforms.get(platform)
