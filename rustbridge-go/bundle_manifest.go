package rustbridge

import (
	"encoding/json"
	"errors"
	"sort"
)

// GitInfo holds git repository information.
type GitInfo struct {
	Commit string `json:"commit,omitempty"`
	Branch string `json:"branch,omitempty"`
	Tag    string `json:"tag,omitempty"`
	Dirty  bool   `json:"dirty,omitempty"`
}

// BuildInfo holds build metadata from the manifest.
type BuildInfo struct {
	BuiltBy           string            `json:"built_by,omitempty"`
	BuiltAt           string            `json:"built_at,omitempty"`
	Host              string            `json:"host,omitempty"`
	Compiler          string            `json:"compiler,omitempty"`
	RustbridgeVersion string            `json:"rustbridge_version,omitempty"`
	Git               *GitInfo          `json:"git,omitempty"`
	Custom            map[string]string `json:"custom,omitempty"`
}

// Sbom holds Software Bill of Materials paths.
type Sbom struct {
	CycloneDX string `json:"cyclonedx,omitempty"`
	SPDX      string `json:"spdx,omitempty"`
}

// SchemaInfo holds schema file information.
type SchemaInfo struct {
	Path        string `json:"path,omitempty"`
	Format      string `json:"format,omitempty"`
	Checksum    string `json:"checksum,omitempty"`
	Description string `json:"description,omitempty"`
}

// BundleManifest represents the manifest.json inside an .rbp bundle.
type BundleManifest struct {
	BundleVersion  string                   `json:"bundle_version"`
	Plugin         PluginInfo               `json:"plugin"`
	Platforms      map[string]*PlatformInfo `json:"platforms"`
	PublicKey      string                   `json:"public_key,omitempty"`
	BuildInfo      *BuildInfo               `json:"build_info,omitempty"`
	Sbom           *Sbom                    `json:"sbom,omitempty"`
	SchemaChecksum string                   `json:"schema_checksum,omitempty"`
	Schemas        map[string]*SchemaInfo   `json:"schemas,omitempty"`
	Notices        string                   `json:"notices,omitempty"`
	LicenseFile    string                   `json:"license_file,omitempty"`
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
	Library        string                 `json:"library"`
	Checksum       string                 `json:"checksum"`
	Build          json.RawMessage        `json:"build,omitempty"`
	BuildInfo      *BuildInfo             `json:"build_info,omitempty"`
	Sbom           *Sbom                  `json:"sbom,omitempty"`
	SchemaChecksum string                 `json:"schema_checksum,omitempty"`
	Schemas        map[string]*SchemaInfo `json:"schemas,omitempty"`
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

// GetEffectiveBuildInfo returns the build info for a platform/variant (v1.1).
// Variant-level overrides top-level.
func (m *BundleManifest) GetEffectiveBuildInfo(platform, variant string) *BuildInfo {
	if pi, ok := m.Platforms[platform]; ok && pi.Variants != nil {
		if vi, ok := pi.Variants[variant]; ok && vi.BuildInfo != nil {
			return vi.BuildInfo
		}
	}
	return m.BuildInfo
}

// GetEffectiveSchemaChecksum returns the schema checksum for a platform/variant (v1.1).
// Variant-level overrides top-level.
func (m *BundleManifest) GetEffectiveSchemaChecksum(platform, variant string) string {
	if pi, ok := m.Platforms[platform]; ok && pi.Variants != nil {
		if vi, ok := pi.Variants[variant]; ok && vi.SchemaChecksum != "" {
			return vi.SchemaChecksum
		}
	}
	return m.SchemaChecksum
}

// GetEffectiveSbom returns the SBOM for a platform/variant (v1.1).
// Variant-level overrides top-level.
func (m *BundleManifest) GetEffectiveSbom(platform, variant string) *Sbom {
	if pi, ok := m.Platforms[platform]; ok && pi.Variants != nil {
		if vi, ok := pi.Variants[variant]; ok && vi.Sbom != nil {
			return vi.Sbom
		}
	}
	return m.Sbom
}

// GetEffectiveSchemas returns the schemas for a platform/variant (v1.1).
// Variant-level overrides top-level.
func (m *BundleManifest) GetEffectiveSchemas(platform, variant string) map[string]*SchemaInfo {
	if pi, ok := m.Platforms[platform]; ok && pi.Variants != nil {
		if vi, ok := pi.Variants[variant]; ok && len(vi.Schemas) > 0 {
			return vi.Schemas
		}
	}
	return m.Schemas
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
