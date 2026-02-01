package com.rustbridge.ffm;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

/**
 * Native function bindings for rustbridge FFI.
 * <p>
 * This class holds the method handles for calling native plugin functions.
 */
public class NativeBindings {
    /**
     * Memory layout for RbResponse struct (binary transport response).
     * <pre>
     * struct RbResponse {
     *     error_code: u32,  // 0 = success
     *     len: u32,         // response data size
     *     capacity: u32,    // allocation capacity
     *     _padding: u32,    // alignment padding
     *     data: *mut c_void // response data pointer
     * }
     * </pre>
     */
    public static final StructLayout RB_RESPONSE_LAYOUT = MemoryLayout.structLayout(
            ValueLayout.JAVA_INT.withName("error_code"),
            ValueLayout.JAVA_INT.withName("len"),
            ValueLayout.JAVA_INT.withName("capacity"),
            MemoryLayout.paddingLayout(4),  // Alignment for pointer
            ValueLayout.ADDRESS.withName("data")
    );

    private final MethodHandle pluginInit;
    private final MethodHandle pluginCall;
    private final MethodHandle pluginCallRaw;      // nullable - binary transport optional
    private final MethodHandle pluginFreeBuffer;
    private final MethodHandle rbResponseFree;     // nullable - binary transport optional
    private final MethodHandle pluginShutdown;
    private final MethodHandle pluginSetLogLevel;
    private final MethodHandle pluginGetState;
    private final MethodHandle pluginGetRejectedCount;
    private final boolean hasBinaryTransport;

    /**
     * Create bindings from a symbol lookup.
     *
     * @param lookup the symbol lookup for the native library
     * @param linker the native linker
     */
    public NativeBindings(SymbolLookup lookup, Linker linker) {
        // FfiBuffer layout (for return type)
        // struct FfiBuffer { data: *mut u8, len: usize, capacity: usize, error_code: u32 }
        StructLayout ffiBufferLayout = MemoryLayout.structLayout(
                ValueLayout.ADDRESS.withName("data"),
                ValueLayout.JAVA_LONG.withName("len"),
                ValueLayout.JAVA_LONG.withName("capacity"),
                ValueLayout.JAVA_INT.withName("error_code"),
                MemoryLayout.paddingLayout(4) // Alignment padding
        );

        // plugin_init(plugin_ptr, config_json, config_len, log_callback) -> handle
        this.pluginInit = linker.downcallHandle(
                lookup.find("plugin_init").orElseThrow(),
                FunctionDescriptor.of(
                        ValueLayout.ADDRESS,  // return: handle
                        ValueLayout.ADDRESS,  // plugin_ptr
                        ValueLayout.ADDRESS,  // config_json
                        ValueLayout.JAVA_LONG, // config_len
                        ValueLayout.ADDRESS   // log_callback (nullable)
                )
        );

        // plugin_call(handle, type_tag, request, request_len) -> FfiBuffer
        this.pluginCall = linker.downcallHandle(
                lookup.find("plugin_call").orElseThrow(),
                FunctionDescriptor.of(
                        ffiBufferLayout,      // return: FfiBuffer
                        ValueLayout.ADDRESS,  // handle
                        ValueLayout.ADDRESS,  // type_tag
                        ValueLayout.ADDRESS,  // request
                        ValueLayout.JAVA_LONG // request_len
                )
        );

        // plugin_call_raw(handle, message_id, request, request_size) -> RbResponse
        // Optional - binary transport may not be available
        var callRawSymbol = lookup.find("plugin_call_raw");
        if (callRawSymbol.isPresent()) {
            this.pluginCallRaw = linker.downcallHandle(
                    callRawSymbol.get(),
                    FunctionDescriptor.of(
                            RB_RESPONSE_LAYOUT,   // return: RbResponse
                            ValueLayout.ADDRESS,  // handle
                            ValueLayout.JAVA_INT, // message_id
                            ValueLayout.ADDRESS,  // request
                            ValueLayout.JAVA_LONG // request_size
                    )
            );
        } else {
            this.pluginCallRaw = null;
        }

        // plugin_free_buffer(buffer)
        this.pluginFreeBuffer = linker.downcallHandle(
                lookup.find("plugin_free_buffer").orElseThrow(),
                FunctionDescriptor.ofVoid(
                        ValueLayout.ADDRESS   // buffer pointer
                )
        );

        // rb_response_free(response)
        // Optional - binary transport may not be available
        var rbResponseFreeSymbol = lookup.find("rb_response_free");
        if (rbResponseFreeSymbol.isPresent()) {
            this.rbResponseFree = linker.downcallHandle(
                    rbResponseFreeSymbol.get(),
                    FunctionDescriptor.ofVoid(
                            ValueLayout.ADDRESS   // response pointer
                    )
            );
        } else {
            this.rbResponseFree = null;
        }

        // Binary transport is available if both symbols are present
        this.hasBinaryTransport = this.pluginCallRaw != null && this.rbResponseFree != null;

        // plugin_shutdown(handle) -> bool
        this.pluginShutdown = linker.downcallHandle(
                lookup.find("plugin_shutdown").orElseThrow(),
                FunctionDescriptor.of(
                        ValueLayout.JAVA_BOOLEAN, // return: success
                        ValueLayout.ADDRESS       // handle
                )
        );

        // plugin_set_log_level(handle, level)
        this.pluginSetLogLevel = linker.downcallHandle(
                lookup.find("plugin_set_log_level").orElseThrow(),
                FunctionDescriptor.ofVoid(
                        ValueLayout.ADDRESS,  // handle
                        ValueLayout.JAVA_BYTE // level
                )
        );

        // plugin_get_state(handle) -> u8
        this.pluginGetState = linker.downcallHandle(
                lookup.find("plugin_get_state").orElseThrow(),
                FunctionDescriptor.of(
                        ValueLayout.JAVA_BYTE, // return: state code
                        ValueLayout.ADDRESS    // handle
                )
        );

        // plugin_get_rejected_count(handle) -> u64
        this.pluginGetRejectedCount = linker.downcallHandle(
                lookup.find("plugin_get_rejected_count").orElseThrow(),
                FunctionDescriptor.of(
                        ValueLayout.JAVA_LONG, // return: rejected count
                        ValueLayout.ADDRESS    // handle
                )
        );
    }

    public MethodHandle pluginInit() {
        return pluginInit;
    }

    public MethodHandle pluginCall() {
        return pluginCall;
    }

    public MethodHandle pluginCallRaw() {
        return pluginCallRaw;
    }

    public MethodHandle pluginFreeBuffer() {
        return pluginFreeBuffer;
    }

    public MethodHandle rbResponseFree() {
        return rbResponseFree;
    }

    public MethodHandle pluginShutdown() {
        return pluginShutdown;
    }

    public MethodHandle pluginSetLogLevel() {
        return pluginSetLogLevel;
    }

    public MethodHandle pluginGetState() {
        return pluginGetState;
    }

    public MethodHandle pluginGetRejectedCount() {
        return pluginGetRejectedCount;
    }

    /**
     * Check if binary transport is supported by this plugin.
     *
     * @return true if binary transport is available
     */
    public boolean hasBinaryTransport() {
        return hasBinaryTransport;
    }
}
