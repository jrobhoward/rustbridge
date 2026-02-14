package rustbridge

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"unsafe"
)

func BenchmarkCall___SmallEcho(b *testing.B) {
	plugin := benchLoadPlugin(b)
	request := `{"message": "benchmark test"}`

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := plugin.Call("echo", request)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkCall___MediumEcho(b *testing.B) {
	plugin := benchLoadPlugin(b)
	msg := strings.Repeat("x", 1000)
	request := fmt.Sprintf(`{"message": "%s"}`, msg)

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := plugin.Call("echo", request)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkCallRaw___SmallBinary(b *testing.B) {
	plugin := benchLoadPlugin(b)
	if !plugin.HasBinaryTransport() {
		b.Skip("binary transport not available")
	}

	req := NewSmallRequest("bench_key", 0)
	reqSize := int(unsafe.Sizeof(req))

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := plugin.CallRaw(MsgBenchSmall, unsafe.Pointer(&req), reqSize)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkCall___ConcurrentEcho(b *testing.B) {
	plugin := benchLoadPlugin(b)
	request := `{"message": "concurrent bench"}`

	b.ReportAllocs()
	b.ResetTimer()

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			_, err := plugin.Call("echo", request)
			if err != nil {
				b.Fatal(err)
			}
		}
	})
}

func benchLoadPlugin(b *testing.B) *Plugin {
	b.Helper()

	path := benchFindPlugin(b)
	plugin, err := Load(path)
	if err != nil {
		b.Fatalf("Load failed: %v", err)
	}

	b.Cleanup(func() {
		plugin.Close()
	})

	return plugin
}

func benchFindPlugin(b *testing.B) string {
	b.Helper()

	var libName string
	switch runtime.GOOS {
	case "darwin":
		libName = "libhello_plugin.dylib"
	case "windows":
		libName = "hello_plugin.dll"
	default:
		libName = "libhello_plugin.so"
	}

	_, thisFile, _, ok := runtime.Caller(0)
	if !ok {
		b.Skip("cannot determine test file location")
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

	b.Skipf("hello-plugin not found (%s)", libName)
	return ""
}
