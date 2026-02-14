package rustbridge

import (
	"archive/zip"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

// BundleLoader extracts and verifies .rbp plugin bundles.
type BundleLoader struct {
	verifySignatures bool
	publicKeyOverride string
}

// BundleOption configures a BundleLoader.
type BundleOption func(*BundleLoader)

// WithVerifySignatures enables or disables signature verification.
func WithVerifySignatures(verify bool) BundleOption {
	return func(bl *BundleLoader) {
		bl.verifySignatures = verify
	}
}

// WithPublicKeyOverride sets a public key to use instead of the manifest's key.
func WithPublicKeyOverride(key string) BundleOption {
	return func(bl *BundleLoader) {
		bl.publicKeyOverride = key
	}
}

// NewBundleLoader creates a new BundleLoader with the given options.
// Signature verification is enabled by default.
func NewBundleLoader(opts ...BundleOption) *BundleLoader {
	bl := &BundleLoader{
		verifySignatures: true,
	}
	for _, opt := range opts {
		opt(bl)
	}
	return bl
}

// LoadBundle loads a plugin from an .rbp bundle file.
// The library is extracted to a temporary directory.
// The returned Plugin must be closed with Close() when no longer needed.
func LoadBundle(bundlePath string, opts ...Option) (*Plugin, error) {
	loader := NewBundleLoader(WithVerifySignatures(false))
	return loader.Load(bundlePath, opts...)
}

// Load extracts the library from a bundle and loads it as a plugin.
func (bl *BundleLoader) Load(bundlePath string, pluginOpts ...Option) (*Plugin, error) {
	tmpDir, err := os.MkdirTemp("", "rustbridge-")
	if err != nil {
		return nil, fmt.Errorf("failed to create temp dir: %w", err)
	}

	libPath, err := bl.ExtractLibrary(bundlePath, tmpDir)
	if err != nil {
		os.RemoveAll(tmpDir)
		return nil, err
	}

	plugin, err := Load(libPath, pluginOpts...)
	if err != nil {
		os.RemoveAll(tmpDir)
		return nil, err
	}

	return plugin, nil
}

// ExtractLibrary extracts the platform-specific library from a bundle to destDir.
// Returns the path to the extracted library file.
func (bl *BundleLoader) ExtractLibrary(bundlePath, destDir string) (string, error) {
	return bl.extractVariant(bundlePath, destDir, "")
}

// ExtractVariant extracts a specific variant of the library from a bundle to destDir.
func (bl *BundleLoader) ExtractVariant(bundlePath, destDir, variant string) (string, error) {
	return bl.extractVariant(bundlePath, destDir, variant)
}

// ExtractLibraryToTemp extracts the library to a unique temporary directory.
// The caller is responsible for cleaning up the returned directory.
func (bl *BundleLoader) ExtractLibraryToTemp(bundlePath string) (string, error) {
	tmpDir, err := os.MkdirTemp("", "rustbridge-")
	if err != nil {
		return "", fmt.Errorf("failed to create temp dir: %w", err)
	}

	libPath, err := bl.ExtractLibrary(bundlePath, tmpDir)
	if err != nil {
		os.RemoveAll(tmpDir)
		return "", err
	}

	return libPath, nil
}

// GetManifest reads the manifest from a bundle without extracting.
func (bl *BundleLoader) GetManifest(bundlePath string) (*BundleManifest, error) {
	r, err := zip.OpenReader(bundlePath)
	if err != nil {
		return nil, fmt.Errorf("failed to open bundle: %w", err)
	}
	defer r.Close()

	manifestData, err := readZipEntry(&r.Reader, "manifest.json")
	if err != nil {
		return nil, err
	}

	return ParseManifest(manifestData)
}

// ListVariants lists available variants for a platform.
// If platform is empty, the current platform is used.
func (bl *BundleLoader) ListVariants(bundlePath, platform string) ([]string, error) {
	manifest, err := bl.GetManifest(bundlePath)
	if err != nil {
		return nil, err
	}

	if platform == "" {
		platform = CurrentPlatform()
	}

	pi, ok := manifest.Platforms[platform]
	if !ok {
		return nil, fmt.Errorf("platform not supported: %s", platform)
	}

	return pi.ListVariants(), nil
}

// CurrentPlatform returns the platform string for the current system
// in the format used by .rbp manifests (e.g., "linux-x86_64").
func CurrentPlatform() string {
	osName := runtime.GOOS
	arch := runtime.GOARCH

	osMap := map[string]string{
		"linux":   "linux",
		"darwin":  "darwin",
		"windows": "windows",
	}

	archMap := map[string]string{
		"amd64": "x86_64",
		"arm64": "aarch64",
	}

	if mapped, ok := osMap[osName]; ok {
		osName = mapped
	}
	if mapped, ok := archMap[arch]; ok {
		arch = mapped
	}

	return osName + "-" + arch
}

func (bl *BundleLoader) extractVariant(bundlePath, destDir, variant string) (string, error) {
	r, err := zip.OpenReader(bundlePath)
	if err != nil {
		return "", fmt.Errorf("failed to open bundle: %w", err)
	}
	defer r.Close()

	// Read and parse manifest
	manifestData, err := readZipEntry(&r.Reader, "manifest.json")
	if err != nil {
		return "", err
	}

	manifest, err := ParseManifest(manifestData)
	if err != nil {
		return "", err
	}

	// Verify manifest signature if enabled
	if bl.verifySignatures {
		if err := bl.verifyManifestSignature(&r.Reader, manifest, manifestData); err != nil {
			return "", err
		}
	}

	// Detect platform
	currentPlatform := CurrentPlatform()
	pi, ok := manifest.Platforms[currentPlatform]
	if !ok {
		return "", fmt.Errorf("platform not supported: %s", currentPlatform)
	}

	// Get effective variant
	if variant == "" {
		variant = pi.GetDefaultVariant()
	}

	// Get library path and checksum
	libraryPath := pi.GetLibrary(variant)
	checksum := pi.GetChecksum(variant)

	if libraryPath == "" {
		return "", fmt.Errorf("variant '%s' not found for platform '%s'", variant, currentPlatform)
	}

	// Read library data
	libData, err := readZipEntry(&r.Reader, libraryPath)
	if err != nil {
		return "", err
	}

	// Verify checksum
	if !verifyChecksum(libData, checksum) {
		return "", fmt.Errorf("checksum verification failed for %s", libraryPath)
	}

	// Verify library signature if enabled
	if bl.verifySignatures {
		if err := bl.verifyLibrarySignature(&r.Reader, manifest, libraryPath, libData); err != nil {
			return "", err
		}
	}

	// Write to destination
	libFilename := filepath.Base(libraryPath)
	outputPath := filepath.Join(destDir, libFilename)

	if err := os.MkdirAll(destDir, 0o755); err != nil {
		return "", fmt.Errorf("failed to create output directory: %w", err)
	}

	if err := os.WriteFile(outputPath, libData, 0o755); err != nil {
		return "", fmt.Errorf("failed to write library: %w", err)
	}

	return outputPath, nil
}

func (bl *BundleLoader) verifyManifestSignature(z *zip.Reader, manifest *BundleManifest, manifestData []byte) error {
	publicKey := bl.publicKeyOverride
	if publicKey == "" {
		publicKey = manifest.PublicKey
	}
	if publicKey == "" {
		return errors.New("signature verification enabled but no public key available")
	}

	sigData, err := readZipEntry(z, "manifest.json.minisig")
	if err != nil {
		return errors.New("signature verification enabled but manifest.json.minisig not found in bundle")
	}

	verifier, err := NewMinisignVerifier(publicKey)
	if err != nil {
		return fmt.Errorf("invalid public key: %w", err)
	}

	valid, err := verifier.Verify(manifestData, string(sigData))
	if err != nil {
		return fmt.Errorf("manifest signature verification error: %w", err)
	}
	if !valid {
		return errors.New("manifest signature verification failed")
	}

	return nil
}

func (bl *BundleLoader) verifyLibrarySignature(z *zip.Reader, manifest *BundleManifest, libraryPath string, libraryData []byte) error {
	publicKey := bl.publicKeyOverride
	if publicKey == "" {
		publicKey = manifest.PublicKey
	}
	if publicKey == "" {
		return errors.New("no public key available for signature verification")
	}

	sigPath := libraryPath + ".minisig"
	sigData, err := readZipEntry(z, sigPath)
	if err != nil {
		return fmt.Errorf("signature verification enabled but %s not found in bundle", sigPath)
	}

	verifier, err := NewMinisignVerifier(publicKey)
	if err != nil {
		return fmt.Errorf("invalid public key: %w", err)
	}

	valid, err := verifier.Verify(libraryData, string(sigData))
	if err != nil {
		return fmt.Errorf("library signature verification error: %w", err)
	}
	if !valid {
		return fmt.Errorf("library signature verification failed: %s", libraryPath)
	}

	return nil
}

// readZipEntry reads a file from a zip archive by path.
func readZipEntry(z *zip.Reader, path string) ([]byte, error) {
	for _, f := range z.File {
		if f.Name == path {
			rc, err := f.Open()
			if err != nil {
				return nil, fmt.Errorf("failed to open %s in bundle: %w", path, err)
			}
			defer rc.Close()
			data, err := io.ReadAll(rc)
			if err != nil {
				return nil, fmt.Errorf("failed to read %s in bundle: %w", path, err)
			}
			return data, nil
		}
	}
	return nil, fmt.Errorf("file not found in bundle: %s", path)
}

// verifyChecksum verifies a SHA256 checksum. Handles both "sha256:xxx" and raw "xxx" formats.
func verifyChecksum(data []byte, expectedChecksum string) bool {
	hash := sha256.Sum256(data)
	actual := hex.EncodeToString(hash[:])

	expected := expectedChecksum
	if strings.HasPrefix(strings.ToLower(expected), "sha256:") {
		expected = expected[7:]
	}

	return strings.EqualFold(actual, expected)
}
