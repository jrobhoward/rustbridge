package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Basic smoke tests for FFM plugin loading.
 */
@Timeout(value = 30, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class FfmPluginLoaderTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Plugin can be loaded")
    void load___valid_path___plugin_active() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            assertNotNull(plugin);
        }
    }

    @Test
    @DisplayName("Plugin can be loaded with configuration")
    void load___with_config___plugin_active() throws PluginException {
        PluginConfig config = PluginConfig.defaults()
                .workerThreads(1);

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            assertNotNull(plugin);
        }
    }

    @Test
    @Disabled("loadByName searches in standard paths - needs library in java.library.path or current dir")
    @DisplayName("Plugin can be loaded by name")
    void loadByName___valid_name___plugin_active() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.loadByName("hello_plugin")) {
            assertNotNull(plugin);
        }
    }

    @Test
    @DisplayName("Invalid library path throws exception")
    void load___invalid_path___throws_exception() {
        assertThrows(PluginException.class, () -> {
            FfmPluginLoader.load("/nonexistent/path/libfake.so");
        });
    }
}
