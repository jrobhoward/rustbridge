package rustbridge

import "testing"

func TestLogLevel___String___ReturnsCorrectName(t *testing.T) {
	tests := []struct {
		level LogLevel
		want  string
	}{
		{LogLevelTrace, "trace"},
		{LogLevelDebug, "debug"},
		{LogLevelInfo, "info"},
		{LogLevelWarn, "warn"},
		{LogLevelError, "error"},
		{LogLevelOff, "off"},
		{LogLevel(99), "unknown"},
	}

	for _, tt := range tests {
		got := tt.level.String()

		if got != tt.want {
			t.Errorf("LogLevel(%d).String() = %q, want %q", tt.level, got, tt.want)
		}
	}
}

func TestParseLogLevel___ValidStrings___RoundTrips(t *testing.T) {
	levels := []LogLevel{
		LogLevelTrace, LogLevelDebug, LogLevelInfo,
		LogLevelWarn, LogLevelError, LogLevelOff,
	}

	for _, level := range levels {
		parsed, ok := ParseLogLevel(level.String())

		if !ok {
			t.Errorf("ParseLogLevel(%q) returned ok=false", level.String())
		}
		if parsed != level {
			t.Errorf("ParseLogLevel(%q) = %d, want %d", level.String(), parsed, level)
		}
	}
}

func TestParseLogLevel___CaseInsensitive___ParsesCorrectly(t *testing.T) {
	level, ok := ParseLogLevel("DEBUG")

	if !ok || level != LogLevelDebug {
		t.Errorf("ParseLogLevel(\"DEBUG\") = %d, %v; want %d, true", level, ok, LogLevelDebug)
	}
}

func TestParseLogLevel___Warning___ParsesAsWarn(t *testing.T) {
	level, ok := ParseLogLevel("warning")

	if !ok || level != LogLevelWarn {
		t.Errorf("ParseLogLevel(\"warning\") = %d, %v; want %d, true", level, ok, LogLevelWarn)
	}
}

func TestParseLogLevel___InvalidString___ReturnsFalse(t *testing.T) {
	_, ok := ParseLogLevel("invalid")

	if ok {
		t.Error("ParseLogLevel(\"invalid\") returned ok=true, want false")
	}
}
