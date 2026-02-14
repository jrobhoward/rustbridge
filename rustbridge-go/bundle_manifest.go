package rustbridge

import (
	"encoding/json"
	"errors"
	"sort"
)

// BundleManifest represents the manifest.json inside an .rbp bundle.
type BundleManifest struct {
	BundleVersion string                   `json:"bundle_version"`
	Plugin        PluginInfo               `json:"plugin"`
	Platforms     map[string]*PlatformInfo `json:"platforms"`
	PublicKey     string                   `json:"public_key,omitempty"`
}

// PluginInfo holds plugin metadata from the manifest.
type PluginInfo struct {
	Name        string   `json:"name"`
	Version     string   `json:"version"`
	Description string   `json:"description,omitempty"`
	Authors     []string `json:"authors,omitempty"`
	License     string   `json:"license,omitempty"`
	Repository  string   `json:"repository,omitempty"`
}

// PlatformInfo holds platform-specific library information.
type PlatformInfo struct {
	Library        string                  `json:"library"`
	Checksum       string                  `json:"checksum"`
	DefaultVariant string                  `json:"default_variant,omitempty"`
	Variants       map[string]*VariantInfo `json:"variants,omitempty"`
}

// VariantInfo holds variant-specific library information.
type VariantInfo struct {
	Library  string `json:"library"`
	Checksum string `json:"checksum"`
}

// GetLibrary returns the effective library path for a variant.
// Falls back to the platform-level library if the variant is not found.
func (p *PlatformInfo) GetLibrary(variant string) string {
	if p.Variants != nil {
		if v, ok := p.Variants[variant]; ok {
			return v.Library
		}
	}
	return p.Library
}

// GetChecksum returns the effective checksum for a variant.
// Falls back to the platform-level checksum if the variant is not found.
func (p *PlatformInfo) GetChecksum(variant string) string {
	if p.Variants != nil {
		if v, ok := p.Variants[variant]; ok {
			return v.Checksum
		}
	}
	return p.Checksum
}

// GetDefaultVariant returns the default variant name, defaulting to "release".
func (p *PlatformInfo) GetDefaultVariant() string {
	if p.DefaultVariant != "" {
		return p.DefaultVariant
	}
	return "release"
}

// ListVariants returns the available variant names for this platform.
func (p *PlatformInfo) ListVariants() []string {
	if len(p.Variants) == 0 {
		return []string{"release"}
	}
	names := make([]string, 0, len(p.Variants))
	for name := range p.Variants {
		names = append(names, name)
	}
	sort.Strings(names)
	return names
}

// ParseManifest parses a bundle manifest from JSON bytes.
func ParseManifest(data []byte) (*BundleManifest, error) {
	var m BundleManifest
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, errors.New("failed to parse manifest JSON: " + err.Error())
	}

	if m.BundleVersion == "" {
		return nil, errors.New("missing required field: bundle_version")
	}
	if m.Plugin.Name == "" {
		return nil, errors.New("missing required field: plugin.name")
	}
	if m.Plugin.Version == "" {
		return nil, errors.New("missing required field: plugin.version")
	}

	return &m, nil
}
