package rustbridge

/*
#cgo LDFLAGS: -ldl

#include <dlfcn.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// FfiBuffer matches Rust #[repr(C)] FfiBuffer
typedef struct {
	uint8_t  *data;
	size_t    len;
	size_t    capacity;
	uint32_t  error_code;
} FfiBuffer;

// RbResponse matches Rust #[repr(C)] RbResponse
// Note: compiler inserts 4-byte padding between capacity and data on 64-bit
typedef struct {
	uint32_t  error_code;
	uint32_t  len;
	uint32_t  capacity;
	void     *data;
} RbResponse;

// Function pointer typedefs matching Rust FFI exports
typedef void* (*plugin_create_fn)(void);
typedef void* (*plugin_init_fn)(void *plugin_ptr, const uint8_t *config_json,
                                 size_t config_len, void *log_callback);
typedef FfiBuffer (*plugin_call_fn)(void *handle, const char *type_tag,
                                     const uint8_t *request, size_t request_len);
typedef void (*plugin_free_buffer_fn)(FfiBuffer *buffer);
typedef int (*plugin_shutdown_fn)(void *handle);
typedef void (*plugin_set_log_level_fn)(void *handle, uint8_t level);
typedef uint8_t (*plugin_get_state_fn)(void *handle);
typedef uint64_t (*plugin_get_rejected_count_fn)(void *handle);
typedef RbResponse (*plugin_call_raw_fn)(void *handle, uint32_t message_id,
                                          const void *request, size_t request_size);
typedef void (*rb_response_free_fn)(RbResponse *response);

// C trampoline functions — CGo cannot call function pointers directly.

static inline void* tramp_create(void *fn) {
	return ((plugin_create_fn)fn)();
}

static inline void* tramp_init(void *fn, void *pp, const uint8_t *cfg,
                                size_t cfg_len, void *cb) {
	return ((plugin_init_fn)fn)(pp, cfg, cfg_len, cb);
}

static inline FfiBuffer tramp_call(void *fn, void *h, const char *tt,
                                    const uint8_t *req, size_t req_len) {
	return ((plugin_call_fn)fn)(h, tt, req, req_len);
}

static inline void tramp_free_buffer(void *fn, FfiBuffer *buf) {
	((plugin_free_buffer_fn)fn)(buf);
}

static inline int tramp_shutdown(void *fn, void *h) {
	return ((plugin_shutdown_fn)fn)(h);
}

static inline void tramp_set_log_level(void *fn, void *h, uint8_t level) {
	((plugin_set_log_level_fn)fn)(h, level);
}

static inline uint8_t tramp_get_state(void *fn, void *h) {
	return ((plugin_get_state_fn)fn)(h);
}

static inline uint64_t tramp_get_rejected_count(void *fn, void *h) {
	return ((plugin_get_rejected_count_fn)fn)(h);
}

static inline RbResponse tramp_call_raw(void *fn, void *h, uint32_t mid,
                                         const void *req, size_t req_sz) {
	return ((plugin_call_raw_fn)fn)(h, mid, req, req_sz);
}

static inline void tramp_response_free(void *fn, RbResponse *resp) {
	((rb_response_free_fn)fn)(resp);
}

// Forward declaration for the Go-exported log callback.
extern void goLogCallback(uint8_t level, char *target, uint8_t *message, size_t message_len);
*/
import "C"

import (
	"fmt"
	"unsafe"
)

// nativeLibrary holds dlopen handle and resolved function pointers.
type nativeLibrary struct {
	handle             unsafe.Pointer
	fnCreate           unsafe.Pointer
	fnInit             unsafe.Pointer
	fnCall             unsafe.Pointer
	fnFreeBuffer       unsafe.Pointer
	fnShutdown         unsafe.Pointer
	fnSetLogLevel      unsafe.Pointer
	fnGetState         unsafe.Pointer
	fnGetRejectedCount unsafe.Pointer
	fnCallRaw          unsafe.Pointer // optional (binary transport)
	fnResponseFree     unsafe.Pointer // optional (binary transport)
}

// openLibrary loads a native plugin library and resolves all required symbols.
func openLibrary(path string) (*nativeLibrary, error) {
	cpath := C.CString(path)
	defer C.free(unsafe.Pointer(cpath))

	handle := C.dlopen(cpath, C.RTLD_NOW|C.RTLD_LOCAL)
	if handle == nil {
		return nil, fmt.Errorf("dlopen(%s): %s", path, C.GoString(C.dlerror()))
	}

	lib := &nativeLibrary{handle: handle}

	required := []struct {
		name string
		dest *unsafe.Pointer
	}{
		{"plugin_create", &lib.fnCreate},
		{"plugin_init", &lib.fnInit},
		{"plugin_call", &lib.fnCall},
		{"plugin_free_buffer", &lib.fnFreeBuffer},
		{"plugin_shutdown", &lib.fnShutdown},
		{"plugin_set_log_level", &lib.fnSetLogLevel},
		{"plugin_get_state", &lib.fnGetState},
		{"plugin_get_rejected_count", &lib.fnGetRejectedCount},
	}

	for _, sym := range required {
		csym := C.CString(sym.name)
		ptr := C.dlsym(handle, csym)
		C.free(unsafe.Pointer(csym))
		if ptr == nil {
			C.dlclose(handle)
			return nil, fmt.Errorf("dlsym(%s): symbol not found in %s", sym.name, path)
		}
		*sym.dest = ptr
	}

	optional := []struct {
		name string
		dest *unsafe.Pointer
	}{
		{"plugin_call_raw", &lib.fnCallRaw},
		{"rb_response_free", &lib.fnResponseFree},
	}

	for _, sym := range optional {
		csym := C.CString(sym.name)
		ptr := C.dlsym(handle, csym)
		C.free(unsafe.Pointer(csym))
		*sym.dest = ptr
	}

	return lib, nil
}

func (lib *nativeLibrary) close() {
	if lib.handle != nil {
		C.dlclose(lib.handle)
		lib.handle = nil
	}
}

func (lib *nativeLibrary) hasBinaryTransport() bool {
	return lib.fnCallRaw != nil && lib.fnResponseFree != nil
}

// --- Go wrapper functions for C trampolines ---

func ffiCreate(fn unsafe.Pointer) unsafe.Pointer {
	return C.tramp_create(fn)
}

func ffiInit(fn unsafe.Pointer, pluginPtr unsafe.Pointer, configJSON []byte, logCallback unsafe.Pointer) unsafe.Pointer {
	var cfgPtr *C.uint8_t
	cfgLen := C.size_t(len(configJSON))
	if len(configJSON) > 0 {
		cfgPtr = (*C.uint8_t)(unsafe.Pointer(&configJSON[0]))
	}
	return C.tramp_init(fn, pluginPtr, cfgPtr, cfgLen, logCallback)
}

// ffiCall performs a JSON call and returns data copied to the Go heap.
func ffiCall(fnCall, fnFree unsafe.Pointer, handle unsafe.Pointer, typeTag, request string) ([]byte, uint32) {
	cTag := C.CString(typeTag)
	defer C.free(unsafe.Pointer(cTag))

	var reqPtr *C.uint8_t
	reqLen := C.size_t(len(request))
	if len(request) > 0 {
		reqPtr = (*C.uint8_t)(unsafe.Pointer(unsafe.StringData(request)))
	}

	buf := C.tramp_call(fnCall, handle, cTag, reqPtr, reqLen)

	var data []byte
	if buf.data != nil && buf.len > 0 {
		data = C.GoBytes(unsafe.Pointer(buf.data), C.int(buf.len))
	}
	errCode := uint32(buf.error_code)

	C.tramp_free_buffer(fnFree, &buf)

	return data, errCode
}

func ffiShutdown(fn unsafe.Pointer, handle unsafe.Pointer) bool {
	return C.tramp_shutdown(fn, handle) != 0
}

func ffiGetState(fn unsafe.Pointer, handle unsafe.Pointer) uint8 {
	return uint8(C.tramp_get_state(fn, handle))
}

func ffiSetLogLevel(fn unsafe.Pointer, handle unsafe.Pointer, level uint8) {
	C.tramp_set_log_level(fn, handle, C.uint8_t(level))
}

func ffiGetRejectedCount(fn unsafe.Pointer, handle unsafe.Pointer) uint64 {
	return uint64(C.tramp_get_rejected_count(fn, handle))
}

// ffiCallRaw performs a binary call and returns data copied to the Go heap.
func ffiCallRaw(fnRaw, fnFree unsafe.Pointer, handle unsafe.Pointer,
	messageID uint32, request unsafe.Pointer, requestSize int) ([]byte, uint32) {
	resp := C.tramp_call_raw(fnRaw, handle, C.uint32_t(messageID),
		request, C.size_t(requestSize))

	errCode := uint32(resp.error_code)
	var data []byte
	if resp.data != nil && resp.len > 0 {
		data = C.GoBytes(resp.data, C.int(resp.len))
	}

	C.tramp_response_free(fnFree, &resp)

	return data, errCode
}

func ffiLogCallbackPtr() unsafe.Pointer {
	return unsafe.Pointer(C.goLogCallback)
}
