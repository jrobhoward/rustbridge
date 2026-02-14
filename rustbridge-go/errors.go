package rustbridge

import (
	"errors"
	"fmt"
)

// ErrorCode represents a numeric error code from the FFI layer.
// Values match the Rust PluginError::error_code() mapping in rustbridge-core.
type ErrorCode uint32

const (
	ErrorCodeSuccess             ErrorCode = 0
	ErrorCodeInvalidState        ErrorCode = 1
	ErrorCodeInitializationFailed ErrorCode = 2
	ErrorCodeShutdownFailed      ErrorCode = 3
	ErrorCodeConfigError         ErrorCode = 4
	ErrorCodeSerializationError  ErrorCode = 5
	ErrorCodeUnknownMessageType  ErrorCode = 6
	ErrorCodeHandlerError        ErrorCode = 7
	ErrorCodeRuntimeError        ErrorCode = 8
	ErrorCodeCancelled           ErrorCode = 9
	ErrorCodeTimeout             ErrorCode = 10
	ErrorCodeInternal            ErrorCode = 11
	ErrorCodeFfiError            ErrorCode = 12
	ErrorCodeTooManyRequests     ErrorCode = 13
)

// String returns the string representation of the error code.
func (c ErrorCode) String() string {
	switch c {
	case ErrorCodeSuccess:
		return "Success"
	case ErrorCodeInvalidState:
		return "InvalidState"
	case ErrorCodeInitializationFailed:
		return "InitializationFailed"
	case ErrorCodeShutdownFailed:
		return "ShutdownFailed"
	case ErrorCodeConfigError:
		return "ConfigError"
	case ErrorCodeSerializationError:
		return "SerializationError"
	case ErrorCodeUnknownMessageType:
		return "UnknownMessageType"
	case ErrorCodeHandlerError:
		return "HandlerError"
	case ErrorCodeRuntimeError:
		return "RuntimeError"
	case ErrorCodeCancelled:
		return "Cancelled"
	case ErrorCodeTimeout:
		return "Timeout"
	case ErrorCodeInternal:
		return "Internal"
	case ErrorCodeFfiError:
		return "FfiError"
	case ErrorCodeTooManyRequests:
		return "TooManyRequests"
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
// It uses errors.As so it works with wrapped errors.
func IsPluginError(err error) (*PluginError, bool) {
	var pe *PluginError
	if errors.As(err, &pe) {
		return pe, true
	}
	return nil, false
}
