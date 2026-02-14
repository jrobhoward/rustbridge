using System.Text.Json.Serialization;

namespace RustBridge;

/// <summary>
/// Bundle manifest structure.
/// </summary>
public class BundleManifest
{
    /// <summary>
    /// Bundle format version.
    /// </summary>
    [JsonPropertyName("bundle_version")]
    public string? BundleVersion { get; set; }

    /// <summary>
    /// Plugin metadata.
    /// </summary>
    [JsonPropertyName("plugin")]
    public PluginInfo? Plugin { get; set; }

    /// <summary>
    /// Platform-specific libraries.
    /// </summary>
    [JsonPropertyName("platforms")]
    public Dictionary<string, PlatformInfo>? Platforms { get; set; }

    /// <summary>
    /// Minisign public key (base64).
    /// </summary>
    [JsonPropertyName("public_key")]
    public string? PublicKey { get; set; }

    /// <summary>
    /// Schema files in the bundle.
    /// </summary>
    [JsonPropertyName("schemas")]
    public Dictionary<string, SchemaInfo>? Schemas { get; set; }

    /// <summary>
    /// Build metadata (v1.0+).
    /// </summary>
    [JsonPropertyName("build_info")]
    public BuildInfo? BuildInfoData { get; set; }

    /// <summary>
    /// SBOM information (v1.0+).
    /// </summary>
    [JsonPropertyName("sbom")]
    public Sbom? SbomData { get; set; }

    /// <summary>
    /// Combined schema checksum for validation (v1.0+).
    /// </summary>
    [JsonPropertyName("schema_checksum")]
    public string? SchemaChecksum { get; set; }

    /// <summary>
    /// Path to license notices file in bundle (v1.0+).
    /// </summary>
    [JsonPropertyName("notices")]
    public string? Notices { get; set; }

    /// <summary>
    /// Bridge libraries bundled with the plugin (e.g., JNI bridge).
    /// </summary>
    [JsonPropertyName("bridges")]
    public BridgeInfo? Bridges { get; set; }

    /// <summary>
    /// Get the effective build info for a platform/variant (v1.1).
    /// Variant-level overrides top-level.
    /// </summary>
    /// <param name="platform">Platform string (e.g., "linux-x86_64").</param>
    /// <param name="variant">Variant name (e.g., "release").</param>
    /// <returns>Effective build info, or null if neither set.</returns>
    public BuildInfo? GetEffectiveBuildInfo(string platform, string variant)
    {
        if (Platforms != null && Platforms.TryGetValue(platform, out var pi)
            && pi.Variants != null && pi.Variants.TryGetValue(variant, out var vi)
            && vi.BuildInfoData != null)
        {
            return vi.BuildInfoData;
        }
        return BuildInfoData;
    }

    /// <summary>
    /// Get the effective schema checksum for a platform/variant (v1.1).
    /// Variant-level overrides top-level.
    /// </summary>
    /// <param name="platform">Platform string (e.g., "linux-x86_64").</param>
    /// <param name="variant">Variant name (e.g., "release").</param>
    /// <returns>Effective schema checksum, or null if neither set.</returns>
    public string? GetEffectiveSchemaChecksum(string platform, string variant)
    {
        if (Platforms != null && Platforms.TryGetValue(platform, out var pi)
            && pi.Variants != null && pi.Variants.TryGetValue(variant, out var vi)
            && vi.SchemaChecksum != null)
        {
            return vi.SchemaChecksum;
        }
        return SchemaChecksum;
    }

    /// <summary>
    /// Get the effective SBOM for a platform/variant (v1.1).
    /// Variant-level overrides top-level.
    /// </summary>
    public Sbom? GetEffectiveSbom(string platform, string variant)
    {
        if (Platforms != null && Platforms.TryGetValue(platform, out var pi)
            && pi.Variants != null && pi.Variants.TryGetValue(variant, out var vi)
            && vi.SbomData != null)
        {
            return vi.SbomData;
        }
        return SbomData;
    }

    /// <summary>
    /// Get the effective schemas for a platform/variant (v1.1).
    /// Variant-level overrides top-level.
    /// </summary>
    public Dictionary<string, SchemaInfo> GetEffectiveSchemas(string platform, string variant)
    {
        if (Platforms != null && Platforms.TryGetValue(platform, out var pi)
            && pi.Variants != null && pi.Variants.TryGetValue(variant, out var vi)
            && vi.Schemas != null && vi.Schemas.Count > 0)
        {
            return vi.Schemas;
        }
        return Schemas ?? new Dictionary<string, SchemaInfo>();
    }

    /// <summary>
    /// Plugin metadata information.
    /// </summary>
    public class PluginInfo
    {
        [JsonPropertyName("name")]
        public string? Name { get; set; }

        [JsonPropertyName("version")]
        public string? Version { get; set; }

        [JsonPropertyName("description")]
        public string? Description { get; set; }

        [JsonPropertyName("authors")]
        public List<string>? Authors { get; set; }

        [JsonPropertyName("license")]
        public string? License { get; set; }

        [JsonPropertyName("repository")]
        public string? Repository { get; set; }
    }

    /// <summary>
    /// Variant-specific library information.
    /// </summary>
    public class VariantInfo
    {
        /// <summary>
        /// Path to the library file within the bundle.
        /// </summary>
        [JsonPropertyName("library")]
        public string Library { get; set; } = "";

        /// <summary>
        /// SHA256 checksum of the library.
        /// </summary>
        [JsonPropertyName("checksum")]
        public string Checksum { get; set; } = "";

        /// <summary>
        /// Optional build metadata (profile, opt_level, features, etc.).
        /// </summary>
        [JsonPropertyName("build")]
        public object? Build { get; set; }

        /// <summary>
        /// Variant-level build info (v1.1, overrides top-level).
        /// </summary>
        [JsonPropertyName("build_info")]
        public BuildInfo? BuildInfoData { get; set; }

        /// <summary>
        /// Variant-level SBOM paths (v1.1, overrides top-level).
        /// </summary>
        [JsonPropertyName("sbom")]
        public Sbom? SbomData { get; set; }

        /// <summary>
        /// Variant-level schema checksum (v1.1, overrides top-level).
        /// </summary>
        [JsonPropertyName("schema_checksum")]
        public string? SchemaChecksum { get; set; }

        /// <summary>
        /// Variant-level schemas (v1.1, overrides top-level).
        /// </summary>
        [JsonPropertyName("schemas")]
        public Dictionary<string, SchemaInfo>? Schemas { get; set; }
    }

    /// <summary>
    /// Platform-specific library information with variant support.
    /// </summary>
    public class PlatformInfo
    {
        /// <summary>
        /// Path to the library file within the bundle (backward compat / default variant).
        /// </summary>
        [JsonPropertyName("library")]
        public string Library { get; set; } = "";

        /// <summary>
        /// SHA256 checksum of the library (backward compat / default variant).
        /// </summary>
        [JsonPropertyName("checksum")]
        public string Checksum { get; set; } = "";

        /// <summary>
        /// The default variant name (usually "release").
        /// </summary>
        [JsonPropertyName("default_variant")]
        public string? DefaultVariant { get; set; }

        /// <summary>
        /// Map of variant name to VariantInfo (v1.0+).
        /// </summary>
        [JsonPropertyName("variants")]
        public Dictionary<string, VariantInfo>? Variants { get; set; }

        /// <summary>
        /// Get the effective library path for a variant.
        /// </summary>
        public string GetLibrary(string variant)
        {
            if (Variants != null && Variants.TryGetValue(variant, out var variantInfo))
            {
                return variantInfo.Library;
            }
            return Library;
        }

        /// <summary>
        /// Get the effective checksum for a variant.
        /// </summary>
        public string GetChecksum(string variant)
        {
            if (Variants != null && Variants.TryGetValue(variant, out var variantInfo))
            {
                return variantInfo.Checksum;
            }
            return Checksum;
        }

        /// <summary>
        /// Get the default variant name.
        /// </summary>
        public string GetDefaultVariant() => DefaultVariant ?? "release";

        /// <summary>
        /// List available variants for this platform.
        /// </summary>
        public IReadOnlyList<string> ListVariants()
        {
            if (Variants == null || Variants.Count == 0)
            {
                return new[] { "release" };
            }
            return Variants.Keys.ToList();
        }
    }

    /// <summary>
    /// Schema file information.
    /// </summary>
    public class SchemaInfo
    {
        /// <summary>
        /// Path to the schema file within the bundle.
        /// </summary>
        [JsonPropertyName("path")]
        public string Path { get; set; } = "";

        /// <summary>
        /// Schema format (e.g., "json-schema", "c-header").
        /// </summary>
        [JsonPropertyName("format")]
        public string? Format { get; set; }

        /// <summary>
        /// SHA256 checksum of the schema file.
        /// </summary>
        [JsonPropertyName("checksum")]
        public string Checksum { get; set; } = "";

        /// <summary>
        /// Schema description.
        /// </summary>
        [JsonPropertyName("description")]
        public string? Description { get; set; }
    }

    /// <summary>
    /// Git repository information.
    /// </summary>
    public class GitInfo
    {
        /// <summary>
        /// Full commit hash (required if git section present).
        /// </summary>
        [JsonPropertyName("commit")]
        public string? Commit { get; set; }

        /// <summary>
        /// Branch name.
        /// </summary>
        [JsonPropertyName("branch")]
        public string? Branch { get; set; }

        /// <summary>
        /// Tag name (if on a tagged commit).
        /// </summary>
        [JsonPropertyName("tag")]
        public string? Tag { get; set; }

        /// <summary>
        /// Whether working directory had uncommitted changes.
        /// </summary>
        [JsonPropertyName("dirty")]
        public bool? Dirty { get; set; }
    }

    /// <summary>
    /// Build metadata information.
    /// </summary>
    public class BuildInfo
    {
        /// <summary>
        /// Who/what built the bundle (e.g., "CI/CD", "developer").
        /// </summary>
        [JsonPropertyName("built_by")]
        public string? BuiltBy { get; set; }

        /// <summary>
        /// ISO 8601 timestamp.
        /// </summary>
        [JsonPropertyName("built_at")]
        public string? BuiltAt { get; set; }

        /// <summary>
        /// Host triple (e.g., "x86_64-unknown-linux-gnu").
        /// </summary>
        [JsonPropertyName("host")]
        public string? Host { get; set; }

        /// <summary>
        /// Compiler version (e.g., "rustc 1.90.0").
        /// </summary>
        [JsonPropertyName("compiler")]
        public string? Compiler { get; set; }

        /// <summary>
        /// rustbridge CLI version.
        /// </summary>
        [JsonPropertyName("rustbridge_version")]
        public string? RustbridgeVersion { get; set; }

        /// <summary>
        /// Git repository info.
        /// </summary>
        [JsonPropertyName("git")]
        public GitInfo? Git { get; set; }

        /// <summary>
        /// Custom key-value metadata.
        /// </summary>
        [JsonPropertyName("custom")]
        public Dictionary<string, string>? Custom { get; set; }
    }

    /// <summary>
    /// Software Bill of Materials (SBOM) paths.
    /// </summary>
    public class Sbom
    {
        /// <summary>
        /// Path to CycloneDX SBOM file (e.g., "sbom/sbom.cdx.json").
        /// </summary>
        [JsonPropertyName("cyclonedx")]
        public string? Cyclonedx { get; set; }

        /// <summary>
        /// Path to SPDX SBOM file (e.g., "sbom/sbom.spdx.json").
        /// </summary>
        [JsonPropertyName("spdx")]
        public string? Spdx { get; set; }
    }

    /// <summary>
    /// Bridge libraries bundled with the plugin.
    /// <para>
    /// Allows bundling bridge libraries (like the JNI bridge) alongside
    /// the plugin for self-contained distribution.
    /// </para>
    /// </summary>
    public class BridgeInfo
    {
        /// <summary>
        /// JNI bridge libraries by platform.
        /// </summary>
        [JsonPropertyName("jni")]
        public Dictionary<string, PlatformInfo>? Jni { get; set; }
    }
}
