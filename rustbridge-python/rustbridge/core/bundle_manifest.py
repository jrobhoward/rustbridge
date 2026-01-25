"""Bundle manifest parsing for .rbp files."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Any

from rustbridge.core.plugin_exception import PluginException


@dataclass
class PlatformInfo:
    """Platform-specific library information."""

    library: str
    """Path to the library file within the bundle."""

    checksum: str
    """SHA256 checksum of the library."""


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
class MessageInfo:
    """Message type information."""

    type_tag: str | None = None
    description: str | None = None
    request_schema: str | None = None
    response_schema: str | None = None
    message_id: int | None = None
    cstruct_request: str | None = None
    cstruct_response: str | None = None


@dataclass
class ApiInfo:
    """API information for the plugin."""

    min_rustbridge_version: str | None = None
    transports: list[str] = field(default_factory=list)
    messages: list[MessageInfo] = field(default_factory=list)


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

    api: ApiInfo | None = None
    """API information."""

    schemas: dict[str, SchemaInfo] = field(default_factory=dict)
    """Schema files in the bundle."""

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
            platforms[platform_key] = PlatformInfo(
                library=platform_value.get("library", ""),
                checksum=platform_value.get("checksum", ""),
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

        # Parse API info
        api = None
        api_data = data.get("api")
        if api_data:
            messages = []
            for msg_data in api_data.get("messages", []):
                messages.append(
                    MessageInfo(
                        type_tag=msg_data.get("type_tag"),
                        description=msg_data.get("description"),
                        request_schema=msg_data.get("request_schema"),
                        response_schema=msg_data.get("response_schema"),
                        message_id=msg_data.get("message_id"),
                        cstruct_request=msg_data.get("cstruct_request"),
                        cstruct_response=msg_data.get("cstruct_response"),
                    )
                )
            api = ApiInfo(
                min_rustbridge_version=api_data.get("min_rustbridge_version"),
                transports=api_data.get("transports", []),
                messages=messages,
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

        return cls(
            bundle_version=bundle_version,
            plugin_name=plugin_name,
            plugin_version=plugin_version,
            platforms=platforms,
            public_key=data.get("public_key"),
            plugin_info=plugin_info,
            api=api,
            schemas=schemas,
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
