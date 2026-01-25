package com.rustbridge;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ResponseEnvelopeTest {

    @Test
    void fromJson___success_response___parses_correctly() {
        String json = "{\"status\":\"success\",\"payload\":{\"message\":\"hello\",\"length\":5}}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);

        assertTrue(envelope.isSuccess());
        assertNull(envelope.getErrorCode());
        assertNull(envelope.getErrorMessage());
    }

    @Test
    void fromJson___error_response___parses_correctly() {
        String json = "{\"status\":\"error\",\"error_code\":5,\"error_message\":\"Serialization error\"}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);

        assertFalse(envelope.isSuccess());
        assertEquals(5, envelope.getErrorCode());
        assertEquals("Serialization error", envelope.getErrorMessage());
    }

    @Test
    void fromJson___with_request_id___parses_correctly() {
        String json = "{\"status\":\"success\",\"payload\":{},\"request_id\":12345}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);

        assertTrue(envelope.isSuccess());
        assertEquals(12345L, envelope.getRequestId());
    }

    @Test
    void fromJson___unknown_fields___ignores_them() {
        String json = "{\"status\":\"success\",\"payload\":{},\"unknown_field\":\"value\"}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);

        assertTrue(envelope.isSuccess());
    }

    @Test
    void getPayloadJson___returns_json_string() {
        String json = "{\"status\":\"success\",\"payload\":{\"key\":\"value\",\"num\":42}}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);
        String payloadJson = envelope.getPayloadJson();

        assertNotNull(payloadJson);
        assertTrue(payloadJson.contains("\"key\":\"value\""));
        assertTrue(payloadJson.contains("\"num\":42"));
    }

    @Test
    void getPayloadJson___null_payload___returns_null() {
        String json = "{\"status\":\"error\",\"error_code\":1,\"error_message\":\"error\"}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);
        String payloadJson = envelope.getPayloadJson();

        assertNull(payloadJson);
    }

    @Test
    void getPayload___deserializes_to_type() {
        String json = "{\"status\":\"success\",\"payload\":{\"message\":\"hello\",\"length\":5}}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);
        TestPayload payload = envelope.getPayload(TestPayload.class);

        assertNotNull(payload);
        assertEquals("hello", payload.message);
        assertEquals(5, payload.length);
    }

    @Test
    void getPayload___null_payload___returns_null() {
        String json = "{\"status\":\"error\",\"error_code\":1,\"error_message\":\"error\"}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);
        TestPayload payload = envelope.getPayload(TestPayload.class);

        assertNull(payload);
    }

    @Test
    void toException___creates_plugin_exception() {
        String json = "{\"status\":\"error\",\"error_code\":6,\"error_message\":\"Unknown message type\"}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);
        PluginException exception = envelope.toException();

        assertEquals(6, exception.getErrorCode());
        assertEquals("Unknown message type", exception.getMessage());
    }

    @Test
    void toException___null_error_code___uses_zero() {
        String json = "{\"status\":\"error\",\"error_message\":\"Some error\"}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);
        PluginException exception = envelope.toException();

        assertEquals(0, exception.getErrorCode());
        assertEquals("Some error", exception.getMessage());
    }

    @Test
    void toException___null_error_message___uses_default() {
        String json = "{\"status\":\"error\",\"error_code\":1}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);
        PluginException exception = envelope.toException();

        assertEquals(1, exception.getErrorCode());
        assertEquals("Unknown error", exception.getMessage());
    }

    @Test
    void fromJson___invalid_json___throws_runtime_exception() {
        String invalidJson = "{broken json}";

        assertThrows(RuntimeException.class, () -> {
            ResponseEnvelope.fromJson(invalidJson);
        });
    }

    @Test
    void isSuccess___non_success_status___returns_false() {
        String json = "{\"status\":\"failed\"}";

        ResponseEnvelope envelope = ResponseEnvelope.fromJson(json);

        assertFalse(envelope.isSuccess());
    }

    /**
     * Test helper class for payload deserialization.
     */
    public static class TestPayload {
        public String message;
        public int length;
    }
}
