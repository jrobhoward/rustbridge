package com.rustbridge;

import org.jetbrains.annotations.NotNull;

/**
 * Platform detection utilities for native library loading.
 * <p>
 * Provides methods to determine platform-specific library names and search paths.
 */
public final class PlatformUtil {

    private static final String OS_NAME = System.getProperty("os.name").toLowerCase();
    private static final boolean IS_LINUX = OS_NAME.contains("linux");
    private static final boolean IS_MAC = OS_NAME.contains("mac") || OS_NAME.contains("darwin");
    private static final boolean IS_WINDOWS = OS_NAME.contains("windows");

    private PlatformUtil() {
        // Utility class
    }

    /**
     * Check if the current platform is Linux.
     *
     * @return true if running on Linux
     */
    public static boolean isLinux() {
        return IS_LINUX;
    }

    /**
     * Check if the current platform is macOS.
     *
     * @return true if running on macOS
     */
    public static boolean isMac() {
        return IS_MAC;
    }

    /**
     * Check if the current platform is Windows.
     *
     * @return true if running on Windows
     */
    public static boolean isWindows() {
        return IS_WINDOWS;
    }

    /**
     * Get the platform-specific library file name for a given base name.
     * <p>
     * Adds the appropriate prefix and extension for the current platform:
     * <ul>
     *   <li>Linux: lib{name}.so</li>
     *   <li>macOS: lib{name}.dylib</li>
     *   <li>Windows: {name}.dll</li>
     * </ul>
     *
     * @param libraryName the base library name (without prefix or extension)
     * @return the platform-specific library file name
     * @throws PluginException if the platform is not supported
     */
    public static @NotNull String getLibraryFileName(@NotNull String libraryName) throws PluginException {
        if (IS_LINUX) {
            return "lib" + libraryName + ".so";
        } else if (IS_MAC) {
            return "lib" + libraryName + ".dylib";
        } else if (IS_WINDOWS) {
            return libraryName + ".dll";
        } else {
            throw new PluginException("Unsupported operating system: " + OS_NAME);
        }
    }

    /**
     * Get the default search paths for native libraries.
     * <p>
     * Returns paths commonly used during development:
     * <ul>
     *   <li>Current directory</li>
     *   <li>./target/release (Rust release builds)</li>
     *   <li>./target/debug (Rust debug builds)</li>
     *   <li>java.library.path system property</li>
     * </ul>
     *
     * @return array of search paths
     */
    public static @NotNull String[] getDefaultSearchPaths() {
        return new String[] {
            ".",
            "./target/release",
            "./target/debug",
            System.getProperty("java.library.path", "")
        };
    }
}
