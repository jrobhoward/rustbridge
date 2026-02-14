package rustbridge

import (
	"archive/zip"
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

// createTestBundle creates a minimal .rbp bundle ZIP file in memory.
func createTestBundle(t *testing.T, libContent []byte, platform string) string {
	t.Helper()

	checksum := sha256.Sum256(libContent)
	checksumStr := "sha256:" + hex.EncodeToString(checksum[:])

	var libName string
	switch {
	case strings.Contains(platform, "linux"):
		libName = "libtest.so"
	case strings.Contains(platform, "darwin"):
		libName = "libtest.dylib"
	case strings.Contains(platform, "windows"):
		libName = "test.dll"
	default:
		libName = "libtest.so"
	}

	libPath := "lib/" + platform + "/" + libName

	manifest := map[string]any{
		"bundle_version": "1.0",
		"plugin": map[string]any{
			"name":    "test-plugin",
			"version": "1.0.0",
		},
		"platforms": map[string]any{
			platform: map[string]any{
				"library":  libPath,
				"checksum": checksumStr,
			},
		},
	}

	manifestJSON, err := json.Marshal(manifest)
	if err != nil {
		t.Fatalf("json.Marshal manifest: %v", err)
	}

	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)

	w, _ := zw.Create("manifest.json")
	w.Write(manifestJSON)

	w, _ = zw.Create(libPath)
	w.Write(libContent)

	zw.Close()

	tmpFile := filepath.Join(t.TempDir(), "test-plugin.rbp")
	if err := os.WriteFile(tmpFile, buf.Bytes(), 0o644); err != nil {
		t.Fatalf("write bundle: %v", err)
	}

	return tmpFile
}

func TestCurrentPlatform___ReturnsValidFormat(t *testing.T) {
	platform := CurrentPlatform()

	parts := strings.SplitN(platform, "-", 2)
	if len(parts) != 2 {
		t.Fatalf("CurrentPlatform() = %q, expected os-arch format", platform)
	}
	if parts[0] == "" || parts[1] == "" {
		t.Fatalf("CurrentPlatform() = %q, has empty component", platform)
	}
}

func TestCurrentPlatform___LinuxAmd64___ReturnsLinuxX86_64(t *testing.T) {
	if runtime.GOOS != "linux" || runtime.GOARCH != "amd64" {
		t.Skip("test only runs on linux/amd64")
	}

	platform := CurrentPlatform()

	if platform != "linux-x86_64" {
		t.Errorf("CurrentPlatform() = %q, want linux-x86_64", platform)
	}
}

func TestGetManifest___ValidBundle___ParsesManifest(t *testing.T) {
	bundlePath := createTestBundle(t, []byte("fake lib content"), CurrentPlatform())
	loader := NewBundleLoader(WithVerifySignatures(false))

	m, err := loader.GetManifest(bundlePath)

	if err != nil {
		t.Fatalf("GetManifest error: %v", err)
	}
	if m.Plugin.Name != "test-plugin" {
		t.Errorf("Plugin.Name = %q", m.Plugin.Name)
	}
	if m.Plugin.Version != "1.0.0" {
		t.Errorf("Plugin.Version = %q", m.Plugin.Version)
	}
}

func TestExtractLibrary___ValidBundle___ExtractsLibrary(t *testing.T) {
	libContent := []byte("fake library binary content")
	bundlePath := createTestBundle(t, libContent, CurrentPlatform())
	loader := NewBundleLoader(WithVerifySignatures(false))

	destDir := t.TempDir()
	libPath, err := loader.ExtractLibrary(bundlePath, destDir)

	if err != nil {
		t.Fatalf("ExtractLibrary error: %v", err)
	}
	if libPath == "" {
		t.Fatal("libPath is empty")
	}

	data, err := os.ReadFile(libPath)
	if err != nil {
		t.Fatalf("ReadFile error: %v", err)
	}
	if !bytes.Equal(data, libContent) {
		t.Error("extracted content does not match original")
	}
}

func TestExtractLibrary___UnsupportedPlatform___ReturnsError(t *testing.T) {
	bundlePath := createTestBundle(t, []byte("lib"), "some-other-arch")
	loader := NewBundleLoader(WithVerifySignatures(false))

	_, err := loader.ExtractLibrary(bundlePath, t.TempDir())

	if err == nil {
		t.Fatal("expected error for unsupported platform")
	}
	if !strings.Contains(err.Error(), "platform not supported") {
		t.Errorf("error = %v, want 'platform not supported'", err)
	}
}

func TestGetManifest___FileNotFound___ReturnsError(t *testing.T) {
	loader := NewBundleLoader(WithVerifySignatures(false))

	_, err := loader.GetManifest("/nonexistent/path/bundle.rbp")

	if err == nil {
		t.Fatal("expected error for file not found")
	}
}

func TestExtractLibrary___CorruptedZip___ReturnsError(t *testing.T) {
	tmpFile := filepath.Join(t.TempDir(), "bad.rbp")
	os.WriteFile(tmpFile, []byte("not a zip file"), 0o644)

	loader := NewBundleLoader(WithVerifySignatures(false))

	_, err := loader.ExtractLibrary(tmpFile, t.TempDir())

	if err == nil {
		t.Fatal("expected error for corrupted zip")
	}
}

func TestExtractLibrary___MissingManifest___ReturnsError(t *testing.T) {
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)
	w, _ := zw.Create("some_file.txt")
	w.Write([]byte("content"))
	zw.Close()

	tmpFile := filepath.Join(t.TempDir(), "no-manifest.rbp")
	os.WriteFile(tmpFile, buf.Bytes(), 0o644)

	loader := NewBundleLoader(WithVerifySignatures(false))

	_, err := loader.ExtractLibrary(tmpFile, t.TempDir())

	if err == nil {
		t.Fatal("expected error for missing manifest")
	}
	if !strings.Contains(err.Error(), "manifest") {
		t.Errorf("error = %v, want mention of manifest", err)
	}
}

func TestExtractLibrary___ChecksumMismatch___ReturnsError(t *testing.T) {
	platform := CurrentPlatform()
	libContent := []byte("original content")

	// Create bundle with correct checksum
	bundlePath := createTestBundle(t, libContent, platform)

	// Now tamper with the library inside the zip
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)

	// Read original manifest
	r, _ := zip.OpenReader(bundlePath)
	manifestData, _ := readZipEntry(&r.Reader, "manifest.json")
	r.Close()

	// Write manifest unchanged
	w, _ := zw.Create("manifest.json")
	w.Write(manifestData)

	// Write tampered library
	var libName string
	switch {
	case strings.Contains(platform, "linux"):
		libName = "libtest.so"
	case strings.Contains(platform, "darwin"):
		libName = "libtest.dylib"
	default:
		libName = "libtest.so"
	}
	w, _ = zw.Create("lib/" + platform + "/" + libName)
	w.Write([]byte("tampered content that does not match checksum"))
	zw.Close()

	tmpFile := filepath.Join(t.TempDir(), "tampered.rbp")
	os.WriteFile(tmpFile, buf.Bytes(), 0o644)

	loader := NewBundleLoader(WithVerifySignatures(false))

	_, err := loader.ExtractLibrary(tmpFile, t.TempDir())

	if err == nil {
		t.Fatal("expected error for checksum mismatch")
	}
	if !strings.Contains(err.Error(), "checksum") {
		t.Errorf("error = %v, want checksum error", err)
	}
}

func TestVerifyChecksum___ValidSHA256___ReturnsTrue(t *testing.T) {
	data := []byte("hello rustbridge")
	hash := sha256.Sum256(data)
	checksum := hex.EncodeToString(hash[:])

	result := verifyChecksum(data, checksum)

	if !result {
		t.Error("verifyChecksum returned false for valid checksum")
	}
}

func TestVerifyChecksum___WithPrefix___HandlesCorrectly(t *testing.T) {
	data := []byte("hello rustbridge")
	hash := sha256.Sum256(data)
	checksum := "sha256:" + hex.EncodeToString(hash[:])

	result := verifyChecksum(data, checksum)

	if !result {
		t.Error("verifyChecksum returned false for valid checksum with sha256: prefix")
	}
}

func TestVerifyChecksum___WrongData___ReturnsFalse(t *testing.T) {
	data := []byte("hello rustbridge")
	hash := sha256.Sum256(data)
	checksum := hex.EncodeToString(hash[:])

	result := verifyChecksum([]byte("different data"), checksum)

	if result {
		t.Error("verifyChecksum returned true for wrong data")
	}
}

func TestListVariants___ValidBundle___ReturnsVariants(t *testing.T) {
	platform := CurrentPlatform()

	// Create a bundle with variants
	checksum1 := sha256.Sum256([]byte("release"))
	checksum2 := sha256.Sum256([]byte("debug"))
	manifest := map[string]any{
		"bundle_version": "1.0",
		"plugin": map[string]any{
			"name":    "test-plugin",
			"version": "1.0.0",
		},
		"platforms": map[string]any{
			platform: map[string]any{
				"library":         "lib/libtest.so",
				"checksum":        "sha256:" + hex.EncodeToString(checksum1[:]),
				"default_variant": "release",
				"variants": map[string]any{
					"release": map[string]any{
						"library":  "lib/release/libtest.so",
						"checksum": "sha256:" + hex.EncodeToString(checksum1[:]),
					},
					"debug": map[string]any{
						"library":  "lib/debug/libtest.so",
						"checksum": "sha256:" + hex.EncodeToString(checksum2[:]),
					},
				},
			},
		},
	}

	manifestJSON, _ := json.Marshal(manifest)
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)
	w, _ := zw.Create("manifest.json")
	w.Write(manifestJSON)
	zw.Close()

	tmpFile := filepath.Join(t.TempDir(), "variants.rbp")
	os.WriteFile(tmpFile, buf.Bytes(), 0o644)

	loader := NewBundleLoader(WithVerifySignatures(false))

	variants, err := loader.ListVariants(tmpFile, "")

	if err != nil {
		t.Fatalf("ListVariants error: %v", err)
	}
	if len(variants) != 2 {
		t.Fatalf("len(variants) = %d, want 2", len(variants))
	}
}

func TestExtractLibraryToTemp___ValidBundle___ExtractsToTempDir(t *testing.T) {
	libContent := []byte("fake lib for temp test")
	bundlePath := createTestBundle(t, libContent, CurrentPlatform())
	loader := NewBundleLoader(WithVerifySignatures(false))

	libPath, err := loader.ExtractLibraryToTemp(bundlePath)

	if err != nil {
		t.Fatalf("ExtractLibraryToTemp error: %v", err)
	}

	defer os.RemoveAll(filepath.Dir(libPath))

	data, err := os.ReadFile(libPath)
	if err != nil {
		t.Fatalf("ReadFile error: %v", err)
	}
	if !bytes.Equal(data, libContent) {
		t.Error("extracted content does not match original")
	}
}
