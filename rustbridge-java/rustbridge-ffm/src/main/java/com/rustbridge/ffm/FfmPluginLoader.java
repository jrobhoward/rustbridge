package com.rustbridge.ffm;

import com.rustbridge.*;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;

/**
 * Loader for rustbridge plugins using Java 21+ FFM API.
 * <p>
 * This loader uses Project Panama's Foreign Function & Memory API
 * to load and interact with native plugins without JNI.
 *
 * <h2>Usage</h2>
 * <pre>{@code
 * try (Plugin plugin = FfmPluginLoader.load("libmyplugin.so")) {
 *     String response = plugin.call("echo", "{\"message\": \"hello\"}");
 *     System.out.println(response);
 * }
 * }</pre>
 */
public class FfmPluginLoader {

    private FfmPluginLoader() {
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
        return load(Path.of(libraryPath), PluginConfig.defaults(), null);
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
        return load(Path.of(libraryPath), config, null);
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
    public static Plugin load(Path libraryPath, PluginConfig config, LogCallback logCallback)
            throws PluginException {

        // Create arena for plugin lifetime
        Arena arena = Arena.ofConfined();

        try {
            // Load the native library
            Linker linker = Linker.nativeLinker();
            SymbolLookup lookup = SymbolLookup.libraryLookup(libraryPath, arena);

            // Create bindings
            NativeBindings bindings = new NativeBindings(lookup, linker);

            // Look up plugin_create function
            MethodHandle pluginCreate = linker.downcallHandle(
                    lookup.find("plugin_create").orElseThrow(() ->
                            new PluginException("plugin_create function not found")),
                    FunctionDescriptor.of(ValueLayout.ADDRESS)
            );

            // Create the plugin instance
            MemorySegment pluginPtr;
            try {
                pluginPtr = (MemorySegment) pluginCreate.invokeExact();
            } catch (Throwable t) {
                throw new PluginException("Failed to create plugin instance", t);
            }

            if (pluginPtr.equals(MemorySegment.NULL)) {
                throw new PluginException("plugin_create returned null");
            }

            // Prepare config
            byte[] configBytes = config.toJsonBytes();
            MemorySegment configSegment = arena.allocate(configBytes.length);
            configSegment.copyFrom(MemorySegment.ofArray(configBytes));

            // Prepare log callback (TODO: implement callback upcall)
            MemorySegment logCallbackPtr = MemorySegment.NULL;

            // Initialize the plugin
            MemorySegment handle;
            try {
                handle = (MemorySegment) bindings.pluginInit().invokeExact(
                        pluginPtr,
                        configSegment,
                        (long) configBytes.length,
                        logCallbackPtr
                );
            } catch (Throwable t) {
                throw new PluginException("Failed to initialize plugin", t);
            }

            if (handle.equals(MemorySegment.NULL)) {
                throw new PluginException("plugin_init returned null handle");
            }

            return new FfmPlugin(arena, handle, bindings, logCallback);

        } catch (PluginException e) {
            arena.close();
            throw e;
        } catch (Exception e) {
            arena.close();
            throw new PluginException("Failed to load plugin", e);
        }
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
            if (basePath.isEmpty()) continue;

            Path fullPath = Path.of(basePath, libraryFileName);
            if (fullPath.toFile().exists()) {
                return load(fullPath, config, null);
            }
        }

        throw new PluginException("Could not find library: " + libraryFileName);
    }
}
