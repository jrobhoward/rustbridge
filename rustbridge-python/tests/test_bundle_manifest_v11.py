"""Tests for v1.1 variant-level metadata in BundleManifest."""

import json

from rustbridge import BundleManifest
from rustbridge.core.bundle_manifest import BuildInfo, Sbom, VariantInfo


class TestBundleManifestV11:
    """Tests for v1.1 variant-level metadata parsing and resolution."""

    def test_from_json___v11_with_variant_build_info___parses_correctly(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.1",
            "plugin": {"name": "test", "version": "1.0.0"},
            "platforms": {
                "linux-x86_64": {
                    "library": "lib/linux-x86_64/libtest.so",
                    "checksum": "sha256:abc123",
                    "default_variant": "release",
                    "variants": {
                        "release": {
                            "library": "lib/linux-x86_64/release/libtest.so",
                            "checksum": "sha256:rel123",
                            "build_info": {
                                "built_by": "CI/CD-linux",
                                "built_at": "2025-01-01T00:00:00Z",
                                "host": "x86_64-unknown-linux-gnu",
                                "compiler": "rustc 1.90.0",
                                "git": {
                                    "commit": "abc123def",
                                    "branch": "main",
                                    "dirty": False,
                                },
                            },
                            "sbom": {
                                "cyclonedx": "sbom/linux-x86_64/sbom.cdx.json",
                            },
                            "schema_checksum": "sha256:sch111",
                            "schemas": {
                                "messages.h": {
                                    "path": "schemas/linux/messages.h",
                                    "checksum": "sha256:msg111",
                                    "format": "c-header",
                                },
                            },
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        assert manifest.bundle_version == "1.1"
        assert "linux-x86_64" in manifest.platforms

        platform = manifest.platforms["linux-x86_64"]
        assert "release" in platform.variants

        variant = platform.variants["release"]
        assert variant.library == "lib/linux-x86_64/release/libtest.so"
        assert variant.checksum == "sha256:rel123"

        assert variant.build_info is not None
        assert variant.build_info.built_by == "CI/CD-linux"
        assert variant.build_info.built_at == "2025-01-01T00:00:00Z"
        assert variant.build_info.host == "x86_64-unknown-linux-gnu"
        assert variant.build_info.compiler == "rustc 1.90.0"
        assert variant.build_info.git is not None
        assert variant.build_info.git.commit == "abc123def"
        assert variant.build_info.git.branch == "main"
        assert variant.build_info.git.dirty is False

        assert variant.sbom is not None
        assert variant.sbom.cyclonedx == "sbom/linux-x86_64/sbom.cdx.json"

        assert variant.schema_checksum == "sha256:sch111"

        assert len(variant.schemas) == 1
        assert variant.schemas["messages.h"].path == "schemas/linux/messages.h"

    def test_get_effective_build_info___variant_has_build_info___returns_variant(
        self,
    ) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.1",
            "plugin": {"name": "test", "version": "1.0.0"},
            "build_info": {"built_by": "top-level"},
            "platforms": {
                "linux-x86_64": {
                    "library": "",
                    "checksum": "",
                    "variants": {
                        "release": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                            "build_info": {"built_by": "variant-level"},
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_build_info("linux-x86_64", "release")

        assert result is not None
        assert result.built_by == "variant-level"

    def test_get_effective_build_info___variant_no_build_info___falls_back(
        self,
    ) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.1",
            "plugin": {"name": "test", "version": "1.0.0"},
            "build_info": {"built_by": "top-level"},
            "platforms": {
                "linux-x86_64": {
                    "library": "",
                    "checksum": "",
                    "variants": {
                        "release": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_build_info("linux-x86_64", "release")

        assert result is not None
        assert result.built_by == "top-level"

    def test_get_effective_build_info___neither_set___returns_none(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"name": "test", "version": "1.0.0"},
            "platforms": {
                "linux-x86_64": {
                    "library": "lib/libtest.so",
                    "checksum": "sha256:abc",
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_build_info("linux-x86_64", "release")

        assert result is None

    def test_get_effective_sbom___variant_has_sbom___returns_variant(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.1",
            "plugin": {"name": "test", "version": "1.0.0"},
            "sbom": {"cyclonedx": "sbom/top.cdx.json"},
            "platforms": {
                "linux-x86_64": {
                    "library": "",
                    "checksum": "",
                    "variants": {
                        "release": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                            "sbom": {"cyclonedx": "sbom/variant.cdx.json"},
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_sbom("linux-x86_64", "release")

        assert result is not None
        assert result.cyclonedx == "sbom/variant.cdx.json"

    def test_get_effective_sbom___variant_no_sbom___falls_back(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.1",
            "plugin": {"name": "test", "version": "1.0.0"},
            "sbom": {"spdx": "sbom/top.spdx.json"},
            "platforms": {
                "linux-x86_64": {
                    "library": "",
                    "checksum": "",
                    "variants": {
                        "release": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_sbom("linux-x86_64", "release")

        assert result is not None
        assert result.spdx == "sbom/top.spdx.json"

    def test_get_effective_schemas___variant_has_schemas___returns_variant(
        self,
    ) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.1",
            "plugin": {"name": "test", "version": "1.0.0"},
            "schemas": {
                "top.h": {"path": "schemas/top.h", "checksum": "sha256:top"},
            },
            "platforms": {
                "linux-x86_64": {
                    "library": "",
                    "checksum": "",
                    "variants": {
                        "release": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                            "schemas": {
                                "variant.h": {
                                    "path": "schemas/variant.h",
                                    "checksum": "sha256:var",
                                },
                            },
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_schemas("linux-x86_64", "release")

        assert len(result) == 1
        assert "variant.h" in result
        assert result["variant.h"].path == "schemas/variant.h"

    def test_get_effective_schemas___variant_no_schemas___falls_back(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.1",
            "plugin": {"name": "test", "version": "1.0.0"},
            "schemas": {
                "top.h": {"path": "schemas/top.h", "checksum": "sha256:top"},
            },
            "platforms": {
                "linux-x86_64": {
                    "library": "",
                    "checksum": "",
                    "variants": {
                        "release": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_schemas("linux-x86_64", "release")

        assert len(result) == 1
        assert "top.h" in result

    def test_from_json___v10_backward_compat___variant_fields_are_none(self) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"name": "test", "version": "1.0.0"},
            "platforms": {
                "linux-x86_64": {
                    "library": "lib/libtest.so",
                    "checksum": "sha256:abc",
                    "variants": {
                        "release": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                        },
                    },
                },
            },
        })

        manifest = BundleManifest.from_json(manifest_json)

        assert manifest.bundle_version == "1.0"

        variant = manifest.platforms["linux-x86_64"].variants["release"]
        assert variant.build_info is None
        assert variant.sbom is None
        assert variant.schema_checksum is None
        assert variant.schemas == {}

    def test_get_effective_build_info___unknown_platform___returns_top_level(
        self,
    ) -> None:
        manifest_json = json.dumps({
            "bundle_version": "1.0",
            "plugin": {"name": "test", "version": "1.0.0"},
            "build_info": {"built_by": "top-level"},
            "platforms": {},
        })

        manifest = BundleManifest.from_json(manifest_json)

        result = manifest.get_effective_build_info("nonexistent-platform", "release")

        assert result is not None
        assert result.built_by == "top-level"
