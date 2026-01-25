package com.rustbridge;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Centralized ObjectMapper configuration for rustbridge.
 * <p>
 * Provides a singleton ObjectMapper instance configured with rustbridge defaults.
 * Using a shared instance avoids the overhead of creating multiple ObjectMapper
 * instances and ensures consistent JSON serialization behavior across the library.
 */
public final class JsonMapper {
    private static final ObjectMapper INSTANCE = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

    private JsonMapper() {
        // Utility class, no instantiation
    }

    /**
     * Get the shared ObjectMapper instance.
     * <p>
     * This instance is thread-safe for concurrent use. It is configured to:
     * <ul>
     *   <li>Ignore unknown properties during deserialization</li>
     * </ul>
     *
     * @return the shared ObjectMapper instance
     */
    public static ObjectMapper getInstance() {
        return INSTANCE;
    }
}
