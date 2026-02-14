package rustbridge

import (
	"testing"
	"unsafe"
)

func TestCallRaw___SmallBenchmark___ReturnsValidResponse(t *testing.T) {
	plugin := loadTestPlugin(t)

	req := NewSmallRequest("test_key", 0x01)

	data, err := plugin.CallRaw(MsgBenchSmall, unsafe.Pointer(&req), int(unsafe.Sizeof(req)))

	if err != nil {
		t.Fatalf("CallRaw error: %v", err)
	}

	if len(data) != int(unsafe.Sizeof(SmallResponseRaw{})) {
		t.Fatalf("response size = %d, want %d", len(data), unsafe.Sizeof(SmallResponseRaw{}))
	}

	resp := (*SmallResponseRaw)(unsafe.Pointer(&data[0]))
	if resp.Version != 1 {
		t.Errorf("Version = %d, want 1", resp.Version)
	}
	if resp.ValueLen == 0 {
		t.Error("ValueLen = 0, expected non-zero")
	}
	if resp.TtlSeconds != 3600 {
		t.Errorf("TtlSeconds = %d, want 3600", resp.TtlSeconds)
	}
	if resp.CacheHit != 1 {
		t.Errorf("CacheHit = %d, want 1 (flags & 1 != 0)", resp.CacheHit)
	}
}

func TestCallRaw___UnknownMessageId___ReturnsError(t *testing.T) {
	plugin := loadTestPlugin(t)

	req := NewSmallRequest("test_key", 0)

	_, err := plugin.CallRaw(999, unsafe.Pointer(&req), int(unsafe.Sizeof(req)))

	if err == nil {
		t.Fatal("expected error for unknown message_id")
	}

	pe, ok := IsPluginError(err)
	if !ok {
		t.Fatalf("expected PluginError, got %T: %v", err, err)
	}
	if pe.Code != ErrorCodeUnknownMessageType {
		t.Errorf("Code = %d, want %d", pe.Code, ErrorCodeUnknownMessageType)
	}
}

func TestStructSizes___MatchRustLayout(t *testing.T) {
	// These are compile-time assertions (the var _ lines in binary_transport.go),
	// but we verify them in a test for documentation purposes.
	if got := unsafe.Sizeof(SmallRequestRaw{}); got != 76 {
		t.Errorf("SmallRequestRaw size = %d, want 76", got)
	}
	if got := unsafe.Sizeof(SmallResponseRaw{}); got != 80 {
		t.Errorf("SmallResponseRaw size = %d, want 80", got)
	}
}

func TestCallRaw___CacheHitFlag___TogglesBehavior(t *testing.T) {
	plugin := loadTestPlugin(t)

	// flags=0 → cache_hit should be 0
	reqMiss := NewSmallRequest("key", 0)
	dataMiss, err := plugin.CallRaw(MsgBenchSmall, unsafe.Pointer(&reqMiss), int(unsafe.Sizeof(reqMiss)))
	if err != nil {
		t.Fatalf("CallRaw error: %v", err)
	}

	respMiss := (*SmallResponseRaw)(unsafe.Pointer(&dataMiss[0]))
	if respMiss.CacheHit != 0 {
		t.Errorf("flags=0: CacheHit = %d, want 0", respMiss.CacheHit)
	}

	// flags=1 → cache_hit should be 1
	reqHit := NewSmallRequest("key", 1)
	dataHit, err := plugin.CallRaw(MsgBenchSmall, unsafe.Pointer(&reqHit), int(unsafe.Sizeof(reqHit)))
	if err != nil {
		t.Fatalf("CallRaw error: %v", err)
	}

	respHit := (*SmallResponseRaw)(unsafe.Pointer(&dataHit[0]))
	if respHit.CacheHit != 1 {
		t.Errorf("flags=1: CacheHit = %d, want 1", respHit.CacheHit)
	}
}
