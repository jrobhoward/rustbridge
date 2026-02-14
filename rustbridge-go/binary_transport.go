package rustbridge

import (
	"errors"
	"unsafe"
)

// Binary transport message IDs (matching hello-plugin)
const (
	MsgBenchSmall  uint32 = 1
	MsgBenchMedium uint32 = 2
	MsgBenchLarge  uint32 = 3
)

// SmallRequestRaw matches the Rust #[repr(C)] SmallRequestRaw struct (76 bytes).
type SmallRequestRaw struct {
	Version  uint8
	Reserved [3]uint8
	Key      [64]uint8
	KeyLen   uint32
	Flags    uint32
}

// Compile-time size assertion: SmallRequestRaw must be exactly 76 bytes.
var _ [76]byte = [unsafe.Sizeof(SmallRequestRaw{})]byte{}

// NewSmallRequest creates a SmallRequestRaw from a key string and flags.
func NewSmallRequest(key string, flags uint32) SmallRequestRaw {
	req := SmallRequestRaw{
		Version: 1,
		Flags:   flags,
	}
	n := copy(req.Key[:], key)
	req.KeyLen = uint32(n)
	return req
}

// SmallResponseRaw matches the Rust #[repr(C)] SmallResponseRaw struct (80 bytes).
type SmallResponseRaw struct {
	Version    uint8
	Reserved   [3]uint8
	Value      [64]uint8
	ValueLen   uint32
	TtlSeconds uint32
	CacheHit   uint8
	Padding    [3]uint8
}

// Compile-time size assertion: SmallResponseRaw must be exactly 80 bytes.
var _ [80]byte = [unsafe.Sizeof(SmallResponseRaw{})]byte{}

// ValueString returns the value as a Go string.
func (r *SmallResponseRaw) ValueString() string {
	n := int(r.ValueLen)
	if n > 64 {
		n = 64
	}
	return string(r.Value[:n])
}

// CallRaw sends a binary request to the plugin and returns the raw response bytes.
// The caller is responsible for interpreting the response bytes as the correct struct type.
func (p *Plugin) CallRaw(messageID uint32, request unsafe.Pointer, requestSize int) ([]byte, error) {
	p.mu.RLock()
	defer p.mu.RUnlock()

	if p.closed {
		return nil, errors.New("plugin is closed")
	}

	if !p.lib.hasBinaryTransport() {
		return nil, &PluginError{Code: ErrorCodeFfiError, Message: "binary transport not supported"}
	}

	data, errCode := ffiCallRaw(p.lib.fnCallRaw, p.lib.fnResponseFree, p.handle,
		messageID, request, requestSize)

	if errCode != 0 {
		msg := "binary call failed"
		if len(data) > 0 {
			msg = string(data)
		}
		return nil, &PluginError{Code: ErrorCode(errCode), Message: msg}
	}

	return data, nil
}
