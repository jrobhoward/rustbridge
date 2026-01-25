using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace RustBridge.Native;

/// <summary>
/// P/Invoke-based plugin implementation.
/// <para>
/// This implementation uses .NET's P/Invoke to call native plugin functions directly.
/// </para>
/// <para>
/// <b>Thread Safety</b>: This class is thread-safe. The underlying Rust plugin
/// implementation is also thread-safe (Send + Sync), allowing true concurrent execution.
/// </para>
/// </summary>
public sealed class NativePlugin : IPlugin
{
    private readonly NativeLibraryHandle _library;
    private readonly IntPtr _handle;
    private readonly LogCallback? _logCallback;
    private readonly GCHandle? _callbackHandle;
    private volatile bool _disposed;

    internal NativePlugin(
        NativeLibraryHandle library,
        IntPtr handle,
        LogCallback? logCallback,
        GCHandle? callbackHandle)
    {
        _library = library;
        _handle = handle;
        _logCallback = logCallback;
        _callbackHandle = callbackHandle;
    }

    /// <inheritdoc/>
    public LifecycleState State
    {
        get
        {
            if (_disposed)
            {
                return LifecycleState.Stopped;
            }

            var stateCode = _library.PluginGetState(_handle);
            if (stateCode == 255)
            {
                throw new InvalidOperationException("Invalid plugin handle");
            }
            return LifecycleStateExtensions.FromCode(stateCode);
        }
    }

    /// <inheritdoc/>
    public string Call(string typeTag, string request)
    {
        ThrowIfDisposed();

        var typeTagBytes = Encoding.UTF8.GetBytes(typeTag + '\0');
        var requestBytes = Encoding.UTF8.GetBytes(request);

        unsafe
        {
            fixed (byte* typeTagPtr = typeTagBytes)
            fixed (byte* requestPtr = requestBytes)
            {
                var buffer = _library.PluginCall(
                    _handle,
                    (IntPtr)typeTagPtr,
                    (IntPtr)requestPtr,
                    (nuint)requestBytes.Length
                );

                return ParseResultBuffer(buffer);
            }
        }
    }

    /// <inheritdoc/>
    public TResponse Call<TRequest, TResponse>(string typeTag, TRequest request)
    {
        var requestJson = JsonSerializer.Serialize(request);
        var responseJson = Call(typeTag, requestJson);
        return JsonSerializer.Deserialize<TResponse>(responseJson)
            ?? throw new PluginException("Failed to deserialize response");
    }

    /// <inheritdoc/>
    public void SetLogLevel(LogLevel level)
    {
        ThrowIfDisposed();
        _library.PluginSetLogLevel(_handle, (byte)level);
    }

    /// <inheritdoc/>
    public long RejectedRequestCount
    {
        get
        {
            ThrowIfDisposed();
            return (long)_library.PluginGetRejectedCount(_handle);
        }
    }

    /// <inheritdoc/>
    public TResponse CallRaw<TRequest, TResponse>(int messageId, TRequest request)
        where TRequest : unmanaged, IBinaryStruct
        where TResponse : unmanaged, IBinaryStruct
    {
        ThrowIfDisposed();

        unsafe
        {
            // Get pointer to request struct
            var requestPtr = (IntPtr)Unsafe.AsPointer(ref request);

            var response = _library.PluginCallRaw(
                _handle,
                messageId,
                requestPtr,
                (nuint)request.ByteSize
            );

            return ParseRawResponse<TResponse>(response);
        }
    }

    private unsafe TResponse ParseRawResponse<TResponse>(NativeBindings.RbResponse response)
        where TResponse : unmanaged, IBinaryStruct
    {
        try
        {
            if (response.ErrorCode != 0)
            {
                var errorMessage = "Unknown error";
                if (response.Data != IntPtr.Zero && response.Len > 0)
                {
                    errorMessage = Marshal.PtrToStringUTF8(response.Data, (int)response.Len) ?? errorMessage;
                }
                throw new PluginException((int)response.ErrorCode, errorMessage);
            }

            if (response.Data == IntPtr.Zero || response.Len == 0)
            {
                throw new PluginException("Empty response from raw call");
            }

            var expectedSize = Unsafe.SizeOf<TResponse>();
            if (response.Len != expectedSize)
            {
                throw new PluginException($"Response size mismatch: expected {expectedSize}, got {response.Len}");
            }

            // Copy response data to managed struct
            TResponse result = default;
            var resultPtr = (IntPtr)Unsafe.AsPointer(ref result);
            Buffer.MemoryCopy((void*)response.Data, (void*)resultPtr, expectedSize, expectedSize);

            return result;
        }
        finally
        {
            FreeRawResponse(response);
        }
    }

    private unsafe void FreeRawResponse(NativeBindings.RbResponse response)
    {
        try
        {
            var responsePtr = (IntPtr)Unsafe.AsPointer(ref response);
            _library.RbResponseFree(responsePtr);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Warning: Failed to free raw response: {ex.Message}");
        }
    }

    /// <inheritdoc/>
    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        try
        {
            var success = _library.PluginShutdown(_handle);
            if (!success)
            {
                Console.Error.WriteLine("Warning: Plugin shutdown returned false");
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Warning: Exception during plugin shutdown: {ex.Message}");
        }

        // Free the GC handle for the callback delegate
        _callbackHandle?.Free();

        // Free the native library
        _library.Dispose();
    }

    private string ParseResultBuffer(NativeBindings.FfiBuffer buffer)
    {
        try
        {
            if (buffer.ErrorCode != 0)
            {
                var errorMessage = "Unknown error";
                if (buffer.Data != IntPtr.Zero && buffer.Len > 0)
                {
                    errorMessage = Marshal.PtrToStringUTF8(buffer.Data, (int)buffer.Len) ?? errorMessage;
                }
                throw new PluginException((int)buffer.ErrorCode, errorMessage);
            }

            if (buffer.Data == IntPtr.Zero || buffer.Len == 0)
            {
                return "null";
            }

            var responseJson = Marshal.PtrToStringUTF8(buffer.Data, (int)buffer.Len)
                ?? throw new PluginException("Failed to read response");

            var envelope = ResponseEnvelope.FromJson(responseJson);
            if (!envelope.IsSuccess)
            {
                throw envelope.ToException();
            }

            return envelope.GetPayloadJson();
        }
        finally
        {
            FreeBuffer(buffer);
        }
    }

    private void FreeBuffer(NativeBindings.FfiBuffer buffer)
    {
        try
        {
            unsafe
            {
                // We need to pass a pointer to the buffer struct
                var bufferPtr = (IntPtr)(&buffer);
                _library.PluginFreeBuffer(bufferPtr);
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Warning: Failed to free buffer: {ex.Message}");
        }
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
        {
            throw new ObjectDisposedException(nameof(NativePlugin), "Plugin has been closed");
        }
    }
}
