package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;
import org.junit.jupiter.api.Disabled;

import java.nio.file.Path;
import java.nio.file.Paths;
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
        // Find the hello-plugin library
        String osName = System.getProperty("os.name").toLowerCase();
        String libraryName;

        if (osName.contains("linux")) {
            libraryName = "libhello_plugin.so";
        } else if (osName.contains("mac") || osName.contains("darwin")) {
            libraryName = "libhello_plugin.dylib";
        } else if (osName.contains("windows")) {
            libraryName = "hello_plugin.dll";
        } else {
            throw new RuntimeException("Unsupported OS: " + osName);
        }

        // Look in target/release first, then target/debug
        Path releasePath = Paths.get("../../target/release").resolve(libraryName);
        Path debugPath = Paths.get("../../target/debug").resolve(libraryName);

        if (releasePath.toFile().exists()) {
            PLUGIN_PATH = releasePath.toAbsolutePath();
        } else if (debugPath.toFile().exists()) {
            PLUGIN_PATH = debugPath.toAbsolutePath();
        } else {
            fail("Could not find hello-plugin library. Build it with: cargo build -p hello-plugin --release");
        }

        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Plugin can be loaded")
    void testPluginLoads() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            assertNotNull(plugin);
        }
    }

    @Test
    @DisplayName("Plugin can be loaded with configuration")
    void testPluginLoadsWithConfig() throws PluginException {
        PluginConfig config = PluginConfig.defaults()
                .workerThreads(1);

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            assertNotNull(plugin);
        }
    }

    @Test
    @Disabled("loadByName searches in standard paths - needs library in java.library.path or current dir")
    @DisplayName("Plugin can be loaded by name")
    void testPluginLoadByName() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.loadByName("hello_plugin")) {
            assertNotNull(plugin);
        }
    }

    @Test
    @DisplayName("Invalid library path throws exception")
    void testInvalidPath() {
        assertThrows(PluginException.class, () -> {
            FfmPluginLoader.load("/nonexistent/path/libfake.so");
        });
    }
}
