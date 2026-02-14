package rustbridge

import "strings"

// LogLevel represents the severity level of a log message.
// Values match the Rust LogLevel enum in rustbridge-core.
type LogLevel uint8

const (
	LogLevelTrace LogLevel = 0
	LogLevelDebug LogLevel = 1
	LogLevelInfo  LogLevel = 2
	LogLevelWarn  LogLevel = 3
	LogLevelError LogLevel = 4
	LogLevelOff   LogLevel = 5
)

// String returns the lowercase string representation of the log level.
func (l LogLevel) String() string {
	switch l {
	case LogLevelTrace:
		return "trace"
	case LogLevelDebug:
		return "debug"
	case LogLevelInfo:
		return "info"
	case LogLevelWarn:
		return "warn"
	case LogLevelError:
		return "error"
	case LogLevelOff:
		return "off"
	default:
		return "unknown"
	}
}

// ParseLogLevel parses a string into a LogLevel.
// Parsing is case-insensitive. Returns LogLevelInfo and false for unrecognized strings.
func ParseLogLevel(s string) (LogLevel, bool) {
	switch strings.ToLower(s) {
	case "trace":
		return LogLevelTrace, true
	case "debug":
		return LogLevelDebug, true
	case "info":
		return LogLevelInfo, true
	case "warn", "warning":
		return LogLevelWarn, true
	case "error":
		return LogLevelError, true
	case "off":
		return LogLevelOff, true
	default:
		return LogLevelInfo, false
	}
}
