package com.rustbridge.jni;

import com.rustbridge.*;
import com.rustbridge.jni.JniNativeLibraryCondition.RequiresJniLibrary;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Basic smoke tests for JNI plugin loading.
 */
@RequiresJniLibrary
@Timeout(value = 30, unit = TimeUnit.SECONDS)
class JniPluginLoaderTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Plugin can be loaded")
    void load___valid_path___succeeds() throws PluginException {
        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString())) {
            assertNotNull(plugin);
        }
    }

    @Test
    @DisplayName("Plugin can be loaded with configuration")
    void load___with_config___succeeds() throws PluginException {
        PluginConfig config = PluginConfig.defaults()
                .workerThreads(1);

        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            assertNotNull(plugin);
        }
    }

    @Test
    @Disabled("loadByName searches in standard paths - needs library in java.library.path or current dir")
    @DisplayName("Plugin can be loaded by name")
    void load_by_name___valid_name___succeeds() throws PluginException {
        try (Plugin plugin = JniPluginLoader.loadByName("hello_plugin")) {
            assertNotNull(plugin);
        }
    }

    @Test
    @DisplayName("Invalid library path throws exception")
    void load___invalid_path___throws_exception() {
        assertThrows(PluginException.class, () -> {
            JniPluginLoader.load("/nonexistent/path/libfake.so");
        });
    }
}
