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
 */
public class FfmPlugin implements Plugin {
    private static final Gson GSON = new GsonBuilder().create();

    private final Arena arena;
    private final MemorySegment handle;
    private final NativeBindings bindings;
    private final LogCallback logCallback;

    private volatile boolean closed = false;

    /**
     * Create a new FFM plugin wrapper.
     *
     * @param arena       the memory arena for this plugin's lifetime
     * @param handle      the native plugin handle
     * @param bindings    the native function bindings
     * @param logCallback optional log callback
     */
    FfmPlugin(Arena arena, MemorySegment handle, NativeBindings bindings, LogCallback logCallback) {
        this.arena = arena;
        this.handle = handle;
        this.bindings = bindings;
        this.logCallback = logCallback;
    }

    @Override
    public LifecycleState getState() {
        checkNotClosed();
        try {
            int stateCode = (int) bindings.pluginGetState().invokeExact(handle);
            if (stateCode == 255) {
                throw new IllegalStateException("Invalid plugin handle");
            }
            return LifecycleState.fromCode(stateCode);
        } catch (Throwable t) {
            throw new RuntimeException("Failed to get plugin state", t);
        }
    }

    @Override
    public String call(String typeTag, String request) throws PluginException {
        checkNotClosed();

        try {
            // Allocate type tag as null-terminated string
            MemorySegment typeTagSegment = arena.allocateUtf8String(typeTag);

            // Allocate request data
            byte[] requestBytes = request.getBytes(StandardCharsets.UTF_8);
            MemorySegment requestSegment = arena.allocate(requestBytes.length);
            requestSegment.copyFrom(MemorySegment.ofArray(requestBytes));

            // Call the plugin (arena as SegmentAllocator allocates space for the returned struct)
            MemorySegment resultBuffer = (MemorySegment) bindings.pluginCall().invokeExact(
                    (SegmentAllocator) arena,
                    handle,
                    typeTagSegment,
                    requestSegment,
                    (long) requestBytes.length
            );

            // Parse the result buffer
            return parseResultBuffer(resultBuffer);
        } catch (PluginException e) {
            throw e;
        } catch (Throwable t) {
            throw new PluginException("Native call failed", t);
        }
    }

    @Override
    public <T, R> R call(String typeTag, T request, Class<R> responseType) throws PluginException {
        String requestJson = GSON.toJson(request);
        String responseJson = call(typeTag, requestJson);
        return GSON.fromJson(responseJson, responseType);
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

        // Close the arena (releases all allocated memory)
        arena.close();
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
