package com.rustbridge;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class JsonMapperTest {

    @Test
    void getInstance___returns_same_instance() {
        ObjectMapper first = JsonMapper.getInstance();
        ObjectMapper second = JsonMapper.getInstance();

        assertSame(first, second);
    }

    @Test
    void getInstance___returns_non_null() {
        ObjectMapper mapper = JsonMapper.getInstance();

        assertNotNull(mapper);
    }

    @Test
    void getInstance___ignores_unknown_properties() throws JsonProcessingException {
        ObjectMapper mapper = JsonMapper.getInstance();
        String json = "{\"known\":\"value\",\"unknown\":\"ignored\"}";

        TestClass result = mapper.readValue(json, TestClass.class);

        assertEquals("value", result.known);
    }

    @Test
    void getInstance___serializes_objects() throws JsonProcessingException {
        ObjectMapper mapper = JsonMapper.getInstance();
        TestClass obj = new TestClass();
        obj.known = "test";

        String json = mapper.writeValueAsString(obj);

        assertTrue(json.contains("\"known\":\"test\""));
    }

    @Test
    void getInstance___deserializes_objects() throws JsonProcessingException {
        ObjectMapper mapper = JsonMapper.getInstance();
        String json = "{\"known\":\"test\"}";

        TestClass result = mapper.readValue(json, TestClass.class);

        assertEquals("test", result.known);
    }

    public static class TestClass {
        public String known;
    }
}
