// Package rustbridge provides a Go consumer for rustbridge plugins.
//
// It loads native shared libraries (.so, .dylib, .dll) via CGo and dlopen,
// providing direct in-process FFI calls for low-latency communication with
// Rust plugins built using the rustbridge framework.
//
// Basic usage:
//
//	plugin, err := rustbridge.Load("/path/to/libmyplugin.so")
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer plugin.Close()
//
//	response, err := plugin.Call("echo", `{"message": "hello"}`)
package rustbridge

import (
	"fmt"
	"unsafe"
)

// Load opens a native plugin library and initializes it with the given options.
// The returned Plugin must be closed with Close() when no longer needed.
func Load(libraryPath string, opts ...Option) (*Plugin, error) {
	cfg := defaultConfig()
	for _, opt := range opts {
		opt(cfg)
	}

	lib, err := openLibrary(libraryPath)
	if err != nil {
		return nil, fmt.Errorf("failed to load library: %w", err)
	}

	pluginPtr := ffiCreate(lib.fnCreate)
	if pluginPtr == nil {
		lib.close()
		return nil, &PluginError{Code: ErrorCodeInitFailed, Message: "plugin_create returned null"}
	}

	configJSON, err := cfg.toJSON()
	if err != nil {
		lib.close()
		return nil, fmt.Errorf("failed to serialize config: %w", err)
	}

	var logCallbackPtr unsafe.Pointer
	if cfg.logHandler != nil {
		setGlobalLogHandler(cfg.logHandler)
		logCallbackPtr = ffiLogCallbackPtr()
	}

	handle := ffiInit(lib.fnInit, pluginPtr, configJSON, logCallbackPtr)
	if handle == nil {
		lib.close()
		return nil, &PluginError{Code: ErrorCodeInitFailed, Message: "plugin_init returned null"}
	}

	return &Plugin{
		lib:    lib,
		handle: handle,
	}, nil
}
