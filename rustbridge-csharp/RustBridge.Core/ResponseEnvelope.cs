using System.Text.Json;
using System.Text.Json.Serialization;

namespace RustBridge;

/// <summary>
/// Response envelope from plugin calls.
/// <para>
/// All plugin responses are wrapped in this envelope format, which includes
/// the type tag, payload, and optional error information.
/// </para>
/// </summary>
public class ResponseEnvelope
{
    /// <summary>
    /// The message type tag.
    /// </summary>
    [JsonPropertyName("type_tag")]
    public string? TypeTag { get; set; }

    /// <summary>
    /// The response payload as a JSON element.
    /// </summary>
    [JsonPropertyName("payload")]
    public JsonElement? Payload { get; set; }

    /// <summary>
    /// The error code (0 = success).
    /// </summary>
    [JsonPropertyName("error_code")]
    public int ErrorCode { get; set; }

    /// <summary>
    /// The error message (if any).
    /// </summary>
    [JsonPropertyName("error_message")]
    public string? ErrorMessage { get; set; }

    /// <summary>
    /// Check if this response represents a success.
    /// </summary>
    public bool IsSuccess => ErrorCode == 0;

    /// <summary>
    /// Get the payload as a JSON string.
    /// </summary>
    /// <returns>The payload JSON string, or "null" if no payload.</returns>
    public string GetPayloadJson()
    {
        if (Payload is null)
        {
            return "null";
        }
        return Payload.Value.GetRawText();
    }

    /// <summary>
    /// Convert this error response to a PluginException.
    /// </summary>
    /// <returns>A PluginException with the error details.</returns>
    public PluginException ToException()
    {
        return new PluginException(ErrorCode, ErrorMessage ?? "Unknown error");
    }

    /// <summary>
    /// Parse a response envelope from JSON.
    /// </summary>
    /// <param name="json">The JSON string.</param>
    /// <returns>The parsed envelope.</returns>
    public static ResponseEnvelope FromJson(string json)
    {
        return JsonSerializer.Deserialize<ResponseEnvelope>(json)
            ?? throw new PluginException("Failed to parse response envelope");
    }
}
