package rustbridge

import "testing"

func TestResponseEnvelope___SuccessResponse___ParsesCorrectly(t *testing.T) {
	data := []byte(`{"status":"success","payload":{"message":"hello","length":5}}`)

	env, err := parseResponseEnvelope(data)

	if err != nil {
		t.Fatalf("parseResponseEnvelope error: %v", err)
	}
	if !env.IsSuccess() {
		t.Error("expected success response")
	}
	if env.ToError() != nil {
		t.Error("ToError() should return nil for success")
	}
	if env.PayloadJSON() == "null" {
		t.Error("payload should not be null")
	}
}

func TestResponseEnvelope___ErrorResponse___ParsesCorrectly(t *testing.T) {
	data := []byte(`{"status":"error","error_code":6,"error_message":"Unknown type"}`)

	env, err := parseResponseEnvelope(data)

	if err != nil {
		t.Fatalf("parseResponseEnvelope error: %v", err)
	}
	if env.IsSuccess() {
		t.Error("expected error response")
	}

	pe := env.ToError()
	if pe == nil {
		t.Fatal("ToError() returned nil for error response")
	}
	if pe.Code != ErrorCodeUnknownMessageType {
		t.Errorf("Code = %d, want %d", pe.Code, ErrorCodeUnknownMessageType)
	}
	if pe.Message != "Unknown type" {
		t.Errorf("Message = %q", pe.Message)
	}
}

func TestResponseEnvelope___InvalidJSON___ReturnsError(t *testing.T) {
	data := []byte(`not json`)

	_, err := parseResponseEnvelope(data)

	if err == nil {
		t.Fatal("expected error for invalid JSON")
	}
	pe, ok := IsPluginError(err)
	if !ok {
		t.Fatal("expected PluginError")
	}
	if pe.Code != ErrorCodeSerializationError {
		t.Errorf("Code = %d, want %d", pe.Code, ErrorCodeSerializationError)
	}
}

func TestResponseEnvelope___NullPayload___ReturnsNullString(t *testing.T) {
	data := []byte(`{"status":"success"}`)

	env, err := parseResponseEnvelope(data)

	if err != nil {
		t.Fatalf("parseResponseEnvelope error: %v", err)
	}
	if env.PayloadJSON() != "null" {
		t.Errorf("PayloadJSON() = %q, want \"null\"", env.PayloadJSON())
	}
}
