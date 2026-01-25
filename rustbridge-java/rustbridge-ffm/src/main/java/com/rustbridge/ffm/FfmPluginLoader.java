package com.rustbridge.ffm;

import com.rustbridge.*;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;

/**
 * Loader for rustbridge plugins using Java 21+ FFM API.
 * <p>
 * This loader uses Project Panama's Foreign Function and Memory API
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

        // Create plugin-lifetime arena for library symbols and upcall stub (shared for concurrent access)
        Arena pluginArena = Arena.ofShared();

        try {
            // Load the native library
            Linker linker = Linker.nativeLinker();
            SymbolLookup lookup = SymbolLookup.libraryLookup(libraryPath, pluginArena);

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
            MemorySegment configSegment = pluginArena.allocate(configBytes.length);
            configSegment.copyFrom(MemorySegment.ofArray(configBytes));

            // Prepare log callback upcall
            MemorySegment logCallbackPtr = createLogCallbackUpcall(linker, pluginArena, logCallback);

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

            return new FfmPlugin(pluginArena, handle, bindings, logCallback);

        } catch (PluginException e) {
            pluginArena.close();
            throw e;
        } catch (Exception e) {
            pluginArena.close();
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

    /**
     * Create an upcall stub for the log callback.
     * <p>
     * This creates a native function pointer that can be called from Rust code.
     * When called, it will invoke the Java LogCallback.
     *
     * @param linker      the native linker
     * @param pluginArena the plugin-lifetime memory arena for the upcall stub
     * @param logCallback the Java log callback (nullable)
     * @return memory segment pointing to the upcall stub, or NULL if callback is null
     */
    private static MemorySegment createLogCallbackUpcall(
            Linker linker,
            Arena pluginArena,
            LogCallback logCallback) {

        if (logCallback == null) {
            return MemorySegment.NULL;
        }

        // Define the native callback signature
        // extern "C" fn(level: u8, target: *const c_char, message: *const u8, message_len: usize)
        FunctionDescriptor callbackDescriptor = FunctionDescriptor.ofVoid(
                ValueLayout.JAVA_BYTE,  // level
                ValueLayout.ADDRESS,     // target (null-terminated C string)
                ValueLayout.ADDRESS,     // message (raw bytes)
                ValueLayout.JAVA_LONG    // message_len
        );

        // Create a method handle for our Java callback wrapper
        try {
            MethodHandle wrapperHandle = MethodHandles.lookup().findStatic(
                    FfmPluginLoader.class,
                    "logCallbackWrapper",
                    MethodType.methodType(
                            void.class,
                            LogCallback.class,
                            byte.class,
                            MemorySegment.class,
                            MemorySegment.class,
                            long.class
                    )
            );

            // Bind the LogCallback instance to the wrapper
            MethodHandle boundHandle = wrapperHandle.bindTo(logCallback);

            // Create the upcall stub (persists in plugin arena)
            return linker.upcallStub(boundHandle, callbackDescriptor, pluginArena);

        } catch (NoSuchMethodException | IllegalAccessException e) {
            throw new RuntimeException("Failed to create log callback upcall", e);
        }
    }

    /**
     * Wrapper method that bridges the native callback to the Java LogCallback.
     * <p>
     * This method is called from native code via the upcall stub.
     *
     * @param callback   the Java log callback
     * @param level      the log level (0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error)
     * @param targetPtr  pointer to null-terminated target string
     * @param messagePtr pointer to message bytes
     * @param messageLen length of the message
     */
    private static void logCallbackWrapper(
            LogCallback callback,
            byte level,
            MemorySegment targetPtr,
            MemorySegment messagePtr,
            long messageLen) {

        try {
            // Convert level byte to LogLevel enum
            LogLevel logLevel = LogLevel.fromCode(Byte.toUnsignedInt(level));

            // Extract target as null-terminated C string
            // Need to reinterpret as unbounded segment to read null-terminated string
            String target;
            if (targetPtr.equals(MemorySegment.NULL)) {
                target = "";
            } else {
                MemorySegment unboundedTarget = targetPtr.reinterpret(Long.MAX_VALUE);
                target = unboundedTarget.getUtf8String(0);
            }

            // Extract message from raw bytes
            String message;
            if (messagePtr.equals(MemorySegment.NULL) || messageLen == 0) {
                message = "";
            } else {
                // Reinterpret to the actual message length
                MemorySegment boundedMessage = messagePtr.reinterpret(messageLen);
                byte[] messageBytes = boundedMessage.toArray(ValueLayout.JAVA_BYTE);
                message = new String(messageBytes, StandardCharsets.UTF_8);
            }

            // Invoke the Java callback
            callback.log(logLevel, target, message);

        } catch (Exception e) {
            // Don't let exceptions propagate back to native code
            System.err.println("Error in log callback: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
