package com.rustbridge.benchmarks;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

/**
 * Helper utilities for JMH benchmarks.
 */
public final class BenchmarkHelper {

    private BenchmarkHelper() {
        // Utility class
    }

    /**
     * Find the hello_plugin shared library.
     *
     * @return path to the plugin library
     * @throws IllegalStateException if the library cannot be found
     */
    public static Path findHelloPluginLibrary() {
        String libraryName = getLibraryName();

        // Start from current directory and search upward
        Path current = Paths.get("").toAbsolutePath();
        for (int i = 0; i < 5; i++) {
            Path releaseLib = current.resolve("target/release/" + libraryName);
            if (Files.exists(releaseLib)) {
                return releaseLib;
            }

            Path debugLib = current.resolve("target/debug/" + libraryName);
            if (Files.exists(debugLib)) {
                return debugLib;
            }

            current = current.getParent();
            if (current == null) {
                break;
            }
        }

        throw new IllegalStateException(
                "Could not find " + libraryName + ". " +
                        "Run 'cargo build -p hello-plugin --release' first."
        );
    }

    private static String getLibraryName() {
        String osName = System.getProperty("os.name").toLowerCase();
        if (osName.contains("windows")) {
            return "hello_plugin.dll";
        } else if (osName.contains("mac")) {
            return "libhello_plugin.dylib";
        } else {
            return "libhello_plugin.so";
        }
    }
}
