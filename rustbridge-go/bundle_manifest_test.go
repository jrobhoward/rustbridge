package rustbridge

import (
	"encoding/json"
	"testing"
)

func TestParseManifest___ValidJSON___ParsesCorrectly(t *testing.T) {
	manifestJSON, _ := json.Marshal(map[string]any{
		"bundle_version": "1.0",
		"plugin": map[string]any{
			"name":    "test-plugin",
			"version": "1.0.0",
		},
		"platforms": map[string]any{
			"linux-x86_64": map[string]any{
				"library":  "lib/linux-x86_64/libtest.so",
				"checksum": "sha256:abc123",
			},
		},
		"public_key": "RWS...",
	})

	m, err := ParseManifest(manifestJSON)

	if err != nil {
		t.Fatalf("ParseManifest error: %v", err)
	}
	if m.BundleVersion != "1.0" {
		t.Errorf("BundleVersion = %q", m.BundleVersion)
	}
	if m.Plugin.Name != "test-plugin" {
		t.Errorf("Plugin.Name = %q", m.Plugin.Name)
	}
	if m.Plugin.Version != "1.0.0" {
		t.Errorf("Plugin.Version = %q", m.Plugin.Version)
	}
	if m.PublicKey != "RWS..." {
		t.Errorf("PublicKey = %q", m.PublicKey)
	}
	p, ok := m.Platforms["linux-x86_64"]
	if !ok {
		t.Fatal("linux-x86_64 platform not found")
	}
	if p.Library != "lib/linux-x86_64/libtest.so" {
		t.Errorf("Library = %q", p.Library)
	}
}

func TestParseManifest___MissingBundleVersion___ReturnsError(t *testing.T) {
	manifestJSON, _ := json.Marshal(map[string]any{
		"plugin": map[string]any{
			"name":    "test",
			"version": "1.0.0",
		},
		"platforms": map[string]any{},
	})

	_, err := ParseManifest(manifestJSON)

	if err == nil {
		t.Fatal("expected error for missing bundle_version")
	}
}

func TestParseManifest___MissingPluginName___ReturnsError(t *testing.T) {
	manifestJSON, _ := json.Marshal(map[string]any{
		"bundle_version": "1.0",
		"plugin": map[string]any{
			"version": "1.0.0",
		},
		"platforms": map[string]any{},
	})

	_, err := ParseManifest(manifestJSON)

	if err == nil {
		t.Fatal("expected error for missing plugin.name")
	}
}

func TestParseManifest___MissingPluginVersion___ReturnsError(t *testing.T) {
	manifestJSON, _ := json.Marshal(map[string]any{
		"bundle_version": "1.0",
		"plugin": map[string]any{
			"name": "test",
		},
		"platforms": map[string]any{},
	})

	_, err := ParseManifest(manifestJSON)

	if err == nil {
		t.Fatal("expected error for missing plugin.version")
	}
}

func TestParseManifest___InvalidJSON___ReturnsError(t *testing.T) {
	_, err := ParseManifest([]byte("not valid json"))

	if err == nil {
		t.Fatal("expected error for invalid JSON")
	}
}

func TestParseManifest___WithVariants___ParsesVariants(t *testing.T) {
	manifestJSON, _ := json.Marshal(map[string]any{
		"bundle_version": "1.0",
		"plugin": map[string]any{
			"name":    "test-plugin",
			"version": "1.0.0",
		},
		"platforms": map[string]any{
			"linux-x86_64": map[string]any{
				"library":         "lib/linux-x86_64/libtest.so",
				"checksum":        "sha256:abc123",
				"default_variant": "release",
				"variants": map[string]any{
					"release": map[string]any{
						"library":  "lib/linux-x86_64/release/libtest.so",
						"checksum": "sha256:release123",
					},
					"debug": map[string]any{
						"library":  "lib/linux-x86_64/debug/libtest.so",
						"checksum": "sha256:debug456",
					},
				},
			},
		},
	})

	m, err := ParseManifest(manifestJSON)

	if err != nil {
		t.Fatalf("ParseManifest error: %v", err)
	}
	p := m.Platforms["linux-x86_64"]
	if len(p.Variants) != 2 {
		t.Fatalf("len(Variants) = %d, want 2", len(p.Variants))
	}
	if p.Variants["release"].Library != "lib/linux-x86_64/release/libtest.so" {
		t.Errorf("release library = %q", p.Variants["release"].Library)
	}
	if p.Variants["debug"].Checksum != "sha256:debug456" {
		t.Errorf("debug checksum = %q", p.Variants["debug"].Checksum)
	}
}

func TestPlatformInfo___GetLibrary___ReturnsVariantLibrary(t *testing.T) {
	p := &PlatformInfo{
		Library:  "default.so",
		Checksum: "sha256:default",
		Variants: map[string]*VariantInfo{
			"release": {Library: "release.so", Checksum: "sha256:release"},
		},
	}

	result := p.GetLibrary("release")

	if result != "release.so" {
		t.Errorf("GetLibrary(release) = %q", result)
	}
}

func TestPlatformInfo___GetLibrary___FallsBackToDefault(t *testing.T) {
	p := &PlatformInfo{
		Library:  "default.so",
		Checksum: "sha256:default",
	}

	result := p.GetLibrary("release")

	if result != "default.so" {
		t.Errorf("GetLibrary(release) = %q", result)
	}
}

func TestPlatformInfo___GetChecksum___ReturnsVariantChecksum(t *testing.T) {
	p := &PlatformInfo{
		Library:  "default.so",
		Checksum: "sha256:default",
		Variants: map[string]*VariantInfo{
			"release": {Library: "release.so", Checksum: "sha256:release"},
		},
	}

	result := p.GetChecksum("release")

	if result != "sha256:release" {
		t.Errorf("GetChecksum(release) = %q", result)
	}
}

func TestPlatformInfo___GetDefaultVariant___DefaultsToRelease(t *testing.T) {
	p := &PlatformInfo{Library: "lib.so", Checksum: "sha256:abc"}

	result := p.GetDefaultVariant()

	if result != "release" {
		t.Errorf("GetDefaultVariant() = %q", result)
	}
}

func TestPlatformInfo___GetDefaultVariant___ReturnsConfiguredVariant(t *testing.T) {
	p := &PlatformInfo{Library: "lib.so", Checksum: "sha256:abc", DefaultVariant: "debug"}

	result := p.GetDefaultVariant()

	if result != "debug" {
		t.Errorf("GetDefaultVariant() = %q", result)
	}
}

func TestPlatformInfo___ListVariants___ReturnsVariantNames(t *testing.T) {
	p := &PlatformInfo{
		Library:  "lib.so",
		Checksum: "sha256:abc",
		Variants: map[string]*VariantInfo{
			"release": {Library: "release.so", Checksum: "sha256:r"},
			"debug":   {Library: "debug.so", Checksum: "sha256:d"},
		},
	}

	result := p.ListVariants()

	if len(result) != 2 {
		t.Fatalf("len(ListVariants()) = %d", len(result))
	}
	// Sorted
	if result[0] != "debug" || result[1] != "release" {
		t.Errorf("ListVariants() = %v", result)
	}
}

func TestPlatformInfo___ListVariantsEmpty___ReturnsRelease(t *testing.T) {
	p := &PlatformInfo{Library: "lib.so", Checksum: "sha256:abc"}

	result := p.ListVariants()

	if len(result) != 1 || result[0] != "release" {
		t.Errorf("ListVariants() = %v", result)
	}
}

func TestBundleManifest___V11WithVariantBuildInfo___ParsesCorrectly(t *testing.T) {
	manifestJSON, _ := json.Marshal(map[string]any{
		"bundle_version": "1.1",
		"plugin": map[string]any{
			"name":    "test-plugin",
			"version": "1.0.0",
		},
		"platforms": map[string]any{
			"linux-x86_64": map[string]any{
				"library":  "lib/linux-x86_64/libtest.so",
				"checksum": "sha256:abc123",
				"variants": map[string]any{
					"release": map[string]any{
						"library":  "lib/linux-x86_64/release/libtest.so",
						"checksum": "sha256:release123",
						"build_info": map[string]any{
							"built_by": "GitHub Actions",
							"built_at": "2026-02-14T10:00:00Z",
							"host":     "x86_64-unknown-linux-gnu",
							"compiler": "rustc 1.90.0",
						},
						"schema_checksum": "sha256:variant_schema_hash",
					},
				},
			},
		},
		"build_info": map[string]any{
			"built_by": "local dev",
			"compiler": "rustc 1.89.0",
		},
		"schema_checksum": "sha256:top_level_schema_hash",
	})

	m, err := ParseManifest(manifestJSON)

	if err != nil {
		t.Fatalf("ParseManifest error: %v", err)
	}
	if m.BundleVersion != "1.1" {
		t.Errorf("BundleVersion = %q, want 1.1", m.BundleVersion)
	}
	if m.BuildInfo == nil {
		t.Fatal("top-level BuildInfo is nil")
	}
	if m.BuildInfo.BuiltBy != "local dev" {
		t.Errorf("BuildInfo.BuiltBy = %q", m.BuildInfo.BuiltBy)
	}
	if m.SchemaChecksum != "sha256:top_level_schema_hash" {
		t.Errorf("SchemaChecksum = %q", m.SchemaChecksum)
	}
	p := m.Platforms["linux-x86_64"]
	v := p.Variants["release"]
	if v.BuildInfo == nil {
		t.Fatal("variant BuildInfo is nil")
	}
	if v.BuildInfo.BuiltBy != "GitHub Actions" {
		t.Errorf("variant BuildInfo.BuiltBy = %q", v.BuildInfo.BuiltBy)
	}
	if v.SchemaChecksum != "sha256:variant_schema_hash" {
		t.Errorf("variant SchemaChecksum = %q", v.SchemaChecksum)
	}
}

func TestGetEffectiveBuildInfo___VariantOverridesTopLevel___ReturnsVariantBuildInfo(t *testing.T) {
	m := &BundleManifest{
		BundleVersion: "1.1",
		Plugin:        PluginInfo{Name: "test", Version: "1.0.0"},
		Platforms: map[string]*PlatformInfo{
			"linux-x86_64": {
				Library:  "lib.so",
				Checksum: "sha256:abc",
				Variants: map[string]*VariantInfo{
					"release": {
						Library:  "release.so",
						Checksum: "sha256:release",
						BuildInfo: &BuildInfo{
							BuiltBy:  "CI",
							Compiler: "rustc 1.90.0",
						},
					},
				},
			},
		},
		BuildInfo: &BuildInfo{
			BuiltBy:  "local",
			Compiler: "rustc 1.89.0",
		},
	}

	result := m.GetEffectiveBuildInfo("linux-x86_64", "release")

	if result == nil {
		t.Fatal("GetEffectiveBuildInfo returned nil")
	}
	if result.BuiltBy != "CI" {
		t.Errorf("BuiltBy = %q, want CI", result.BuiltBy)
	}
	if result.Compiler != "rustc 1.90.0" {
		t.Errorf("Compiler = %q, want rustc 1.90.0", result.Compiler)
	}
}

func TestGetEffectiveBuildInfo___NoVariantBuildInfo___FallsBackToTopLevel(t *testing.T) {
	m := &BundleManifest{
		BundleVersion: "1.1",
		Plugin:        PluginInfo{Name: "test", Version: "1.0.0"},
		Platforms: map[string]*PlatformInfo{
			"linux-x86_64": {
				Library:  "lib.so",
				Checksum: "sha256:abc",
				Variants: map[string]*VariantInfo{
					"release": {
						Library:  "release.so",
						Checksum: "sha256:release",
					},
				},
			},
		},
		BuildInfo: &BuildInfo{
			BuiltBy:  "local",
			Compiler: "rustc 1.89.0",
		},
	}

	result := m.GetEffectiveBuildInfo("linux-x86_64", "release")

	if result == nil {
		t.Fatal("GetEffectiveBuildInfo returned nil")
	}
	if result.BuiltBy != "local" {
		t.Errorf("BuiltBy = %q, want local", result.BuiltBy)
	}
}

func TestGetEffectiveSchemaChecksum___VariantOverridesTopLevel___ReturnsVariantChecksum(t *testing.T) {
	m := &BundleManifest{
		BundleVersion:  "1.1",
		Plugin:         PluginInfo{Name: "test", Version: "1.0.0"},
		SchemaChecksum: "sha256:top_level",
		Platforms: map[string]*PlatformInfo{
			"linux-x86_64": {
				Library:  "lib.so",
				Checksum: "sha256:abc",
				Variants: map[string]*VariantInfo{
					"release": {
						Library:        "release.so",
						Checksum:       "sha256:release",
						SchemaChecksum: "sha256:variant_level",
					},
				},
			},
		},
	}

	result := m.GetEffectiveSchemaChecksum("linux-x86_64", "release")

	if result != "sha256:variant_level" {
		t.Errorf("GetEffectiveSchemaChecksum = %q, want sha256:variant_level", result)
	}
}
