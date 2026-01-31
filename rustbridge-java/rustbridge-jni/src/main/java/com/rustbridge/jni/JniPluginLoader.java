package com.rustbridge.jni;

import com.rustbridge.*;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.SignatureException;

/**
 * Loader for rustbridge plugins using JNI (Java 17+ compatible).
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

    private static boolean jniBridgeLoaded = false;

    static {
        // Try to load the JNI bridge library from system path.
        // This may fail if the library isn't installed system-wide,
        // in which case loadFromBundle() will load it from the bundle.
        try {
            System.loadLibrary("rustbridge_jni");
            jniBridgeLoaded = true;
        } catch (UnsatisfiedLinkError e) {
            // Library not found in system path - this is OK if loadFromBundle() will be used.
            // If load() is called directly without the library, it will fail later.
            jniBridgeLoaded = false;
        }
    }

    /**
     * Check if the JNI bridge library has been loaded.
     *
     * @return true if the JNI bridge is loaded and ready
     */
    public static boolean isJniBridgeLoaded() {
        return jniBridgeLoaded;
    }

    /**
     * Ensure the JNI bridge is loaded, throwing if not.
     */
    private static void ensureJniBridgeLoaded() throws PluginException {
        if (!jniBridgeLoaded) {
            throw new PluginException(
                "JNI bridge library not loaded. Either install librustbridge_jni in the system path, " +
                "or use loadFromBundle() with a bundle that includes the JNI bridge."
            );
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
    public static @NotNull Plugin load(@NotNull String libraryPath) throws PluginException {
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
    public static @NotNull Plugin load(@NotNull String libraryPath, @NotNull PluginConfig config) throws PluginException {
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
    public static @NotNull Plugin load(@NotNull String libraryPath, @NotNull PluginConfig config, @Nullable LogCallback logCallback)
            throws PluginException {

        ensureJniBridgeLoaded();

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
    public static @NotNull Plugin loadByName(@NotNull String libraryName) throws PluginException {
        return loadByName(libraryName, PluginConfig.defaults());
    }

    /**
     * Load a plugin from a bundle file.
     *
     * <p>This method extracts both the JNI bridge library (if present in the bundle)
     * and the plugin library to a temporary directory, then loads them. If the bundle
     * does not contain a JNI bridge, it will try to load the system-installed JNI bridge.
     *
     * @param bundlePath path to the .rbp bundle file
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static @NotNull Plugin loadFromBundle(@NotNull String bundlePath) throws PluginException {
        return loadFromBundle(bundlePath, PluginConfig.defaults(), null);
    }

    /**
     * Load a plugin from a bundle file with configuration.
     *
     * @param bundlePath path to the .rbp bundle file
     * @param config     plugin configuration
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static @NotNull Plugin loadFromBundle(@NotNull String bundlePath, @NotNull PluginConfig config) throws PluginException {
        return loadFromBundle(bundlePath, config, null);
    }

    /**
     * Load a plugin from a bundle file with configuration and log callback.
     *
     * <p>This method extracts both the JNI bridge library (if present in the bundle)
     * and the plugin library to the same temporary directory, ensuring version compatibility.
     * If the bundle does not contain a JNI bridge, it will try to load the system-installed
     * JNI bridge via {@code System.loadLibrary("rustbridge_jni")}.
     *
     * @param bundlePath  path to the .rbp bundle file
     * @param config      plugin configuration
     * @param logCallback optional callback for log messages
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static @NotNull Plugin loadFromBundle(
            @NotNull String bundlePath,
            @NotNull PluginConfig config,
            @Nullable LogCallback logCallback
    ) throws PluginException {
        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false) // Signature verification done during extraction
                .build()) {

            // Create temp directory for this bundle's libraries
            // Both JNI bridge and plugin extracted together to ensure version match
            Path tempDir = Files.createTempDirectory("rustbridge-");

            // Extract and load JNI bridge if present in bundle
            if (loader.hasJniBridge()) {
                Path jniBridge = loader.extractJniBridge(tempDir);
                System.load(jniBridge.toAbsolutePath().toString());
                jniBridgeLoaded = true;
            } else if (!jniBridgeLoaded) {
                // No JNI bridge in bundle and none loaded from system path
                throw new PluginException(
                    "Bundle does not contain JNI bridge and no system JNI bridge is available. " +
                    "Either include --jni-lib when creating the bundle, or install librustbridge_jni in the system path."
                );
            }

            // Extract plugin to same temp directory (version-matched pair)
            Path pluginLib = loader.extractLibrary(tempDir);
            return load(pluginLib.toString(), config, logCallback);

        } catch (IOException e) {
            throw new PluginException("Failed to load plugin from bundle: " + e.getMessage(), e);
        } catch (SignatureException e) {
            throw new PluginException("Bundle signature verification failed: " + e.getMessage(), e);
        }
    }

    /**
     * Load a plugin by name with configuration.
     *
     * @param libraryName the library name (without lib prefix or extension)
     * @param config      plugin configuration
     * @return the loaded plugin
     * @throws PluginException if loading fails
     */
    public static @NotNull Plugin loadByName(@NotNull String libraryName, @NotNull PluginConfig config) throws PluginException {
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
