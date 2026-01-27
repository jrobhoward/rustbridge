package com.rustbridge.jni;

import com.rustbridge.*;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

/**
 * JNI-based plugin implementation for Java 17+ compatibility.
 * <p>
 * This implementation uses JNI to call native plugin functions.
 * Use this when FFM (Java 21+) is not available.
 */
public class JniPlugin implements Plugin {
    private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

    static {
        // Load the JNI bridge library
        try {
            System.loadLibrary("rustbridge_jni");
        } catch (UnsatisfiedLinkError e) {
            throw new RuntimeException("Failed to load rustbridge_jni native library", e);
        }
    }

    private final long handle;
    private final LogCallback logCallback;
    private volatile boolean closed = false;

    /**
     * Create a new JNI plugin wrapper.
     *
     * @param handle      the native plugin handle
     * @param logCallback optional log callback
     */
    JniPlugin(long handle, @Nullable LogCallback logCallback) {
        this.handle = handle;
        this.logCallback = logCallback;
    }

    private static native int nativeGetState(long handle);

    private static native String nativeCall(long handle, String typeTag, String request)
            throws PluginException;

    private static native byte[] nativeCallRaw(long handle, int messageId, byte[] request)
            throws PluginException;

    private static native boolean nativeHasBinaryTransport(long handle);

    private static native void nativeSetLogLevel(long handle, int level);

    private static native long nativeGetRejectedCount(long handle);

    private static native boolean nativeShutdown(long handle);

    @Override
    public @NotNull LifecycleState getState() {
        checkNotClosed();
        int stateCode = nativeGetState(handle);
        if (stateCode == 255) {
            throw new IllegalStateException("Invalid plugin handle");
        }
        return LifecycleState.fromCode(stateCode);
    }

    @Override
    public @NotNull String call(@NotNull String typeTag, @NotNull String request) throws PluginException {
        checkNotClosed();
        return nativeCall(handle, typeTag, request);
    }

    // Native methods (implemented in Rust)

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
     * Check if this plugin supports binary transport.
     *
     * @return true if binary transport is available
     */
    public boolean hasBinaryTransport() {
        checkNotClosed();
        return nativeHasBinaryTransport(handle);
    }

    /**
     * Call the plugin with a binary struct request (raw binary transport).
     * <p>
     * This method bypasses JSON serialization for high-performance scenarios.
     * The request and response are fixed-size C structs serialized as byte arrays.
     *
     * @param messageId the binary message ID (registered with register_binary_handler)
     * @param request   the request struct as a byte array
     * @return the response struct as a byte array
     * @throws PluginException if the call fails or binary transport is not supported
     */
    public byte @NotNull [] callRaw(int messageId, byte @NotNull [] request) throws PluginException {
        checkNotClosed();
        return nativeCallRaw(handle, messageId, request);
    }

    // Native methods (implemented in Rust)

    @Override
    public void setLogLevel(@NotNull LogLevel level) {
        checkNotClosed();
        nativeSetLogLevel(handle, level.getCode());
    }

    @Override
    public long getRejectedRequestCount() {
        checkNotClosed();
        return nativeGetRejectedCount(handle);
    }

    @Override
    public void close() {
        if (closed) {
            return;
        }
        closed = true;
        nativeShutdown(handle);
    }

    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("Plugin has been closed");
        }
    }
}
