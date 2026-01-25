package com.example.messages;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Response to a greeting request.
 */
public class GreetingResponse {

    /**
     * The generated greeting message.
     */
    public String message;

    /**
     * Timestamp when the greeting was created (Unix epoch milliseconds).
     */
    public long timestamp;

    public GreetingResponse() {}

    public GreetingResponse(String message, long timestamp) {
        this.message = message;
        this.timestamp = timestamp;
    }

    public String getMessage() {
        return message;
    }

    public void setMessage(String message) {
        this.message = message;
    }

    public long getTimestamp() {
        return timestamp;
    }

    public void setTimestamp(long timestamp) {
        this.timestamp = timestamp;
    }
}
