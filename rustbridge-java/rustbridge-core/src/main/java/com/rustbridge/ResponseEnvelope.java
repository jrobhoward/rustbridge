package com.rustbridge;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

/**
 * Response envelope for FFI communication.
 */
public class ResponseEnvelope {
    private static final ObjectMapper OBJECT_MAPPER = JsonMapper.getInstance();

    @JsonProperty("status")
    private @Nullable String status;

    @JsonProperty("payload")
    private @Nullable JsonNode payload;

    @JsonProperty("error_code")
    private @Nullable Integer errorCode;

    @JsonProperty("error_message")
    private @Nullable String errorMessage;

    @JsonProperty("request_id")
    private @Nullable Long requestId;

    /**
     * Parse a response envelope from JSON.
     *
     * @param json the JSON string
     * @return the parsed envelope
     */
    public static @NotNull ResponseEnvelope fromJson(@NotNull String json) {
        try {
            return OBJECT_MAPPER.readValue(json, ResponseEnvelope.class);
        } catch (JsonProcessingException e) {
            throw new RuntimeException("Failed to parse response envelope", e);
        }
    }

    /**
     * Check if the response indicates success.
     *
     * @return true if successful
     */
    public boolean isSuccess() {
        return "success".equals(status);
    }

    /**
     * Get the payload as a JSON string.
     *
     * @return the payload JSON, or null if error
     */
    public @Nullable String getPayloadJson() {
        try {
            return payload != null ? OBJECT_MAPPER.writeValueAsString(payload) : null;
        } catch (JsonProcessingException e) {
            throw new RuntimeException("Failed to serialize payload", e);
        }
    }

    /**
     * Get the payload deserialized to a specific type.
     *
     * @param type the target type
     * @param <T>  the type parameter
     * @return the deserialized payload, or null if no payload
     */
    public <T> @Nullable T getPayload(@NotNull Class<T> type) {
        try {
            return payload != null ? OBJECT_MAPPER.treeToValue(payload, type) : null;
        } catch (JsonProcessingException e) {
            throw new RuntimeException("Failed to deserialize payload", e);
        }
    }

    /**
     * Get the error code.
     *
     * @return the error code, or null if success
     */
    public @Nullable Integer getErrorCode() {
        return errorCode;
    }

    /**
     * Get the error message.
     *
     * @return the error message, or null if success
     */
    public @Nullable String getErrorMessage() {
        return errorMessage;
    }

    /**
     * Get the request ID.
     *
     * @return the request ID, or null if not set
     */
    public @Nullable Long getRequestId() {
        return requestId;
    }

    /**
     * Convert a PluginException based on the error in this envelope.
     *
     * @return the exception
     */
    public @NotNull PluginException toException() {
        int code = errorCode != null ? errorCode : 0;
        String message = errorMessage != null ? errorMessage : "Unknown error";
        return new PluginException(code, message);
    }
}
