package rustbridge

// LifecycleState represents the current state of a plugin instance.
// Values match the Rust LifecycleState enum in rustbridge-core.
type LifecycleState uint8

const (
	StateInstalled LifecycleState = 0
	StateStarting  LifecycleState = 1
	StateActive    LifecycleState = 2
	StateStopping  LifecycleState = 3
	StateStopped   LifecycleState = 4
	StateFailed    LifecycleState = 5
	StateInvalid   LifecycleState = 255
)

// String returns the string representation of the lifecycle state.
func (s LifecycleState) String() string {
	switch s {
	case StateInstalled:
		return "Installed"
	case StateStarting:
		return "Starting"
	case StateActive:
		return "Active"
	case StateStopping:
		return "Stopping"
	case StateStopped:
		return "Stopped"
	case StateFailed:
		return "Failed"
	case StateInvalid:
		return "Invalid"
	default:
		return "Unknown"
	}
}

// CanHandleRequests returns true if the plugin can accept calls in this state.
func (s LifecycleState) CanHandleRequests() bool {
	return s == StateActive
}

// IsTerminal returns true if the plugin has reached a final state.
func (s LifecycleState) IsTerminal() bool {
	return s == StateStopped || s == StateFailed
}
