package com.rustbridge.kotlin

import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

/**
 * Utility for finding the hello_plugin library in tests.
 */
object TestPluginLoader {

    /**
     * Find the hello_plugin shared library.
     *
     * Searches for the library in the Rust target directory (release first, then debug).
     *
     * @return the path to the library
     * @throws IllegalStateException if the library cannot be found
     */
    fun findHelloPluginLibrary(): Path {
        // Determine the library name based on OS
        val libraryName = when {
            System.getProperty("os.name").lowercase().contains("windows") -> "hello_plugin.dll"
            System.getProperty("os.name").lowercase().contains("mac") -> "libhello_plugin.dylib"
            else -> "libhello_plugin.so"
        }

        // Start from current directory and search upward for target/
        var current = Paths.get("").toAbsolutePath()
        repeat(5) {
            val releaseLib = current.resolve("target/release/$libraryName")
            if (Files.exists(releaseLib)) {
                return releaseLib
            }

            val debugLib = current.resolve("target/debug/$libraryName")
            if (Files.exists(debugLib)) {
                return debugLib
            }

            current = current.parent ?: return@repeat
        }

        throw IllegalStateException(
            "Could not find $libraryName. " +
            "Run 'cargo build -p hello-plugin --release' first."
        )
    }
}
