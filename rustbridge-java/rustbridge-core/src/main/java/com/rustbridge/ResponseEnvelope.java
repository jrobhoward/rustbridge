package com.rustbridge;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.annotations.SerializedName;

/**
 * Response envelope for FFI communication.
 */
public class ResponseEnvelope {
    private static final Gson GSON = new GsonBuilder().create();

    @SerializedName("status")
    private String status;

    @SerializedName("payload")
    private JsonElement payload;

    @SerializedName("error_code")
    private Integer errorCode;

    @SerializedName("error_message")
    private String errorMessage;

    @SerializedName("request_id")
    private Long requestId;

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
        return payload != null ? GSON.toJson(payload) : null;
    }

    /**
     * Get the payload deserialized to a specific type.
     *
     * @param type the target type
     * @param <T>  the type parameter
     * @return the deserialized payload
     */
    public <T> T getPayload(Class<T> type) {
        return payload != null ? GSON.fromJson(payload, type) : null;
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
     * Parse a response envelope from JSON.
     *
     * @param json the JSON string
     * @return the parsed envelope
     */
    public static ResponseEnvelope fromJson(String json) {
        return GSON.fromJson(json, ResponseEnvelope.class);
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
