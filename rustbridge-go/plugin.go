package rustbridge

import (
	"encoding/json"
	"errors"
	"sync"
	"unsafe"
)

// Plugin represents a loaded and initialized rustbridge plugin.
// All methods are safe for concurrent use from multiple goroutines.
type Plugin struct {
	lib    *nativeLibrary
	handle unsafe.Pointer
	closed bool
	mu     sync.RWMutex
}

// Call sends a JSON request to the plugin and returns the response payload as a JSON string.
func (p *Plugin) Call(typeTag string, request string) (string, error) {
	p.mu.RLock()
	if p.closed {
		p.mu.RUnlock()
		return "", errors.New("plugin is closed")
	}
	lib := p.lib
	handle := p.handle
	p.mu.RUnlock()

	data, errCode := ffiCall(lib.fnCall, lib.fnFreeBuffer, handle, typeTag, request)

	if len(data) == 0 {
		if errCode != 0 {
			return "", &PluginError{Code: ErrorCode(errCode), Message: "empty error response"}
		}
		return "", &PluginError{Code: ErrorCodeInternal, Message: "empty response"}
	}

	env, err := parseResponseEnvelope(data)
	if err != nil {
		return "", err
	}

	if !env.IsSuccess() {
		return "", env.ToError()
	}

	return env.PayloadJSON(), nil
}

// CallTyped marshals the request to JSON, calls the plugin, and unmarshals the response.
func (p *Plugin) CallTyped(typeTag string, request any, response any) error {
	reqJSON, err := json.Marshal(request)
	if err != nil {
		return &PluginError{Code: ErrorCodeSerializationError, Message: "failed to marshal request: " + err.Error()}
	}

	respJSON, err := p.Call(typeTag, string(reqJSON))
	if err != nil {
		return err
	}

	if err := json.Unmarshal([]byte(respJSON), response); err != nil {
		return &PluginError{Code: ErrorCodeSerializationError, Message: "failed to unmarshal response: " + err.Error()}
	}

	return nil
}

// State returns the current lifecycle state of the plugin.
func (p *Plugin) State() LifecycleState {
	p.mu.RLock()
	defer p.mu.RUnlock()

	if p.closed {
		return StateStopped
	}

	return LifecycleState(ffiGetState(p.lib.fnGetState, p.handle))
}

// SetLogLevel changes the plugin's log level.
func (p *Plugin) SetLogLevel(level LogLevel) {
	p.mu.RLock()
	defer p.mu.RUnlock()

	if p.closed {
		return
	}

	ffiSetLogLevel(p.lib.fnSetLogLevel, p.handle, uint8(level))
}

// RejectedRequestCount returns the number of requests rejected due to concurrency limits.
func (p *Plugin) RejectedRequestCount() uint64 {
	p.mu.RLock()
	defer p.mu.RUnlock()

	if p.closed {
		return 0
	}

	return ffiGetRejectedCount(p.lib.fnGetRejectedCount, p.handle)
}

// HasBinaryTransport returns true if the plugin supports binary transport.
func (p *Plugin) HasBinaryTransport() bool {
	return p.lib.hasBinaryTransport()
}

// Close shuts down the plugin and releases all resources.
// It is safe to call Close multiple times; subsequent calls are no-ops.
// Close implements io.Closer.
func (p *Plugin) Close() error {
	p.mu.Lock()
	defer p.mu.Unlock()

	if p.closed {
		return nil
	}
	p.closed = true

	ffiShutdown(p.lib.fnShutdown, p.handle)
	p.lib.close()

	return nil
}
