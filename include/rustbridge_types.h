/**
 * rustbridge_types.h - Core FFI types for rustbridge binary transport
 *
 * This header defines the fundamental types used for binary transport
 * between host languages and Rust plugins. These types are designed
 * to be ABI-compatible with their Rust counterparts.
 *
 * Memory Ownership:
 * - Borrowed types (RbString, RbBytes): Caller owns the memory
 * - Owned types (RbStringOwned, RbBytesOwned): Rust owns the memory
 * - For owned types, call the corresponding free function
 *
 * Copyright (c) 2024 rustbridge contributors
 * Licensed under MIT OR Apache-2.0
 */

#ifndef RUSTBRIDGE_TYPES_H
#define RUSTBRIDGE_TYPES_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Borrowed Types (caller-owned memory)
 * ============================================================================ */

/**
 * FFI-safe borrowed string reference
 *
 * This is a view into a UTF-8 string that the caller owns.
 * The string data must remain valid for the duration of the FFI call.
 *
 * Invariants:
 * - If data is non-null, it must point to valid UTF-8 bytes
 * - If data is non-null, it must be null-terminated
 * - len is the byte length, NOT including the null terminator
 * - len=0 and data=NULL indicates "not present" (None)
 * - len=0 and data!=NULL indicates empty string ""
 */
typedef struct {
    uint32_t len;           /* Length in bytes (excluding null terminator) */
    const uint8_t* data;    /* Pointer to null-terminated UTF-8 data */
} RbString;

/**
 * Create an empty/absent RbString (represents None)
 */
#define RB_STRING_NONE ((RbString){ .len = 0, .data = NULL })

/**
 * Create an RbString from a string literal
 * Note: The literal should NOT include the null terminator in len
 */
#define RB_STRING_LITERAL(s) ((RbString){ .len = sizeof(s) - 1, .data = (const uint8_t*)(s) })

/**
 * Check if an RbString is present (not None)
 */
#define RB_STRING_IS_PRESENT(s) ((s).data != NULL)

/**
 * FFI-safe borrowed byte slice reference
 *
 * This is a view into binary data that the caller owns.
 * The data must remain valid for the duration of the FFI call.
 *
 * Invariants:
 * - If data is non-null, it must point to len valid bytes
 * - len=0 and data=NULL indicates "not present" (None)
 * - Maximum size is 4GB (uint32_t max)
 */
typedef struct {
    uint32_t len;           /* Length in bytes */
    const uint8_t* data;    /* Pointer to binary data */
} RbBytes;

/**
 * Create an empty/absent RbBytes (represents None)
 */
#define RB_BYTES_NONE ((RbBytes){ .len = 0, .data = NULL })

/**
 * Check if an RbBytes is present (not None)
 */
#define RB_BYTES_IS_PRESENT(b) ((b).data != NULL)

/* ============================================================================
 * Owned Types (Rust-owned memory, must be freed)
 * ============================================================================ */

/**
 * FFI-safe owned string
 *
 * Unlike RbString, this type owns its memory and must be freed
 * by calling rb_string_free(). This is used for strings returned
 * from Rust to the host.
 *
 * Memory ownership:
 * - Memory is allocated by Rust
 * - Must be freed by calling rb_string_free()
 * - Do NOT free with host language's free()
 */
typedef struct {
    uint32_t len;           /* Length in bytes (excluding null terminator) */
    uint8_t* data;          /* Pointer to null-terminated UTF-8 data (Rust-owned) */
    uint32_t capacity;      /* Allocation capacity (for proper deallocation) */
} RbStringOwned;

/**
 * FFI-safe owned byte buffer
 *
 * Unlike RbBytes, this type owns its memory and must be freed
 * by calling rb_bytes_free(). This is used for binary data returned
 * from Rust to the host.
 *
 * Memory ownership:
 * - Memory is allocated by Rust
 * - Must be freed by calling rb_bytes_free()
 * - Do NOT free with host language's free()
 */
typedef struct {
    uint32_t len;           /* Length in bytes */
    uint8_t* data;          /* Pointer to binary data (Rust-owned) */
    uint32_t capacity;      /* Allocation capacity (for proper deallocation) */
} RbBytesOwned;

/* ============================================================================
 * Response Buffer for Binary Transport
 * ============================================================================ */

/**
 * FFI buffer for binary transport responses
 *
 * Similar to FfiBuffer but designed specifically for binary struct responses.
 * The response data is a raw C struct that can be cast directly by the host.
 *
 * Usage:
 * - error_code == 0: Success, data points to response struct
 * - error_code != 0: Error, data may point to null-terminated error message
 */
typedef struct {
    uint32_t error_code;    /* Error code (0 = success) */
    uint32_t len;           /* Size of response data in bytes */
    uint32_t capacity;      /* Allocation capacity */
    void* data;             /* Pointer to response data (or error message) */
} RbResponse;

/**
 * Check if an RbResponse indicates success
 */
#define RB_RESPONSE_IS_SUCCESS(r) ((r).error_code == 0)

/**
 * Check if an RbResponse indicates an error
 */
#define RB_RESPONSE_IS_ERROR(r) ((r).error_code != 0)

/**
 * Get the error message from an RbResponse (only valid if IS_ERROR)
 */
#define RB_RESPONSE_ERROR_MSG(r) ((const char*)(r).data)

/* ============================================================================
 * Error Codes
 * ============================================================================ */

/**
 * Standard error codes returned by rustbridge functions
 */
typedef enum {
    RB_ERROR_NONE               = 0,    /* Success */
    RB_ERROR_INVALID_STATE      = 1,    /* Plugin in invalid state for operation */
    RB_ERROR_INIT_FAILED        = 2,    /* Plugin initialization failed */
    RB_ERROR_SHUTDOWN_FAILED    = 3,    /* Plugin shutdown failed */
    RB_ERROR_CONFIG             = 4,    /* Configuration error */
    RB_ERROR_SERIALIZATION      = 5,    /* Serialization/deserialization error */
    RB_ERROR_UNKNOWN_MESSAGE    = 6,    /* Unknown message type */
    RB_ERROR_HANDLER            = 7,    /* Handler returned an error */
    RB_ERROR_RUNTIME            = 8,    /* Runtime error */
    RB_ERROR_CANCELLED          = 9,    /* Operation was cancelled */
    RB_ERROR_TIMEOUT            = 10,   /* Operation timed out */
    RB_ERROR_INTERNAL           = 11,   /* Internal error (including panics) */
    RB_ERROR_FFI                = 12,   /* FFI-specific error */
} RbErrorCode;

/* ============================================================================
 * Plugin Lifecycle States
 * ============================================================================ */

/**
 * Plugin lifecycle states
 */
typedef enum {
    RB_STATE_INSTALLED  = 0,    /* Plugin created, not initialized */
    RB_STATE_STARTING   = 1,    /* Initializing */
    RB_STATE_ACTIVE     = 2,    /* Ready to handle requests */
    RB_STATE_STOPPING   = 3,    /* Shutdown in progress */
    RB_STATE_STOPPED    = 4,    /* Shutdown complete */
    RB_STATE_FAILED     = 5,    /* Error occurred */
} RbLifecycleState;

/* ============================================================================
 * Log Levels
 * ============================================================================ */

/**
 * Log levels for plugin logging
 */
typedef enum {
    RB_LOG_TRACE    = 0,
    RB_LOG_DEBUG    = 1,
    RB_LOG_INFO     = 2,
    RB_LOG_WARN     = 3,
    RB_LOG_ERROR    = 4,
    RB_LOG_OFF      = 5,
} RbLogLevel;

/* ============================================================================
 * Plugin Handle
 * ============================================================================ */

/**
 * Opaque handle to a plugin instance
 * This is returned by plugin_init and passed to other plugin functions
 */
typedef void* RbPluginHandle;

/* ============================================================================
 * Log Callback
 * ============================================================================ */

/**
 * Callback function for receiving log messages from the plugin
 *
 * @param level     Log level (RbLogLevel)
 * @param message   Null-terminated log message
 * @param len       Length of message (excluding null terminator)
 */
typedef void (*RbLogCallback)(uint8_t level, const char* message, size_t len);

/* ============================================================================
 * Function Declarations
 * ============================================================================ */

/**
 * Initialize a plugin instance
 *
 * @param plugin_ptr    Pointer to plugin created by plugin_create()
 * @param config_json   JSON configuration (null for defaults)
 * @param config_len    Length of config_json
 * @param log_callback  Callback for log messages (may be NULL)
 * @return              Plugin handle, or NULL on failure
 */
RbPluginHandle plugin_init(
    void* plugin_ptr,
    const uint8_t* config_json,
    size_t config_len,
    RbLogCallback log_callback
);

/**
 * Make a synchronous JSON request to the plugin
 *
 * @param handle        Plugin handle from plugin_init()
 * @param type_tag      Null-terminated message type tag
 * @param request       JSON request payload
 * @param request_len   Length of request payload
 * @return              FfiBuffer with JSON response
 */
/* Note: Returns FfiBuffer, not RbResponse - for JSON transport */

/**
 * Make a synchronous binary request to the plugin
 *
 * @param handle        Plugin handle from plugin_init()
 * @param message_id    Numeric message identifier
 * @param request       Pointer to request struct
 * @param request_size  Size of request struct (for validation)
 * @return              RbResponse with binary response
 */
RbResponse plugin_call_raw(
    RbPluginHandle handle,
    uint32_t message_id,
    const void* request,
    size_t request_size
);

/**
 * Free an RbResponse returned by plugin_call_raw
 *
 * @param response      Pointer to response to free
 */
void rb_response_free(RbResponse* response);

/**
 * Shutdown a plugin instance
 *
 * @param handle        Plugin handle from plugin_init()
 * @return              true on success, false on failure
 */
bool plugin_shutdown(RbPluginHandle handle);

/**
 * Get the current lifecycle state of a plugin
 *
 * @param handle        Plugin handle from plugin_init()
 * @return              Current state (RbLifecycleState), or 255 for invalid handle
 */
uint8_t plugin_get_state(RbPluginHandle handle);

/**
 * Set the log level for a plugin
 *
 * @param handle        Plugin handle from plugin_init()
 * @param level         New log level (RbLogLevel)
 */
void plugin_set_log_level(RbPluginHandle handle, uint8_t level);

#ifdef __cplusplus
}
#endif

#endif /* RUSTBRIDGE_TYPES_H */
