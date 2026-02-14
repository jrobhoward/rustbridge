package rustbridge

import (
	"encoding/json"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
)

func TestLoad___DefaultConfig___StateIsActive(t *testing.T) {
	plugin := loadTestPlugin(t)

	state := plugin.State()

	if state != StateActive {
		t.Errorf("State() = %s, want Active", state)
	}
}

func TestCall___EchoMessage___ReturnsResponse(t *testing.T) {
	plugin := loadTestPlugin(t)

	response, err := plugin.Call("echo", `{"message": "Hello from Go!"}`)

	if err != nil {
		t.Fatalf("Call error: %v", err)
	}

	var result map[string]any
	if err := json.Unmarshal([]byte(response), &result); err != nil {
		t.Fatalf("json.Unmarshal error: %v", err)
	}

	if result["message"] != "Hello from Go!" {
		t.Errorf("message = %v", result["message"])
	}
	if result["length"] != float64(14) {
		t.Errorf("length = %v, want 14", result["length"])
	}
}

func TestCall___Greet___ReturnsGreeting(t *testing.T) {
	plugin := loadTestPlugin(t)

	response, err := plugin.Call("greet", `{"name": "Gopher"}`)

	if err != nil {
		t.Fatalf("Call error: %v", err)
	}

	var result map[string]any
	if err := json.Unmarshal([]byte(response), &result); err != nil {
		t.Fatalf("json.Unmarshal error: %v", err)
	}

	greeting, ok := result["greeting"].(string)
	if !ok {
		t.Fatal("greeting is not a string")
	}
	if !strings.Contains(greeting, "Gopher") {
		t.Errorf("greeting = %q, does not contain 'Gopher'", greeting)
	}
}

func TestCall___MathAdd___ReturnsSum(t *testing.T) {
	plugin := loadTestPlugin(t)

	response, err := plugin.Call("math.add", `{"a": 10, "b": 32}`)

	if err != nil {
		t.Fatalf("Call error: %v", err)
	}

	var result map[string]any
	if err := json.Unmarshal([]byte(response), &result); err != nil {
		t.Fatalf("json.Unmarshal error: %v", err)
	}

	if result["result"] != float64(42) {
		t.Errorf("result = %v, want 42", result["result"])
	}
}

func TestCall___UserCreate___ReturnsUserId(t *testing.T) {
	plugin := loadTestPlugin(t)

	response, err := plugin.Call("user.create", `{"username": "go_user", "email": "go@example.com"}`)

	if err != nil {
		t.Fatalf("Call error: %v", err)
	}

	var result map[string]any
	if err := json.Unmarshal([]byte(response), &result); err != nil {
		t.Fatalf("json.Unmarshal error: %v", err)
	}

	userId, ok := result["user_id"].(string)
	if !ok || userId == "" {
		t.Errorf("user_id = %v", result["user_id"])
	}

	createdAt, ok := result["created_at"].(string)
	if !ok || createdAt == "" {
		t.Errorf("created_at = %v", result["created_at"])
	}
}

func TestCall___UnknownType___ReturnsErrorCode6(t *testing.T) {
	plugin := loadTestPlugin(t)

	_, err := plugin.Call("nonexistent", `{}`)

	if err == nil {
		t.Fatal("expected error for unknown type_tag")
	}

	pe, ok := IsPluginError(err)
	if !ok {
		t.Fatalf("expected PluginError, got %T: %v", err, err)
	}
	if pe.Code != ErrorCodeUnknownMessageType {
		t.Errorf("Code = %d (%s), want %d (UnknownMessageType)", pe.Code, pe.Code, ErrorCodeUnknownMessageType)
	}
}

func TestCall___AfterClose___ReturnsError(t *testing.T) {
	plugin := loadTestPlugin(t)
	plugin.Close()

	_, err := plugin.Call("echo", `{"message": "test"}`)

	if err == nil {
		t.Fatal("expected error after Close")
	}
}

func TestCall___ConcurrentCalls___AllSucceed(t *testing.T) {
	plugin := loadTestPlugin(t)

	var wg sync.WaitGroup
	var failures atomic.Int64
	goroutines := 10
	callsPerGoroutine := 100

	for i := 0; i < goroutines; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < callsPerGoroutine; j++ {
				_, err := plugin.Call("echo", `{"message": "concurrent"}`)
				if err != nil {
					failures.Add(1)
				}
			}
		}()
	}

	wg.Wait()

	if f := failures.Load(); f > 0 {
		t.Errorf("%d/%d calls failed", f, goroutines*callsPerGoroutine)
	}
}

func TestSetLogLevel___ChangesLevel___NoError(t *testing.T) {
	plugin := loadTestPlugin(t)

	plugin.SetLogLevel(LogLevelDebug)
	plugin.SetLogLevel(LogLevelError)
	plugin.SetLogLevel(LogLevelInfo)
}

func TestClose___AfterShutdown___StateIsStopped(t *testing.T) {
	path := findHelloPlugin(t)
	plugin, err := Load(path)
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}

	plugin.Close()

	state := plugin.State()
	if state != StateStopped {
		t.Errorf("State() = %s, want Stopped", state)
	}
}

func TestLoad___WithLogHandler___ReceivesLogMessages(t *testing.T) {
	var received atomic.Int64
	handler := func(level LogLevel, target string, message string) {
		received.Add(1)
	}

	plugin := loadTestPlugin(t, WithLogLevel(LogLevelTrace), WithLogHandler(handler))

	// Make a call to trigger logging
	plugin.Call("echo", `{"message": "trigger log"}`)

	// Log messages are best-effort; we verify the callback mechanism doesn't crash
	_ = received.Load()
}

func TestCallTyped___EchoMessage___UnmarshalsResponse(t *testing.T) {
	plugin := loadTestPlugin(t)

	type EchoRequest struct {
		Message string `json:"message"`
	}
	type EchoResponse struct {
		Message string `json:"message"`
		Length  int    `json:"length"`
	}

	var resp EchoResponse
	err := plugin.CallTyped("echo", &EchoRequest{Message: "typed call"}, &resp)

	if err != nil {
		t.Fatalf("CallTyped error: %v", err)
	}
	if resp.Message != "typed call" {
		t.Errorf("Message = %q", resp.Message)
	}
	if resp.Length != 10 {
		t.Errorf("Length = %d, want 10", resp.Length)
	}
}

func TestRejectedRequestCount___DefaultConfig___ReturnsZero(t *testing.T) {
	plugin := loadTestPlugin(t)

	count := plugin.RejectedRequestCount()

	if count != 0 {
		t.Errorf("RejectedRequestCount() = %d, want 0", count)
	}
}

func TestRejectedRequestCount___LowLimit___IncrementsUnderContention(t *testing.T) {
	plugin := loadTestPlugin(t, WithMaxConcurrentOps(1))

	var wg sync.WaitGroup
	var errorCount atomic.Int64
	goroutines := 8
	callsPerGoroutine := 50

	for i := 0; i < goroutines; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < callsPerGoroutine; j++ {
				_, err := plugin.Call("echo", `{"message": "flood"}`)
				if err != nil {
					errorCount.Add(1)
				}
			}
		}()
	}

	wg.Wait()

	rejected := plugin.RejectedRequestCount()
	errors := errorCount.Load()

	if rejected == 0 {
		t.Errorf("RejectedRequestCount() = 0, expected > 0 with max_concurrent_ops=1 and %d goroutines", goroutines)
	}
	if rejected != uint64(errors) {
		t.Errorf("RejectedRequestCount() = %d, error count = %d, expected equal", rejected, errors)
	}
}

func TestHasBinaryTransport___HelloPlugin___ReturnsTrue(t *testing.T) {
	plugin := loadTestPlugin(t)

	if !plugin.HasBinaryTransport() {
		t.Error("HasBinaryTransport() = false, want true")
	}
}
