package com.rustbridge;

import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

class PluginConfigTest {

    @Test
    void defaults___creates_config_with_default_values() {
        PluginConfig config = PluginConfig.defaults();

        byte[] json = config.toJsonBytes();
        String jsonStr = new String(json);

        assertTrue(jsonStr.contains("\"log_level\":\"info\""));
        assertTrue(jsonStr.contains("\"max_concurrent_ops\":1000"));
        assertTrue(jsonStr.contains("\"shutdown_timeout_ms\":5000"));
    }

    @Test
    void initParam___adds_single_parameter() {
        PluginConfig config = PluginConfig.defaults()
            .initParam("migrations_path", "/db/migrations")
            .initParam("seed_data", true);

        byte[] json = config.toJsonBytes();
        String jsonStr = new String(json);

        assertTrue(jsonStr.contains("\"init_params\""));
        assertTrue(jsonStr.contains("\"migrations_path\":\"/db/migrations\""));
        assertTrue(jsonStr.contains("\"seed_data\":true"));
    }

    @Test
    void initParams___sets_all_parameters_at_once() {
        Map<String, Object> params = new HashMap<>();
        params.put("setup_mode", "development");
        params.put("max_retries", 5);
        params.put("enable_cache", false);

        PluginConfig config = PluginConfig.defaults()
            .initParams(params);

        byte[] json = config.toJsonBytes();
        String jsonStr = new String(json);

        assertTrue(jsonStr.contains("\"init_params\""));
        assertTrue(jsonStr.contains("\"setup_mode\":\"development\""));
        assertTrue(jsonStr.contains("\"max_retries\":5"));
        assertTrue(jsonStr.contains("\"enable_cache\":false"));
    }

    @Test
    void initParams___replaces_existing_parameters() {
        Map<String, Object> params1 = new HashMap<>();
        params1.put("key1", "value1");

        Map<String, Object> params2 = new HashMap<>();
        params2.put("key2", "value2");

        PluginConfig config = PluginConfig.defaults()
            .initParams(params1)
            .initParams(params2);  // Should replace params1

        byte[] json = config.toJsonBytes();
        String jsonStr = new String(json);

        assertFalse(jsonStr.contains("\"key1\""));
        assertTrue(jsonStr.contains("\"key2\":\"value2\""));
    }

    @Test
    void initParam___can_be_chained_with_other_config() {
        PluginConfig config = PluginConfig.defaults()
            .logLevel("debug")
            .workerThreads(4)
            .maxConcurrentOps(100)
            .initParam("setup_mode", "production")
            .set("api_key", "secret123");

        byte[] json = config.toJsonBytes();
        String jsonStr = new String(json);

        // Check all fields are present
        assertTrue(jsonStr.contains("\"log_level\":\"debug\""));
        assertTrue(jsonStr.contains("\"worker_threads\":4"));
        assertTrue(jsonStr.contains("\"max_concurrent_ops\":100"));
        assertTrue(jsonStr.contains("\"init_params\""));
        assertTrue(jsonStr.contains("\"setup_mode\":\"production\""));
        assertTrue(jsonStr.contains("\"api_key\":\"secret123\""));
    }

    @Test
    void toJsonBytes___omits_init_params_when_not_set() {
        PluginConfig config = PluginConfig.defaults()
            .logLevel("info");

        byte[] json = config.toJsonBytes();
        String jsonStr = new String(json);

        assertFalse(jsonStr.contains("\"init_params\""));
    }

    @Test
    void initParam___works_with_nested_objects() {
        Map<String, Object> databaseConfig = new HashMap<>();
        databaseConfig.put("url", "postgres://localhost/test");
        databaseConfig.put("pool_size", 10);

        PluginConfig config = PluginConfig.defaults()
            .initParam("database", databaseConfig);

        byte[] json = config.toJsonBytes();
        String jsonStr = new String(json);

        assertTrue(jsonStr.contains("\"database\""));
        assertTrue(jsonStr.contains("\"url\":\"postgres://localhost/test\""));
        assertTrue(jsonStr.contains("\"pool_size\":10"));
    }
}
