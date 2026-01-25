package com.rustbridge.ffm;

import com.rustbridge.*;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;

/**
 * FFM-based plugin implementation using Java 21+ Foreign Function and Memory API.
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
 *       holds library symbols and upcall stubs, closed when plugin is closed.</li>
 *   <li><b>Per-call arenas</b>: Created for each {@link #call} using {@link Arena#ofConfined()},
 *       which is faster than shared arenas since no synchronization is needed.</li>
 * </ul>
 *
 * <h2>Performance Tips</h2>
 * For maximum binary transport performance, use {@link #callRaw(int, BinaryStruct, long, Arena)}
 * with a caller-managed arena to avoid per-call arena creation overhead. For even simpler
 * high-performance code, use {@link #callRawBytes(int, BinaryStruct)} which returns a byte array.
 *
 * @see FfmPluginLoader
 * @see Arena
 */
public class FfmPlugin implements Plugin {
    private static final ObjectMapper OBJECT_MAPPER = JsonMapper.getInstance();

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
    FfmPlugin(@NotNull Arena pluginArena, @NotNull MemorySegment handle, @NotNull NativeBindings bindings, @Nullable LogCallback logCallback) {
        this.pluginArena = pluginArena;
        this.handle = handle;
        this.bindings = bindings;
        this.logCallback = logCallback;
    }

    @Override
    public @NotNull LifecycleState getState() {
        // After close, return STOPPED instead of throwing
        if (closed) {
            return LifecycleState.STOPPED;
        }
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
    public @NotNull String call(@NotNull String typeTag, @NotNull String request) throws PluginException {
        if (closed) {
            throw new PluginException(1, "Plugin has been closed");
        }

        // Use confined arena - faster than shared, safe since we only use it in this thread
        try (Arena callArena = Arena.ofConfined()) {
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
    }

    @Override
    public <T, R> @NotNull R call(@NotNull String typeTag, @NotNull T request, @NotNull Class<R> responseType) throws PluginException {
        String requestJson;
        try {
            requestJson = OBJECT_MAPPER.writeValueAsString(request);
        } catch (JsonProcessingException e) {
            throw new RuntimeException("Failed to serialize request", e);
        }
        String responseJson = call(typeTag, requestJson);
        try {
            return OBJECT_MAPPER.readValue(responseJson, responseType);
        } catch (JsonProcessingException e) {
            throw new RuntimeException("Failed to deserialize response", e);
        }
    }

    /**
     * Call the plugin with a binary struct request (raw binary transport).
     * <p>
     * This method bypasses JSON serialization for high-performance scenarios.
     * The request and response are fixed-size C structs.
     * <p>
     * <b>Note:</b> This method allocates response data in the plugin's arena, which
     * accumulates until the plugin is closed. For high-frequency calls, prefer
     * {@link #callRaw(int, BinaryStruct, long, Arena)} or {@link #callRawBytes(int, BinaryStruct)}.
     *
     * @param messageId    the binary message ID (registered with register_binary_handler)
     * @param request      the request struct
     * @param responseSize expected response size in bytes
     * @return a memory segment containing the response data (valid until plugin close)
     * @throws PluginException if the call fails
     */
    public @NotNull MemorySegment callRaw(int messageId, @NotNull BinaryStruct request, long responseSize)
            throws PluginException {
        return callRaw(messageId, request, responseSize, pluginArena);
    }

    /**
     * Call the plugin with a binary struct request, using caller-provided arena for response.
     * <p>
     * This is the <b>fastest</b> binary transport method when you can reuse an arena across
     * multiple calls. The response is allocated in the provided arena, avoiding per-call
     * arena creation overhead.
     *
     * <pre>{@code
     * try (Arena arena = Arena.ofConfined()) {
     *     for (int i = 0; i < 10000; i++) {
     *         var request = new MyRequest(arena, ...);
     *         MemorySegment response = plugin.callRaw(MSG_ID, request, MyResponse.SIZE, arena);
     *         // process response...
     *     }
     * } // arena freed here
     * }</pre>
     *
     * @param messageId     the binary message ID
     * @param request       the request struct
     * @param responseSize  expected response size in bytes
     * @param responseArena arena to allocate the response in (caller manages lifecycle)
     * @return a memory segment containing the response data (valid until arena is closed)
     * @throws PluginException if the call fails
     */
    public @NotNull MemorySegment callRaw(int messageId, @NotNull BinaryStruct request, long responseSize,
                                          @NotNull Arena responseArena) throws PluginException {
        if (closed) {
            throw new PluginException(1, "Plugin has been closed");
        }

        try {
            // Use confined arena for FFI call overhead only - faster than shared
            MemorySegment resultBuffer;
            try (Arena ffiArena = Arena.ofConfined()) {
                resultBuffer = (MemorySegment) bindings.pluginCallRaw().invoke(
                        ffiArena,
                        handle,
                        messageId,
                        request.segment(),
                        request.byteSize()
                );

                // Parse response while ffiArena is still valid
                return parseRawResultBuffer(resultBuffer, responseSize, responseArena);
            }
        } catch (PluginException e) {
            throw e;
        } catch (Throwable t) {
            throw new PluginException("Native raw call failed", t);
        }
    }

    /**
     * Call the plugin with a binary struct request, returning response as byte array.
     * <p>
     * This is the <b>simplest</b> high-performance binary transport method. No arena
     * management is needed - the response is copied to a Java byte array.
     *
     * <pre>{@code
     * byte[] response = plugin.callRawBytes(MSG_ID, request);
     * // Parse response bytes...
     * }</pre>
     *
     * @param messageId the binary message ID
     * @param request   the request struct
     * @return response data as a byte array
     * @throws PluginException if the call fails
     */
    public byte @NotNull [] callRawBytes(int messageId, @NotNull BinaryStruct request) throws PluginException {
        if (closed) {
            throw new PluginException(1, "Plugin has been closed");
        }

        try (Arena ffiArena = Arena.ofConfined()) {
            MemorySegment resultBuffer = (MemorySegment) bindings.pluginCallRaw().invoke(
                    ffiArena,
                    handle,
                    messageId,
                    request.segment(),
                    request.byteSize()
            );

            return parseRawResultBufferToBytes(resultBuffer);
        } catch (PluginException e) {
            throw e;
        } catch (Throwable t) {
            throw new PluginException("Native raw call failed", t);
        }
    }

    /**
     * Parse the RbResponse buffer and copy to caller's arena.
     */
    private MemorySegment parseRawResultBuffer(MemorySegment responseStruct, long expectedSize,
                                                Arena responseArena) throws PluginException {
        // RbResponse layout: { error_code: u32, len: u32, capacity: u32, _padding: u32, data: *mut c_void }
        int errorCode = responseStruct.get(ValueLayout.JAVA_INT, 0);
        int len = responseStruct.get(ValueLayout.JAVA_INT, 4);
        MemorySegment data = responseStruct.get(ValueLayout.ADDRESS, 16);

        try {
            if (errorCode != 0) {
                String errorMessage = "Unknown error";
                if (!data.equals(MemorySegment.NULL) && len > 0) {
                    MemorySegment slice = data.reinterpret(len);
                    errorMessage = new String(slice.toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
                }
                throw new PluginException(errorCode, errorMessage);
            }

            if (data.equals(MemorySegment.NULL) || len == 0) {
                throw new PluginException("Empty response from raw call");
            }

            if (len != expectedSize) {
                throw new PluginException(String.format(
                        "Response size mismatch: expected %d, got %d", expectedSize, len));
            }

            // Copy response data to caller's arena
            MemorySegment responseData = responseArena.allocate(len);
            MemorySegment sourceSlice = data.reinterpret(len);
            responseData.copyFrom(sourceSlice);

            return responseData;
        } finally {
            freeRawResponse(responseStruct);
        }
    }

    /**
     * Parse the RbResponse buffer and return as byte array (no arena needed).
     */
    private byte[] parseRawResultBufferToBytes(MemorySegment responseStruct) throws PluginException {
        int errorCode = responseStruct.get(ValueLayout.JAVA_INT, 0);
        int len = responseStruct.get(ValueLayout.JAVA_INT, 4);
        MemorySegment data = responseStruct.get(ValueLayout.ADDRESS, 16);

        try {
            if (errorCode != 0) {
                String errorMessage = "Unknown error";
                if (!data.equals(MemorySegment.NULL) && len > 0) {
                    MemorySegment slice = data.reinterpret(len);
                    errorMessage = new String(slice.toArray(ValueLayout.JAVA_BYTE), StandardCharsets.UTF_8);
                }
                throw new PluginException(errorCode, errorMessage);
            }

            if (data.equals(MemorySegment.NULL) || len == 0) {
                throw new PluginException("Empty response from raw call");
            }

            // Copy directly to byte array - no intermediate arena needed
            MemorySegment sourceSlice = data.reinterpret(len);
            return sourceSlice.toArray(ValueLayout.JAVA_BYTE);
        } finally {
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
    public void setLogLevel(@NotNull LogLevel level) {
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
