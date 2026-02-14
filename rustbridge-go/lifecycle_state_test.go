package rustbridge

import "testing"

func TestLifecycleState___String___ReturnsCorrectName(t *testing.T) {
	tests := []struct {
		state LifecycleState
		want  string
	}{
		{StateInstalled, "Installed"},
		{StateStarting, "Starting"},
		{StateActive, "Active"},
		{StateStopping, "Stopping"},
		{StateStopped, "Stopped"},
		{StateFailed, "Failed"},
		{StateInvalid, "Invalid"},
		{LifecycleState(99), "Unknown"},
	}

	for _, tt := range tests {
		got := tt.state.String()

		if got != tt.want {
			t.Errorf("LifecycleState(%d).String() = %q, want %q", tt.state, got, tt.want)
		}
	}
}

func TestCanHandleRequests___ActiveState___ReturnsTrue(t *testing.T) {
	if !StateActive.CanHandleRequests() {
		t.Error("Active.CanHandleRequests() = false, want true")
	}
}

func TestCanHandleRequests___NonActiveStates___ReturnsFalse(t *testing.T) {
	nonActive := []LifecycleState{
		StateInstalled, StateStarting, StateStopping, StateStopped, StateFailed, StateInvalid,
	}

	for _, s := range nonActive {
		if s.CanHandleRequests() {
			t.Errorf("%s.CanHandleRequests() = true, want false", s)
		}
	}
}

func TestIsTerminal___StoppedAndFailed___ReturnsTrue(t *testing.T) {
	if !StateStopped.IsTerminal() {
		t.Error("Stopped.IsTerminal() = false, want true")
	}
	if !StateFailed.IsTerminal() {
		t.Error("Failed.IsTerminal() = false, want true")
	}
}

func TestIsTerminal___NonTerminalStates___ReturnsFalse(t *testing.T) {
	nonTerminal := []LifecycleState{
		StateInstalled, StateStarting, StateActive, StateStopping,
	}

	for _, s := range nonTerminal {
		if s.IsTerminal() {
			t.Errorf("%s.IsTerminal() = true, want false", s)
		}
	}
}
