package com.rustbridge.jni;

import com.rustbridge.*;

import java.io.File;

/**
 * Loader for rustbridge plugins using JNI (Java 8+ compatible).
 * <p>
 * This loader uses JNI to load and interact with native plugins.
 * For Java 21+, consider using {@code FfmPluginLoader} instead for
 * better performance.
 *
 * <h2>Usage</h2>
 * <pre>{@code
 * try (Plugin plugin = JniPluginLoader.load("libmyplugin.so")) {
 *     String response = plugin.call("echo", "{\"message\": \"hello\"}");
 *     System.out.println(response);
 * }
 * }</pre>
 */
public class JniPluginLoader {

    static {
        // Load the JNI bridge library
        try {
            System.loadLibrary("rustbridge_jni");
        } catch (UnsatisfiedLinkError e) {
            throw new RuntimeException("Failed to load rustbridge_jni native library", e);
        }
    }

    private JniPluginLoader() {
        // Utility class
    }

    /**
     * Load a plugin from the specified library path.
     *
     * @param libraryPath path to the shared library
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static Plugin load(String libraryPath) throws PluginException {
        return load(libraryPath, PluginConfig.defaults(), null);
    }

    /**
     * Load a plugin with configuration.
     *
     * @param libraryPath path to the shared library
     * @param config      plugin configuration
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static Plugin load(String libraryPath, PluginConfig config) throws PluginException {
        return load(libraryPath, config, null);
    }

    /**
     * Load a plugin with configuration and log callback.
     *
     * @param libraryPath path to the shared library
     * @param config      plugin configuration
     * @param logCallback optional callback for log messages
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static Plugin load(String libraryPath, PluginConfig config, LogCallback logCallback)
            throws PluginException {

        byte[] configBytes = config.toJsonBytes();
        long handle = nativeLoadPlugin(libraryPath, configBytes);

        if (handle == 0) {
            throw new PluginException("Failed to load plugin from: " + libraryPath);
        }

        return new JniPlugin(handle, logCallback);
    }

    /**
     * Load a plugin by name, searching in standard library paths.
     *
     * @param libraryName the library name (without lib prefix or extension)
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static Plugin loadByName(String libraryName) throws PluginException {
        return loadByName(libraryName, PluginConfig.defaults());
    }

    /**
     * Load a plugin by name with configuration.
     *
     * @param libraryName the library name (without lib prefix or extension)
     * @param config      plugin configuration
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static Plugin loadByName(String libraryName, PluginConfig config) throws PluginException {
        String osName = System.getProperty("os.name").toLowerCase();
        String libraryFileName;

        if (osName.contains("linux")) {
            libraryFileName = "lib" + libraryName + ".so";
        } else if (osName.contains("mac") || osName.contains("darwin")) {
            libraryFileName = "lib" + libraryName + ".dylib";
        } else if (osName.contains("windows")) {
            libraryFileName = libraryName + ".dll";
        } else {
            throw new PluginException("Unsupported operating system: " + osName);
        }

        // Search in common locations
        String[] searchPaths = {
                ".",
                "./target/release",
                "./target/debug",
                System.getProperty("java.library.path", "")
        };

        for (String basePath : searchPaths) {
            if (basePath == null || basePath.isEmpty()) continue;

            File fullPath = new File(basePath, libraryFileName);
            if (fullPath.exists()) {
                return load(fullPath.getAbsolutePath(), config, null);
            }
        }

        throw new PluginException("Could not find library: " + libraryFileName);
    }

    // Native methods

    private static native long nativeLoadPlugin(String libraryPath, byte[] configJson)
            throws PluginException;
}
