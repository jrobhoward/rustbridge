using System.Runtime.InteropServices;
using System.Text;

namespace RustBridge.Native;

/// <summary>
/// Loader for RustBridge plugins using P/Invoke.
/// <para>
/// This loader uses .NET's NativeLibrary API to load and interact with native plugins.
/// </para>
/// <example>
/// <code>
/// using var plugin = NativePluginLoader.Load("libmyplugin.so");
/// var response = plugin.Call("echo", "{\"message\": \"hello\"}");
/// Console.WriteLine(response);
/// </code>
/// </example>
/// </summary>
public static class NativePluginLoader
{
    /// <summary>
    /// Load a plugin from the specified library path.
    /// </summary>
    /// <param name="libraryPath">Path to the shared library.</param>
    /// <returns>The loaded plugin.</returns>
    /// <exception cref="PluginException">If loading fails.</exception>
    public static IPlugin Load(string libraryPath)
    {
        return Load(libraryPath, PluginConfig.Defaults(), null);
    }

    /// <summary>
    /// Load a plugin with configuration.
    /// </summary>
    /// <param name="libraryPath">Path to the shared library.</param>
    /// <param name="config">Plugin configuration.</param>
    /// <returns>The loaded plugin.</returns>
    /// <exception cref="PluginException">If loading fails.</exception>
    public static IPlugin Load(string libraryPath, PluginConfig config)
    {
        return Load(libraryPath, config, null);
    }

    /// <summary>
    /// Load a plugin with configuration and log callback.
    /// </summary>
    /// <param name="libraryPath">Path to the shared library.</param>
    /// <param name="config">Plugin configuration.</param>
    /// <param name="logCallback">Optional callback for log messages.</param>
    /// <returns>The loaded plugin.</returns>
    /// <exception cref="PluginException">If loading fails.</exception>
    public static IPlugin Load(string libraryPath, PluginConfig config, LogCallback? logCallback)
    {
        var library = NativeLibraryHandle.Load(libraryPath);
        GCHandle? callbackHandle = null;

        try
        {
            // Create the plugin instance
            var pluginPtr = library.PluginCreate();
            if (pluginPtr == IntPtr.Zero)
            {
                throw new PluginException("plugin_create returned null");
            }

            // Prepare config
            var configBytes = config.ToJsonBytes();

            // Prepare log callback
            IntPtr logCallbackPtr = IntPtr.Zero;
            if (logCallback != null)
            {
                var nativeCallback = CreateLogCallbackDelegate(logCallback);
                callbackHandle = GCHandle.Alloc(nativeCallback);
                logCallbackPtr = Marshal.GetFunctionPointerForDelegate(nativeCallback);
            }

            // Initialize the plugin
            IntPtr handle;
            unsafe
            {
                fixed (byte* configPtr = configBytes)
                {
                    handle = library.PluginInit(
                        pluginPtr,
                        (IntPtr)configPtr,
                        (nuint)configBytes.Length,
                        logCallbackPtr
                    );
                }
            }

            if (handle == IntPtr.Zero)
            {
                throw new PluginException("plugin_init returned null handle");
            }

            return new NativePlugin(library, handle, logCallback, callbackHandle);
        }
        catch
        {
            callbackHandle?.Free();
            library.Dispose();
            throw;
        }
    }

    /// <summary>
    /// Load a plugin by name, searching in standard library paths.
    /// </summary>
    /// <param name="libraryName">The library name (without lib prefix or extension).</param>
    /// <returns>The loaded plugin.</returns>
    /// <exception cref="PluginException">If loading fails.</exception>
    public static IPlugin LoadByName(string libraryName)
    {
        return LoadByName(libraryName, PluginConfig.Defaults());
    }

    /// <summary>
    /// Load a plugin by name with configuration.
    /// </summary>
    /// <param name="libraryName">The library name (without lib prefix or extension).</param>
    /// <param name="config">Plugin configuration.</param>
    /// <returns>The loaded plugin.</returns>
    /// <exception cref="PluginException">If loading fails.</exception>
    public static IPlugin LoadByName(string libraryName, PluginConfig config)
    {
        var libraryFileName = GetLibraryFileName(libraryName);

        string[] searchPaths =
        [
            ".",
            "./target/release",
            "./target/debug",
            Environment.GetEnvironmentVariable("PATH") ?? ""
        ];

        foreach (var basePath in searchPaths)
        {
            if (string.IsNullOrEmpty(basePath)) continue;

            var fullPath = Path.Combine(basePath, libraryFileName);
            if (File.Exists(fullPath))
            {
                return Load(fullPath, config, null);
            }
        }

        throw new PluginException($"Could not find library: {libraryFileName}");
    }

    private static string GetLibraryFileName(string libraryName)
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            return $"lib{libraryName}.so";
        }
        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            return $"lib{libraryName}.dylib";
        }
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            return $"{libraryName}.dll";
        }

        throw new PluginException($"Unsupported operating system: {RuntimeInformation.OSDescription}");
    }

    private static NativeBindings.LogCallbackDelegate CreateLogCallbackDelegate(LogCallback callback)
    {
        return (level, targetPtr, messagePtr, messageLen) =>
        {
            try
            {
                var logLevel = LogLevelExtensions.FromCode(level);

                var target = targetPtr != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(targetPtr) ?? ""
                    : "";

                var message = messagePtr != IntPtr.Zero && messageLen > 0
                    ? Marshal.PtrToStringUTF8(messagePtr, (int)messageLen) ?? ""
                    : "";

                callback(logLevel, target, message);
            }
            catch (Exception ex)
            {
                // Don't let exceptions propagate back to native code
                Console.Error.WriteLine($"Error in log callback: {ex.Message}");
            }
        };
    }
}
