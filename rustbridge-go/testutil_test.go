package rustbridge

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

// findHelloPlugin locates the hello-plugin shared library by walking up
// from the test file to the workspace root.
func findHelloPlugin(t *testing.T) string {
	t.Helper()

	var libName string
	switch runtime.GOOS {
	case "darwin":
		libName = "libhello_plugin.dylib"
	case "windows":
		libName = "hello_plugin.dll"
	default:
		libName = "libhello_plugin.so"
	}

	// Start from this file's directory and walk up
	_, thisFile, _, ok := runtime.Caller(0)
	if !ok {
		t.Skip("cannot determine test file location")
	}

	dir := filepath.Dir(thisFile)
	for {
		candidate := filepath.Join(dir, "target", "release", libName)
		if _, err := os.Stat(candidate); err == nil {
			return candidate
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}

	t.Skipf("hello-plugin not found (%s). Run: cargo build --release -p hello-plugin", libName)
	return ""
}

// loadTestPlugin loads the hello-plugin or skips the test if not built.
func loadTestPlugin(t *testing.T, opts ...Option) *Plugin {
	t.Helper()

	path := findHelloPlugin(t)
	plugin, err := Load(path, opts...)
	if err != nil {
		t.Fatalf("Load(%s) failed: %v", path, err)
	}

	t.Cleanup(func() {
		plugin.Close()
	})

	return plugin
}
