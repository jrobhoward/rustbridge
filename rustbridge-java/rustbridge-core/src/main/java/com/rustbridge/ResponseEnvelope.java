package com.rustbridge;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Response envelope for FFI communication.
 */
public class ResponseEnvelope {
    private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

    @JsonProperty("status")
    private String status;

    @JsonProperty("payload")
    private JsonNode payload;

    @JsonProperty("error_code")
    private Integer errorCode;

    @JsonProperty("error_message")
    private String errorMessage;

    @JsonProperty("request_id")
    private Long requestId;

    /**
     * Parse a response envelope from JSON.
     *
     * @param json the JSON string
     * @return the parsed envelope
     */
    public static ResponseEnvelope fromJson(String json) {
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
    public String getPayloadJson() {
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
     * @return the deserialized payload
     */
    public <T> T getPayload(Class<T> type) {
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
    public Integer getErrorCode() {
        return errorCode;
    }

    /**
     * Get the error message.
     *
     * @return the error message, or null if success
     */
    public String getErrorMessage() {
        return errorMessage;
    }

    /**
     * Get the request ID.
     *
     * @return the request ID, or null if not set
     */
    public Long getRequestId() {
        return requestId;
    }

    /**
     * Convert a PluginException based on the error in this envelope.
     *
     * @return the exception
     */
    public PluginException toException() {
        int code = errorCode != null ? errorCode : 0;
        String message = errorMessage != null ? errorMessage : "Unknown error";
        return new PluginException(code, message);
    }
}
