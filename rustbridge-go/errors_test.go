package rustbridge

import (
	"errors"
	"testing"
)

func TestPluginError___Error___FormatsCorrectly(t *testing.T) {
	err := &PluginError{Code: ErrorCodeUnknownMessageType, Message: "handler not found"}

	got := err.Error()

	if got != "plugin error 6 (UnknownMessageType): handler not found" {
		t.Errorf("Error() = %q", got)
	}
}

func TestIsPluginError___PluginError___ReturnsTrue(t *testing.T) {
	err := error(&PluginError{Code: ErrorCodeInternal, Message: "test"})

	pe, ok := IsPluginError(err)

	if !ok {
		t.Fatal("IsPluginError returned false for *PluginError")
	}
	if pe.Code != ErrorCodeInternal {
		t.Errorf("Code = %d, want %d", pe.Code, ErrorCodeInternal)
	}
}

func TestIsPluginError___OtherError___ReturnsFalse(t *testing.T) {
	err := errors.New("generic error")

	_, ok := IsPluginError(err)

	if ok {
		t.Error("IsPluginError returned true for generic error")
	}
}

func TestErrorCode___String___AllCodesHaveNames(t *testing.T) {
	codes := []ErrorCode{
		ErrorCodeSuccess, ErrorCodeInvalidHandle, ErrorCodeNotReady,
		ErrorCodeConcurrencyLimit, ErrorCodeInvalidInput, ErrorCodeSerializationError,
		ErrorCodeUnknownMessageType, ErrorCodeTimeout, ErrorCodeShutdown,
		ErrorCodeInternal, ErrorCodeConfigError, ErrorCodePanic,
		ErrorCodeInitFailed, ErrorCodeTransportError,
	}

	for _, code := range codes {
		s := code.String()

		if s == "" {
			t.Errorf("ErrorCode(%d).String() returned empty string", code)
		}
		if s[:1] >= "0" && s[:1] <= "9" {
			t.Errorf("ErrorCode(%d).String() = %q, looks like a fallback", code, s)
		}
	}
}
