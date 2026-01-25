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
    /// API information.
    /// </summary>
    [JsonPropertyName("api")]
    public ApiInfo? Api { get; set; }

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
    /// Platform-specific library information.
    /// </summary>
    public class PlatformInfo
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
    }

    /// <summary>
    /// API information for the plugin.
    /// </summary>
    public class ApiInfo
    {
        [JsonPropertyName("min_rustbridge_version")]
        public string? MinRustbridgeVersion { get; set; }

        [JsonPropertyName("transports")]
        public List<string>? Transports { get; set; }

        [JsonPropertyName("messages")]
        public List<MessageInfo>? Messages { get; set; }
    }

    /// <summary>
    /// Message type information.
    /// </summary>
    public class MessageInfo
    {
        [JsonPropertyName("type_tag")]
        public string? TypeTag { get; set; }

        [JsonPropertyName("description")]
        public string? Description { get; set; }

        [JsonPropertyName("request_schema")]
        public string? RequestSchema { get; set; }

        [JsonPropertyName("response_schema")]
        public string? ResponseSchema { get; set; }

        [JsonPropertyName("message_id")]
        public int? MessageId { get; set; }

        [JsonPropertyName("cstruct_request")]
        public string? CstructRequest { get; set; }

        [JsonPropertyName("cstruct_response")]
        public string? CstructResponse { get; set; }
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
}
