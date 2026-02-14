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

    build_info: BuildInfo | None = None
    """Variant-level build info (v1.1, overrides top-level)."""

    sbom: Sbom | None = None
    """Variant-level SBOM paths (v1.1, overrides top-level)."""

    schema_checksum: str | None = None
    """Variant-level schema checksum (v1.1, overrides top-level)."""

    schemas: dict[str, SchemaInfo] = field(default_factory=dict)
    """Variant-level schemas (v1.1, overrides top-level)."""


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

    @staticmethod
    def _parse_build_info(data: dict[str, Any] | None) -> BuildInfo | None:
        """Parse a BuildInfo from a dictionary."""
        if not data:
            return None
        git_info: GitInfo | None = None
        git_data = data.get("git")
        if git_data:
            git_info = GitInfo(
                commit=git_data.get("commit"),
                branch=git_data.get("branch"),
                tag=git_data.get("tag"),
                dirty=git_data.get("dirty"),
            )
        return BuildInfo(
            built_by=data.get("built_by"),
            built_at=data.get("built_at"),
            host=data.get("host"),
            compiler=data.get("compiler"),
            rustbridge_version=data.get("rustbridge_version"),
            git=git_info,
        )

    @staticmethod
    def _parse_sbom(data: dict[str, Any] | None) -> Sbom | None:
        """Parse an Sbom from a dictionary."""
        if not data:
            return None
        return Sbom(
            cyclonedx=data.get("cyclonedx"),
            spdx=data.get("spdx"),
        )

    @staticmethod
    def _parse_schemas(data: dict[str, Any] | None) -> dict[str, SchemaInfo]:
        """Parse schemas from a dictionary."""
        if not data:
            return {}
        schemas: dict[str, SchemaInfo] = {}
        for schema_name, schema_value in data.items():
            schemas[schema_name] = SchemaInfo(
                path=schema_value.get("path", ""),
                checksum=schema_value.get("checksum", ""),
                format=schema_value.get("format"),
                description=schema_value.get("description"),
            )
        return schemas

    @classmethod
    def _parse_variant(cls, variant_value: dict[str, Any]) -> VariantInfo:
        """Parse a VariantInfo from a dictionary, including v1.1 fields."""
        return VariantInfo(
            library=variant_value.get("library", ""),
            checksum=variant_value.get("checksum", ""),
            build=variant_value.get("build"),
            build_info=cls._parse_build_info(variant_value.get("build_info")),
            sbom=cls._parse_sbom(variant_value.get("sbom")),
            schema_checksum=variant_value.get("schema_checksum"),
            schemas=cls._parse_schemas(variant_value.get("schemas")),
        )

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
            # Parse variants if present (v2.0+), including v1.1 fields
            variants: dict[str, VariantInfo] = {}
            variants_data = platform_value.get("variants", {})
            for variant_name, variant_value in variants_data.items():
                variants[variant_name] = cls._parse_variant(variant_value)

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
        schemas = cls._parse_schemas(data.get("schemas"))

        # Parse build info
        build_info = cls._parse_build_info(data.get("build_info"))

        # Parse SBOM
        sbom = cls._parse_sbom(data.get("sbom"))

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
                    variants[variant_name] = cls._parse_variant(variant_value)

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

    def get_effective_build_info(
        self, platform: str, variant: str = "release"
    ) -> BuildInfo | None:
        """Get effective build info for a platform/variant (v1.1).

        Variant-level overrides top-level.

        Args:
            platform: Platform string (e.g., "linux-x86_64").
            variant: Variant name (e.g., "release").

        Returns:
            Effective BuildInfo, or None if neither set.
        """
        pi = self.platforms.get(platform)
        if pi and pi.variants:
            vi = pi.variants.get(variant)
            if vi and vi.build_info is not None:
                return vi.build_info
        return self.build_info

    def get_effective_sbom(
        self, platform: str, variant: str = "release"
    ) -> Sbom | None:
        """Get effective SBOM for a platform/variant (v1.1).

        Variant-level overrides top-level.

        Args:
            platform: Platform string (e.g., "linux-x86_64").
            variant: Variant name (e.g., "release").

        Returns:
            Effective Sbom, or None if neither set.
        """
        pi = self.platforms.get(platform)
        if pi and pi.variants:
            vi = pi.variants.get(variant)
            if vi and vi.sbom is not None:
                return vi.sbom
        return self.sbom

    def get_effective_schemas(
        self, platform: str, variant: str = "release"
    ) -> dict[str, SchemaInfo]:
        """Get effective schemas for a platform/variant (v1.1).

        Variant-level overrides top-level.

        Args:
            platform: Platform string (e.g., "linux-x86_64").
            variant: Variant name (e.g., "release").

        Returns:
            Effective schemas dictionary.
        """
        pi = self.platforms.get(platform)
        if pi and pi.variants:
            vi = pi.variants.get(variant)
            if vi and vi.schemas:
                return vi.schemas
        return self.schemas

    def get_platform(self, platform: str) -> PlatformInfo | None:
        """
        Get platform info for a specific platform.

        Args:
            platform: Platform string (e.g., "linux-x86_64").

        Returns:
            PlatformInfo if found, None otherwise.
        """
        return self.platforms.get(platform)
