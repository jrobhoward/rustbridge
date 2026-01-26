namespace RustBridge.Benchmarks;

/// <summary>
/// Helper class for finding the hello-plugin library.
/// </summary>
public static class BenchmarkHelper
{
    /// <summary>
    /// Find the hello-plugin library, searching in common locations.
    /// </summary>
    /// <returns>Full path to the library, or null if not found.</returns>
    public static string? FindHelloPlugin()
    {
        var libraryName = GetLibraryFileName("hello_plugin");
        var assemblyLocation = typeof(BenchmarkHelper).Assembly.Location;
        var assemblyDir = Path.GetDirectoryName(assemblyLocation) ?? ".";

        // Search from current directory
        var result = SearchForPlugin(Environment.CurrentDirectory, libraryName);
        if (result != null) return result;

        // Search from assembly location, walking up the directory tree
        // BenchmarkDotNet copies assemblies to deep temp paths like:
        // bin\Release\net8.0\{guid}\bin\Release\net8.0
        // So we need to walk up many levels to find the repo root
        var searchDir = assemblyDir;
        for (int i = 0; i < 15; i++)
        {
            result = SearchForPlugin(searchDir, libraryName);
            if (result != null) return result;

            var parent = Path.GetDirectoryName(searchDir);
            if (parent == null || parent == searchDir) break;
            searchDir = parent;
        }

        return null;
    }

    private static string? SearchForPlugin(string baseDir, string libraryName)
    {
        // Prefer release build for benchmarks
        var releasePath = Path.Combine(baseDir, "target", "release", libraryName);
        if (File.Exists(releasePath))
        {
            return Path.GetFullPath(releasePath);
        }

        var debugPath = Path.Combine(baseDir, "target", "debug", libraryName);
        if (File.Exists(debugPath))
        {
            return Path.GetFullPath(debugPath);
        }

        return null;
    }

    /// <summary>
    /// Get the hello-plugin library path or throw if not found.
    /// </summary>
    public static string GetHelloPluginOrThrow()
    {
        return FindHelloPlugin()
            ?? throw new InvalidOperationException(
                "hello-plugin not found. Run: cargo build --release -p hello-plugin");
    }

    private static string GetLibraryFileName(string name)
    {
        if (OperatingSystem.IsWindows()) return $"{name}.dll";
        if (OperatingSystem.IsMacOS()) return $"lib{name}.dylib";
        return $"lib{name}.so";
    }
}
