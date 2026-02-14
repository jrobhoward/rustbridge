package com.rustbridge;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for v1.1 variant-level metadata in BundleManifest.
 */
class BundleManifestV11Test {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    @Test
    void fromJson___v11_with_variant_build_info___parses_correctly() throws Exception {
        String json = """
                {
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
                                            "dirty": false
                                        }
                                    },
                                    "sbom": {
                                        "cyclonedx": "sbom/linux-x86_64/sbom.cdx.json"
                                    },
                                    "schema_checksum": "sha256:sch111",
                                    "schemas": {
                                        "messages.h": {
                                            "path": "schemas/linux/messages.h",
                                            "checksum": "sha256:msg111",
                                            "format": "c-header"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        assertEquals("1.1", manifest.bundleVersion);
        assertNotNull(manifest.platforms);

        var platform = manifest.platforms.get("linux-x86_64");
        assertNotNull(platform);
        assertNotNull(platform.variants());

        var variant = platform.variants().get("release");
        assertNotNull(variant);
        assertEquals("lib/linux-x86_64/release/libtest.so", variant.library());
        assertEquals("sha256:rel123", variant.checksum());

        assertNotNull(variant.buildInfo());
        assertEquals("CI/CD-linux", variant.buildInfo().builtBy());
        assertEquals("2025-01-01T00:00:00Z", variant.buildInfo().builtAt());
        assertEquals("x86_64-unknown-linux-gnu", variant.buildInfo().host());
        assertEquals("rustc 1.90.0", variant.buildInfo().compiler());
        assertNotNull(variant.buildInfo().git());
        assertEquals("abc123def", variant.buildInfo().git().commit());
        assertEquals("main", variant.buildInfo().git().branch());
        assertEquals(false, variant.buildInfo().git().dirty());

        assertNotNull(variant.sbom());
        assertEquals("sbom/linux-x86_64/sbom.cdx.json", variant.sbom().cyclonedx());

        assertEquals("sha256:sch111", variant.schemaChecksum());

        assertNotNull(variant.schemas());
        assertEquals(1, variant.schemas().size());
        assertEquals("schemas/linux/messages.h", variant.schemas().get("messages.h").path());
    }

    @Test
    void getEffectiveBuildInfo___variant_has_build_info___returns_variant_build_info() throws Exception {
        String json = """
                {
                    "bundle_version": "1.1",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "build_info": {
                        "built_by": "top-level"
                    },
                    "platforms": {
                        "linux-x86_64": {
                            "library": "",
                            "checksum": "",
                            "variants": {
                                "release": {
                                    "library": "lib/libtest.so",
                                    "checksum": "sha256:abc",
                                    "build_info": {
                                        "built_by": "variant-level"
                                    }
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveBuildInfo("linux-x86_64", "release");

        assertNotNull(result);
        assertEquals("variant-level", result.builtBy());
    }

    @Test
    void getEffectiveBuildInfo___variant_no_build_info___falls_back_to_top_level() throws Exception {
        String json = """
                {
                    "bundle_version": "1.1",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "build_info": {
                        "built_by": "top-level"
                    },
                    "platforms": {
                        "linux-x86_64": {
                            "library": "",
                            "checksum": "",
                            "variants": {
                                "release": {
                                    "library": "lib/libtest.so",
                                    "checksum": "sha256:abc"
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveBuildInfo("linux-x86_64", "release");

        assertNotNull(result);
        assertEquals("top-level", result.builtBy());
    }

    @Test
    void getEffectiveBuildInfo___neither_set___returns_null() throws Exception {
        String json = """
                {
                    "bundle_version": "1.0",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "platforms": {
                        "linux-x86_64": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc"
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveBuildInfo("linux-x86_64", "release");

        assertNull(result);
    }

    @Test
    void getEffectiveSbom___variant_has_sbom___returns_variant_sbom() throws Exception {
        String json = """
                {
                    "bundle_version": "1.1",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "sbom": {
                        "cyclonedx": "sbom/top.cdx.json"
                    },
                    "platforms": {
                        "linux-x86_64": {
                            "library": "",
                            "checksum": "",
                            "variants": {
                                "release": {
                                    "library": "lib/libtest.so",
                                    "checksum": "sha256:abc",
                                    "sbom": {
                                        "cyclonedx": "sbom/variant.cdx.json"
                                    }
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveSbom("linux-x86_64", "release");

        assertNotNull(result);
        assertEquals("sbom/variant.cdx.json", result.cyclonedx());
    }

    @Test
    void getEffectiveSbom___variant_no_sbom___falls_back_to_top_level() throws Exception {
        String json = """
                {
                    "bundle_version": "1.1",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "sbom": {
                        "spdx": "sbom/top.spdx.json"
                    },
                    "platforms": {
                        "linux-x86_64": {
                            "library": "",
                            "checksum": "",
                            "variants": {
                                "release": {
                                    "library": "lib/libtest.so",
                                    "checksum": "sha256:abc"
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveSbom("linux-x86_64", "release");

        assertNotNull(result);
        assertEquals("sbom/top.spdx.json", result.spdx());
    }

    @Test
    void getEffectiveSchemas___variant_has_schemas___returns_variant_schemas() throws Exception {
        String json = """
                {
                    "bundle_version": "1.1",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "schemas": {
                        "top.h": {
                            "path": "schemas/top.h",
                            "checksum": "sha256:top"
                        }
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
                                            "checksum": "sha256:var"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveSchemas("linux-x86_64", "release");

        assertEquals(1, result.size());
        assertTrue(result.containsKey("variant.h"));
        assertEquals("schemas/variant.h", result.get("variant.h").path());
    }

    @Test
    void getEffectiveSchemas___variant_no_schemas___falls_back_to_top_level() throws Exception {
        String json = """
                {
                    "bundle_version": "1.1",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "schemas": {
                        "top.h": {
                            "path": "schemas/top.h",
                            "checksum": "sha256:top"
                        }
                    },
                    "platforms": {
                        "linux-x86_64": {
                            "library": "",
                            "checksum": "",
                            "variants": {
                                "release": {
                                    "library": "lib/libtest.so",
                                    "checksum": "sha256:abc"
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveSchemas("linux-x86_64", "release");

        assertEquals(1, result.size());
        assertTrue(result.containsKey("top.h"));
    }

    @Test
    void fromJson___v10_backward_compat___variant_fields_are_null() throws Exception {
        String json = """
                {
                    "bundle_version": "1.0",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "platforms": {
                        "linux-x86_64": {
                            "library": "lib/libtest.so",
                            "checksum": "sha256:abc",
                            "variants": {
                                "release": {
                                    "library": "lib/libtest.so",
                                    "checksum": "sha256:abc"
                                }
                            }
                        }
                    }
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        assertEquals("1.0", manifest.bundleVersion);

        var variant = manifest.platforms.get("linux-x86_64").variants().get("release");
        assertNotNull(variant);
        assertNull(variant.buildInfo());
        assertNull(variant.sbom());
        assertNull(variant.schemaChecksum());
        assertNull(variant.schemas());
    }

    @Test
    void getEffectiveBuildInfo___unknown_platform___returns_top_level() throws Exception {
        String json = """
                {
                    "bundle_version": "1.0",
                    "plugin": {"name": "test", "version": "1.0.0"},
                    "build_info": {
                        "built_by": "top-level"
                    },
                    "platforms": {}
                }
                """;

        BundleLoader.BundleManifest manifest = MAPPER.readValue(json, BundleLoader.BundleManifest.class);

        var result = manifest.getEffectiveBuildInfo("nonexistent-platform", "release");

        assertNotNull(result);
        assertEquals("top-level", result.builtBy());
    }
}
