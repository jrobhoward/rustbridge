using System.Runtime.InteropServices;

namespace RustBridge.Native;

/// <summary>
/// Native function bindings for RustBridge FFI.
/// <para>
/// This class provides P/Invoke declarations for calling native plugin functions.
/// The actual library is loaded dynamically via <see cref="NativePluginLoader"/>.
/// </para>
/// </summary>
internal static class NativeBindings
{
    /// <summary>
    /// FfiBuffer structure returned by plugin_call.
    /// <code>
    /// struct FfiBuffer {
    ///     data: *mut u8,      // pointer to data
    ///     len: usize,         // length of data
    ///     capacity: usize,    // allocation capacity
    ///     error_code: u32     // 0 = success
    /// }
    /// </code>
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct FfiBuffer
    {
        public IntPtr Data;
        public nuint Len;
        public nuint Capacity;
        public uint ErrorCode;
    }

    /// <summary>
    /// RbResponse structure returned by plugin_call_raw (binary transport).
    /// <code>
    /// struct RbResponse {
    ///     error_code: u32,  // 0 = success
    ///     len: u32,         // response data size
    ///     capacity: u32,    // allocation capacity
    ///     _padding: u32,    // alignment padding
    ///     data: *mut c_void // response data pointer
    /// }
    /// </code>
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct RbResponse
    {
        public uint ErrorCode;
        public uint Len;
        public uint Capacity;
        private readonly uint _padding;
        public IntPtr Data;
    }

    /// <summary>
    /// Delegate type for the log callback function.
    /// </summary>
    /// <param name="level">Log level (0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error).</param>
    /// <param name="target">Pointer to null-terminated target string.</param>
    /// <param name="message">Pointer to message bytes.</param>
    /// <param name="messageLen">Length of the message.</param>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void LogCallbackDelegate(byte level, IntPtr target, IntPtr message, nuint messageLen);

    /// <summary>
    /// Create a new plugin instance.
    /// </summary>
    /// <returns>Pointer to the plugin instance.</returns>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr PluginCreateDelegate();

    /// <summary>
    /// Initialize a plugin with configuration.
    /// </summary>
    /// <param name="pluginPtr">Pointer from plugin_create.</param>
    /// <param name="configJson">Pointer to config JSON bytes.</param>
    /// <param name="configLen">Length of config JSON.</param>
    /// <param name="logCallback">Optional log callback (can be null).</param>
    /// <returns>Handle to the initialized plugin, or null on failure.</returns>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr PluginInitDelegate(IntPtr pluginPtr, IntPtr configJson, nuint configLen, IntPtr logCallback);

    /// <summary>
    /// Call the plugin with a JSON request.
    /// </summary>
    /// <param name="handle">Plugin handle from plugin_init.</param>
    /// <param name="typeTag">Null-terminated type tag string.</param>
    /// <param name="request">Pointer to request bytes.</param>
    /// <param name="requestLen">Length of request.</param>
    /// <returns>FfiBuffer containing the response.</returns>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate FfiBuffer PluginCallDelegate(IntPtr handle, IntPtr typeTag, IntPtr request, nuint requestLen);

    /// <summary>
    /// Call the plugin with a binary request (raw transport).
    /// </summary>
    /// <param name="handle">Plugin handle from plugin_init.</param>
    /// <param name="messageId">Binary message ID.</param>
    /// <param name="request">Pointer to request struct.</param>
    /// <param name="requestSize">Size of request struct.</param>
    /// <returns>RbResponse containing the response.</returns>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate RbResponse PluginCallRawDelegate(IntPtr handle, int messageId, IntPtr request, nuint requestSize);

    /// <summary>
    /// Free a buffer returned by plugin_call.
    /// </summary>
    /// <param name="buffer">Pointer to the FfiBuffer.</param>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void PluginFreeBufferDelegate(IntPtr buffer);

    /// <summary>
    /// Free a response returned by plugin_call_raw.
    /// </summary>
    /// <param name="response">Pointer to the RbResponse.</param>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void RbResponseFreeDelegate(IntPtr response);

    /// <summary>
    /// Shutdown the plugin.
    /// </summary>
    /// <param name="handle">Plugin handle.</param>
    /// <returns>True if shutdown was successful.</returns>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.U1)]
    public delegate bool PluginShutdownDelegate(IntPtr handle);

    /// <summary>
    /// Set the log level.
    /// </summary>
    /// <param name="handle">Plugin handle.</param>
    /// <param name="level">Log level code.</param>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void PluginSetLogLevelDelegate(IntPtr handle, byte level);

    /// <summary>
    /// Get the plugin state.
    /// </summary>
    /// <param name="handle">Plugin handle.</param>
    /// <returns>State code (0-5), or 255 for invalid handle.</returns>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate byte PluginGetStateDelegate(IntPtr handle);

    /// <summary>
    /// Get the count of rejected requests.
    /// </summary>
    /// <param name="handle">Plugin handle.</param>
    /// <returns>Number of rejected requests.</returns>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate ulong PluginGetRejectedCountDelegate(IntPtr handle);
}
