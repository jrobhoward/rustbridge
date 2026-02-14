package rustbridge

import "fmt"

// ErrorCode represents a numeric error code from the FFI layer.
// Values match the Rust PluginError codes in rustbridge-core.
type ErrorCode uint32

const (
	ErrorCodeSuccess            ErrorCode = 0
	ErrorCodeInvalidHandle      ErrorCode = 1
	ErrorCodeNotReady           ErrorCode = 2
	ErrorCodeConcurrencyLimit   ErrorCode = 3
	ErrorCodeInvalidInput       ErrorCode = 4
	ErrorCodeSerializationError ErrorCode = 5
	ErrorCodeUnknownMessageType ErrorCode = 6
	ErrorCodeTimeout            ErrorCode = 7
	ErrorCodeShutdown           ErrorCode = 8
	ErrorCodeInternal           ErrorCode = 9
	ErrorCodeConfigError        ErrorCode = 10
	ErrorCodePanic              ErrorCode = 11
	ErrorCodeInitFailed         ErrorCode = 12
	ErrorCodeTransportError     ErrorCode = 13
)

// String returns the string representation of the error code.
func (c ErrorCode) String() string {
	switch c {
	case ErrorCodeSuccess:
		return "Success"
	case ErrorCodeInvalidHandle:
		return "InvalidHandle"
	case ErrorCodeNotReady:
		return "NotReady"
	case ErrorCodeConcurrencyLimit:
		return "ConcurrencyLimit"
	case ErrorCodeInvalidInput:
		return "InvalidInput"
	case ErrorCodeSerializationError:
		return "SerializationError"
	case ErrorCodeUnknownMessageType:
		return "UnknownMessageType"
	case ErrorCodeTimeout:
		return "Timeout"
	case ErrorCodeShutdown:
		return "Shutdown"
	case ErrorCodeInternal:
		return "Internal"
	case ErrorCodeConfigError:
		return "ConfigError"
	case ErrorCodePanic:
		return "Panic"
	case ErrorCodeInitFailed:
		return "InitFailed"
	case ErrorCodeTransportError:
		return "TransportError"
	default:
		return fmt.Sprintf("Unknown(%d)", c)
	}
}

// PluginError represents an error returned by a plugin operation.
type PluginError struct {
	Code    ErrorCode
	Message string
}

// Error implements the error interface.
func (e *PluginError) Error() string {
	return fmt.Sprintf("plugin error %d (%s): %s", e.Code, e.Code.String(), e.Message)
}

// IsPluginError checks if an error is a PluginError and returns it.
func IsPluginError(err error) (*PluginError, bool) {
	if pe, ok := err.(*PluginError); ok {
		return pe, true
	}
	return nil, false
}
