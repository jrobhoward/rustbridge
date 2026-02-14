using System.Text.Json;

namespace RustBridge.Tests;

/// <summary>
/// Tests for v1.1 variant-level metadata in <see cref="BundleManifest"/>.
/// </summary>
public class BundleManifestV11Tests
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
    };

    [Fact]
    public void FromJson___V11WithVariantBuildInfo___ParsesCorrectly()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions);

        Assert.NotNull(manifest);
        Assert.Equal("1.1", manifest.BundleVersion);
        Assert.NotNull(manifest.Platforms);

        var platform = manifest.Platforms["linux-x86_64"];
        Assert.NotNull(platform.Variants);

        var variant = platform.Variants["release"];
        Assert.Equal("lib/linux-x86_64/release/libtest.so", variant.Library);
        Assert.Equal("sha256:rel123", variant.Checksum);

        Assert.NotNull(variant.BuildInfoData);
        Assert.Equal("CI/CD-linux", variant.BuildInfoData.BuiltBy);
        Assert.Equal("2025-01-01T00:00:00Z", variant.BuildInfoData.BuiltAt);
        Assert.Equal("x86_64-unknown-linux-gnu", variant.BuildInfoData.Host);
        Assert.Equal("rustc 1.90.0", variant.BuildInfoData.Compiler);
        Assert.NotNull(variant.BuildInfoData.Git);
        Assert.Equal("abc123def", variant.BuildInfoData.Git.Commit);
        Assert.Equal("main", variant.BuildInfoData.Git.Branch);
        Assert.Equal(false, variant.BuildInfoData.Git.Dirty);

        Assert.NotNull(variant.SbomData);
        Assert.Equal("sbom/linux-x86_64/sbom.cdx.json", variant.SbomData.Cyclonedx);

        Assert.Equal("sha256:sch111", variant.SchemaChecksum);

        Assert.NotNull(variant.Schemas);
        Assert.Single(variant.Schemas);
        Assert.Equal("schemas/linux/messages.h", variant.Schemas["messages.h"].Path);
    }

    [Fact]
    public void GetEffectiveBuildInfo___VariantHasBuildInfo___ReturnsVariantBuildInfo()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveBuildInfo("linux-x86_64", "release");

        Assert.NotNull(result);
        Assert.Equal("variant-level", result.BuiltBy);
    }

    [Fact]
    public void GetEffectiveBuildInfo___VariantNoBuildInfo___FallsBackToTopLevel()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveBuildInfo("linux-x86_64", "release");

        Assert.NotNull(result);
        Assert.Equal("top-level", result.BuiltBy);
    }

    [Fact]
    public void GetEffectiveBuildInfo___NeitherSet___ReturnsNull()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveBuildInfo("linux-x86_64", "release");

        Assert.Null(result);
    }

    [Fact]
    public void GetEffectiveSbom___VariantHasSbom___ReturnsVariantSbom()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveSbom("linux-x86_64", "release");

        Assert.NotNull(result);
        Assert.Equal("sbom/variant.cdx.json", result.Cyclonedx);
    }

    [Fact]
    public void GetEffectiveSbom___VariantNoSbom___FallsBackToTopLevel()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveSbom("linux-x86_64", "release");

        Assert.NotNull(result);
        Assert.Equal("sbom/top.spdx.json", result.Spdx);
    }

    [Fact]
    public void GetEffectiveSchemas___VariantHasSchemas___ReturnsVariantSchemas()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveSchemas("linux-x86_64", "release");

        Assert.Single(result);
        Assert.True(result.ContainsKey("variant.h"));
        Assert.Equal("schemas/variant.h", result["variant.h"].Path);
    }

    [Fact]
    public void GetEffectiveSchemas___VariantNoSchemas___FallsBackToTopLevel()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveSchemas("linux-x86_64", "release");

        Assert.Single(result);
        Assert.True(result.ContainsKey("top.h"));
    }

    [Fact]
    public void FromJson___V10BackwardCompat___VariantFieldsAreNull()
    {
        var json = """
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

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        Assert.Equal("1.0", manifest.BundleVersion);

        var variant = manifest.Platforms!["linux-x86_64"].Variants!["release"];
        Assert.Null(variant.BuildInfoData);
        Assert.Null(variant.SbomData);
        Assert.Null(variant.SchemaChecksum);
        Assert.Null(variant.Schemas);
    }

    [Fact]
    public void GetEffectiveBuildInfo___UnknownPlatform___ReturnsTopLevel()
    {
        var json = """
            {
                "bundle_version": "1.0",
                "plugin": {"name": "test", "version": "1.0.0"},
                "build_info": {
                    "built_by": "top-level"
                },
                "platforms": {}
            }
            """;

        var manifest = JsonSerializer.Deserialize<BundleManifest>(json, JsonOptions)!;

        var result = manifest.GetEffectiveBuildInfo("nonexistent-platform", "release");

        Assert.NotNull(result);
        Assert.Equal("top-level", result.BuiltBy);
    }
}
