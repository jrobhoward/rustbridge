package com.rustbridge.jni;

import java.nio.file.Path;
import java.nio.file.Paths;

import static org.junit.jupiter.api.Assertions.fail;

/**
 * Utility class for locating test plugin libraries.
 * <p>
 * Centralizes the logic for finding compiled plugin libraries across different
 * platforms and build configurations (debug/release).
 */
public final class TestPluginLoader {

    private TestPluginLoader() {
        // Utility class, no instantiation
    }

    /**
     * Find the hello-plugin library for the current platform.
     * <p>
     * Searches in target/release first, then target/debug.
     *
     * @return the absolute path to the hello-plugin library
     * @throws AssertionError if the library cannot be found
     */
    public static Path findHelloPluginLibrary() {
        return findLibrary("hello_plugin");
    }

    /**
     * Find a plugin library by name for the current platform.
     * <p>
     * The library name should be the base name without platform-specific prefix/suffix.
     * For example, "hello_plugin" will find:
     * <ul>
     *   <li>libhello_plugin.so on Linux</li>
     *   <li>libhello_plugin.dylib on macOS</li>
     *   <li>hello_plugin.dll on Windows</li>
     * </ul>
     * <p>
     * Searches in target/release first, then target/debug.
     *
     * @param libraryBaseName the base name of the library (e.g., "hello_plugin")
     * @return the absolute path to the library
     * @throws AssertionError if the library cannot be found
     */
    public static Path findLibrary(String libraryBaseName) {
        String libraryName = getPlatformLibraryName(libraryBaseName);

        // Look in target/release first, then target/debug
        // Path is relative to rustbridge-java/rustbridge-jni
        Path releasePath = Paths.get("../../target/release").resolve(libraryName);
        Path debugPath = Paths.get("../../target/debug").resolve(libraryName);

        if (releasePath.toFile().exists()) {
            return releasePath.toAbsolutePath();
        } else if (debugPath.toFile().exists()) {
            return debugPath.toAbsolutePath();
        } else {
            fail("Could not find " + libraryBaseName + " library. " +
                    "Build it with: cargo build -p " + libraryBaseName.replace('_', '-') + " --release");
            return null; // Never reached, fail() throws
        }
    }

    /**
     * Get the platform-specific library filename.
     *
     * @param baseName the base library name (e.g., "hello_plugin")
     * @return the platform-specific filename (e.g., "libhello_plugin.so")
     */
    public static String getPlatformLibraryName(String baseName) {
        String osName = System.getProperty("os.name").toLowerCase();

        if (osName.contains("linux")) {
            return "lib" + baseName + ".so";
        } else if (osName.contains("mac") || osName.contains("darwin")) {
            return "lib" + baseName + ".dylib";
        } else if (osName.contains("windows")) {
            return baseName + ".dll";
        } else {
            throw new RuntimeException("Unsupported OS: " + osName);
        }
    }

    /**
     * Get the current platform identifier.
     *
     * @return platform string like "linux", "darwin", or "windows"
     */
    public static String getCurrentPlatform() {
        String osName = System.getProperty("os.name").toLowerCase();

        if (osName.contains("linux")) {
            return "linux";
        } else if (osName.contains("mac") || osName.contains("darwin")) {
            return "darwin";
        } else if (osName.contains("windows")) {
            return "windows";
        } else {
            throw new RuntimeException("Unsupported OS: " + osName);
        }
    }
}
