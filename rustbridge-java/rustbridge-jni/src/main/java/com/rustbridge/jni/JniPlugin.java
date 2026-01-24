package com.rustbridge.jni;

import com.rustbridge.*;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * JNI-based plugin implementation for Java 8+ compatibility.
 * <p>
 * This implementation uses JNI to call native plugin functions.
 * Use this when FFM (Java 21+) is not available.
 */
public class JniPlugin implements Plugin {
    private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper()
        .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

    private final long handle;
    private final LogCallback logCallback;
    private volatile boolean closed = false;

    static {
        // Load the JNI bridge library
        try {
            System.loadLibrary("rustbridge_jni");
        } catch (UnsatisfiedLinkError e) {
            throw new RuntimeException("Failed to load rustbridge_jni native library", e);
        }
    }

    /**
     * Create a new JNI plugin wrapper.
     *
     * @param handle      the native plugin handle
     * @param logCallback optional log callback
     */
    JniPlugin(long handle, LogCallback logCallback) {
        this.handle = handle;
        this.logCallback = logCallback;
    }

    @Override
    public LifecycleState getState() {
        checkNotClosed();
        int stateCode = nativeGetState(handle);
        if (stateCode == 255) {
            throw new IllegalStateException("Invalid plugin handle");
        }
        return LifecycleState.fromCode(stateCode);
    }

    @Override
    public String call(String typeTag, String request) throws PluginException {
        checkNotClosed();
        return nativeCall(handle, typeTag, request);
    }

    @Override
    public <T, R> R call(String typeTag, T request, Class<R> responseType) throws PluginException {
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

    @Override
    public void setLogLevel(LogLevel level) {
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

    // Native methods (implemented in Rust)

    private static native int nativeGetState(long handle);

    private static native String nativeCall(long handle, String typeTag, String request)
            throws PluginException;

    private static native void nativeSetLogLevel(long handle, int level);

    private static native long nativeGetRejectedCount(long handle);

    private static native boolean nativeShutdown(long handle);
}
