using System.Runtime.InteropServices;

namespace RustBridge.Native;

/// <summary>
/// Holds references to a loaded native library and its function pointers.
/// </summary>
internal sealed class NativeLibraryHandle : IDisposable
{
    private IntPtr _libraryHandle;
    private bool _disposed;

    public NativeBindings.PluginCreateDelegate PluginCreate { get; }
    public NativeBindings.PluginInitDelegate PluginInit { get; }
    public NativeBindings.PluginCallDelegate PluginCall { get; }
    public NativeBindings.PluginCallRawDelegate? PluginCallRaw { get; }  // nullable - binary transport optional
    public NativeBindings.PluginFreeBufferDelegate PluginFreeBuffer { get; }
    public NativeBindings.RbResponseFreeDelegate? RbResponseFree { get; }  // nullable - binary transport optional
    public NativeBindings.PluginShutdownDelegate PluginShutdown { get; }
    public NativeBindings.PluginSetLogLevelDelegate PluginSetLogLevel { get; }
    public NativeBindings.PluginGetStateDelegate PluginGetState { get; }
    public NativeBindings.PluginGetRejectedCountDelegate PluginGetRejectedCount { get; }

    /// <summary>
    /// Check if binary transport is supported by this library.
    /// </summary>
    public bool HasBinaryTransport => PluginCallRaw != null && RbResponseFree != null;

    private NativeLibraryHandle(
        IntPtr libraryHandle,
        NativeBindings.PluginCreateDelegate pluginCreate,
        NativeBindings.PluginInitDelegate pluginInit,
        NativeBindings.PluginCallDelegate pluginCall,
        NativeBindings.PluginCallRawDelegate? pluginCallRaw,
        NativeBindings.PluginFreeBufferDelegate pluginFreeBuffer,
        NativeBindings.RbResponseFreeDelegate? rbResponseFree,
        NativeBindings.PluginShutdownDelegate pluginShutdown,
        NativeBindings.PluginSetLogLevelDelegate pluginSetLogLevel,
        NativeBindings.PluginGetStateDelegate pluginGetState,
        NativeBindings.PluginGetRejectedCountDelegate pluginGetRejectedCount)
    {
        _libraryHandle = libraryHandle;
        PluginCreate = pluginCreate;
        PluginInit = pluginInit;
        PluginCall = pluginCall;
        PluginCallRaw = pluginCallRaw;
        PluginFreeBuffer = pluginFreeBuffer;
        RbResponseFree = rbResponseFree;
        PluginShutdown = pluginShutdown;
        PluginSetLogLevel = pluginSetLogLevel;
        PluginGetState = pluginGetState;
        PluginGetRejectedCount = pluginGetRejectedCount;
    }

    /// <summary>
    /// Load a native library and resolve all required function pointers.
    /// </summary>
    /// <param name="libraryPath">Path to the native library.</param>
    /// <returns>A handle to the loaded library.</returns>
    /// <exception cref="PluginException">If loading fails.</exception>
    public static NativeLibraryHandle Load(string libraryPath)
    {
        if (!NativeLibrary.TryLoad(libraryPath, out var handle))
        {
            throw new PluginException($"Failed to load native library: {libraryPath}");
        }

        try
        {
            return new NativeLibraryHandle(
                handle,
                GetDelegate<NativeBindings.PluginCreateDelegate>(handle, "plugin_create"),
                GetDelegate<NativeBindings.PluginInitDelegate>(handle, "plugin_init"),
                GetDelegate<NativeBindings.PluginCallDelegate>(handle, "plugin_call"),
                TryGetDelegate<NativeBindings.PluginCallRawDelegate>(handle, "plugin_call_raw"),  // optional
                GetDelegate<NativeBindings.PluginFreeBufferDelegate>(handle, "plugin_free_buffer"),
                TryGetDelegate<NativeBindings.RbResponseFreeDelegate>(handle, "rb_response_free"),  // optional
                GetDelegate<NativeBindings.PluginShutdownDelegate>(handle, "plugin_shutdown"),
                GetDelegate<NativeBindings.PluginSetLogLevelDelegate>(handle, "plugin_set_log_level"),
                GetDelegate<NativeBindings.PluginGetStateDelegate>(handle, "plugin_get_state"),
                GetDelegate<NativeBindings.PluginGetRejectedCountDelegate>(handle, "plugin_get_rejected_count")
            );
        }
        catch
        {
            NativeLibrary.Free(handle);
            throw;
        }
    }

    private static TDelegate GetDelegate<TDelegate>(IntPtr libraryHandle, string functionName)
        where TDelegate : Delegate
    {
        if (!NativeLibrary.TryGetExport(libraryHandle, functionName, out var functionPtr))
        {
            throw new PluginException($"Function not found: {functionName}");
        }
        return Marshal.GetDelegateForFunctionPointer<TDelegate>(functionPtr);
    }

    private static TDelegate? TryGetDelegate<TDelegate>(IntPtr libraryHandle, string functionName)
        where TDelegate : Delegate
    {
        if (!NativeLibrary.TryGetExport(libraryHandle, functionName, out var functionPtr))
        {
            return null;
        }
        return Marshal.GetDelegateForFunctionPointer<TDelegate>(functionPtr);
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        if (_libraryHandle != IntPtr.Zero)
        {
            NativeLibrary.Free(_libraryHandle);
            _libraryHandle = IntPtr.Zero;
        }
    }
}
