package com.rustbridge.ffm;

import com.rustbridge.*;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;

/**
 * FFM-based plugin implementation using Java 21+ Foreign Function & Memory API.
 * <p>
 * This implementation uses Project Panama to call native plugin functions directly,
 * without JNI overhead.
 *
 * <h2>Thread Safety</h2>
 * This class is fully thread-safe and supports concurrent calls from multiple threads.
 * Each call creates its own {@link Arena} for temporary memory allocations, ensuring
 * no contention between threads. The Rust plugin implementation is also thread-safe
 * (Send + Sync), allowing true concurrent execution.
 *
 * <h2>Memory Management</h2>
 * The plugin uses two types of arenas:
 * <ul>
 *   <li><b>Plugin-lifetime arena</b>: Created at load time using {@link Arena#ofShared()},
 *       holds library symbols and upcall stubs, closed when plugin is closed. Shared to
 *       allow concurrent access to the plugin handle.</li>
 *   <li><b>Per-call arenas</b>: Created for each {@link #call} or {@link #callRaw} using
 *       {@link Arena#ofShared()}, holds request/response data, automatically closed when
 *       call completes.</li>
 * </ul>
 * <p>
 * Both arena types use {@link Arena#ofShared()} to support concurrent access from multiple
 * threads. This architecture eliminates synchronization bottlenecks while maintaining FFM
 * safety through proper arena lifecycle management.
 *
 * @see FfmPluginLoader
 * @see Arena
 */
public class FfmPlugin implements Plugin {
    private static final Gson GSON = new GsonBuilder().create();

    private final Arena pluginArena;
    private final MemorySegment handle;
    private final NativeBindings bindings;
    private final LogCallback logCallback;

    private volatile boolean closed = false;

    /**
     * Create a new FFM plugin wrapper.
     *
     * @param pluginArena the memory arena for this plugin's lifetime (library symbols, upcall stub)
     * @param handle      the native plugin handle
     * @param bindings    the native function bindings
     * @param logCallback optional log callback
     */
    FfmPlugin(Arena pluginArena, MemorySegment handle, NativeBindings bindings, LogCallback logCallback) {
        this.pluginArena = pluginArena;
        this.handle = handle;
        this.bindings = bindings;
        this.logCallback = logCallback;
    }

    @Override
    public LifecycleState getState() {
        checkNotClosed();
        try {
            byte stateCode = (byte) bindings.pluginGetState().invokeExact(handle);
            int stateCodeInt = Byte.toUnsignedInt(stateCode);
            if (stateCodeInt == 255) {
                throw new IllegalStateException("Invalid plugin handle");
            }
            return LifecycleState.fromCode(stateCodeInt);
        } catch (Throwable t) {
            throw new RuntimeException("Failed to get plugin state", t);
        }
    }

    @Override
    public String call(String typeTag, String request) throws PluginException {
        checkNotClosed();

        // Create per-call arena for temporary allocations (shared for FFM compatibility)
        try (Arena callArena = Arena.ofShared()) {
            // Allocate type tag as null-terminated string
            MemorySegment typeTagSegment = callArena.allocateUtf8String(typeTag);

            // Allocate request data
            byte[] requestBytes = request.getBytes(StandardCharsets.UTF_8);
            MemorySegment requestSegment = callArena.allocate(requestBytes.length);
            requestSegment.copyFrom(MemorySegment.ofArray(requestBytes));

            // Call the plugin - use callArena as SegmentAllocator for return struct
            MemorySegment resultBuffer = (MemorySegment) bindings.pluginCall().invoke(
                    callArena,  // SegmentAllocator for return value
                    handle,
                    typeTagSegment,
                    requestSegment,
                    (long) requestBytes.length
            );

            // Parse the result buffer (copies data to Java heap, then frees native buffer)
            return parseResultBuffer(resultBuffer);
        } catch (PluginException e) {
            throw e;
        } catch (Throwable t) {
            throw new PluginException("Native call failed", t);
        }
        // callArena automatically closed here
    }

    @Override
    public <T, R> R call(String typeTag, T request, Class<R> responseType) throws PluginException {
        String requestJson = GSON.toJson(request);
        String responseJson = call(typeTag, requestJson);
        return GSON.fromJson(responseJson, responseType);
    }

    /**
     * Call the plugin with a binary struct request (raw binary transport).
     * <p>
     * This method bypasses JSON serialization for high-performance scenarios.
     * The request and response are fixed-size C structs.
     *
     * @param messageId     the binary message ID (registered with register_binary_handler)
     * @param request       the request struct
     * @param responseSize  expected response size in bytes
     * @return a memory segment containing the response data
     * @throws PluginException if the call fails
     */
    public MemorySegment callRaw(int messageId, BinaryStruct request, long responseSize)
            throws PluginException {
        checkNotClosed();

        // Create per-call arena for temporary allocations (shared for FFM compatibility)
        try (Arena callArena = Arena.ofShared()) {
            // Call the plugin with raw binary data
            MemorySegment resultBuffer = (MemorySegment) bindings.pluginCallRaw().invoke(
                    callArena,  // SegmentAllocator for return value
                    handle,
                    messageId,
                    request.segment(),
                    request.byteSize()
            );

            // Parse and validate the response (copies to plugin arena, then frees native buffer)
            return parseRawResultBuffer(resultBuffer, responseSize);
        } catch (PluginException e) {
            throw e;
        } catch (Throwable t) {
            throw new PluginException("Native raw call failed", t);
        }
        // callArena automatically closed here
    }

    /**
     * Parse the RbResponse buffer from a raw plugin call.
     */
    private MemorySegment parseRawResultBuffer(MemorySegment responseStruct, long expectedSize)
            throws PluginException {
        // RbResponse layout: { error_code: u32, len: u32, capacity: u32, _padding: u32, data: *mut c_void }
        int errorCode = responseStruct.get(ValueLayout.JAVA_INT, 0);
        int len = responseStruct.get(ValueLayout.JAVA_INT, 4);
        MemorySegment data = responseStruct.get(ValueLayout.ADDRESS, 16);

        try {
            if (errorCode != 0) {
                // Error case - data contains error message
                String errorMessage = "Unknown error";
                if (!data.equals(MemorySegment.NULL) && len > 0) {
                    MemorySegment slice = data.reinterpret(len);
                    errorMessage = new String(slice.toArray(ValueLayout.JAVA_BYTE),
                            java.nio.charset.StandardCharsets.UTF_8);
                }
                throw new PluginException(errorCode, errorMessage);
            }

            // Success case - copy response data to a new segment
            if (data.equals(MemorySegment.NULL) || len == 0) {
                throw new PluginException("Empty response from raw call");
            }

            if (len != expectedSize) {
                throw new PluginException(String.format(
                        "Response size mismatch: expected %d, got %d", expectedSize, len));
            }

            // Copy response data to plugin arena (persists until plugin close)
            MemorySegment responseData = pluginArena.allocate(len);
            MemorySegment sourceSlice = data.reinterpret(len);
            responseData.copyFrom(sourceSlice);

            return responseData;
        } finally {
            // Free the response buffer
            freeRawResponse(responseStruct);
        }
    }

    /**
     * Free a raw response buffer.
     */
    private void freeRawResponse(MemorySegment responseStruct) {
        try {
            bindings.rbResponseFree().invokeExact(responseStruct);
        } catch (Throwable t) {
            System.err.println("Warning: Failed to free raw response: " + t.getMessage());
        }
    }

    @Override
    public void setLogLevel(LogLevel level) {
        checkNotClosed();
        try {
            bindings.pluginSetLogLevel().invokeExact(handle, (byte) level.getCode());
        } catch (Throwable t) {
            throw new RuntimeException("Failed to set log level", t);
        }
    }

    @Override
    public long getRejectedRequestCount() {
        checkNotClosed();
        try {
            return (long) bindings.pluginGetRejectedCount().invokeExact(handle);
        } catch (Throwable t) {
            throw new RuntimeException("Failed to get rejected request count", t);
        }
    }

    @Override
    public void close() {
        if (closed) {
            return;
        }
        closed = true;

        try {
            boolean success = (boolean) bindings.pluginShutdown().invokeExact(handle);
            if (!success) {
                System.err.println("Warning: Plugin shutdown returned false");
            }
        } catch (Throwable t) {
            System.err.println("Warning: Exception during plugin shutdown: " + t.getMessage());
        }

        // Close the plugin-lifetime arena (releases all allocated memory)
        pluginArena.close();
    }

    /**
     * Parse the result buffer from a plugin call.
     */
    private String parseResultBuffer(MemorySegment bufferStruct) throws PluginException {
        // FfiBuffer layout: { data: *mut u8, len: usize, capacity: usize, error_code: u32 }
        // On 64-bit: data(8) + len(8) + capacity(8) + error_code(4) = 28 bytes (aligned to 32)

        MemorySegment data = bufferStruct.get(ValueLayout.ADDRESS, 0);
        long len = bufferStruct.get(ValueLayout.JAVA_LONG, 8);
        int errorCode = bufferStruct.get(ValueLayout.JAVA_INT, 24);

        try {
            if (errorCode != 0) {
                // Error case - data contains error message
                String errorMessage = "Unknown error";
                if (!data.equals(MemorySegment.NULL) && len > 0) {
                    MemorySegment slice = data.reinterpret(len);
                    errorMessage = new String(slice.toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
                }
                throw new PluginException(errorCode, errorMessage);
            }

            // Success case - parse response envelope
            if (data.equals(MemorySegment.NULL) || len == 0) {
                return "null";
            }

            MemorySegment slice = data.reinterpret(len);
            String responseJson = new String(slice.toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);

            // Parse envelope and extract payload
            ResponseEnvelope envelope = ResponseEnvelope.fromJson(responseJson);
            if (!envelope.isSuccess()) {
                throw envelope.toException();
            }

            return envelope.getPayloadJson();
        } finally {
            // Free the buffer
            freeBuffer(bufferStruct);
        }
    }

    /**
     * Free a result buffer.
     */
    private void freeBuffer(MemorySegment bufferStruct) {
        try {
            bindings.pluginFreeBuffer().invokeExact(bufferStruct);
        } catch (Throwable t) {
            System.err.println("Warning: Failed to free buffer: " + t.getMessage());
        }
    }

    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("Plugin has been closed");
        }
    }
}
