# Section 3: Build Metadata

In this section, you'll add build provenance information to your bundles for traceability.

## Why Include Build Metadata?

Build metadata provides:

- **Traceability**: Know exactly which commit produced a bundle
- **Reproducibility**: Recreate builds with the same configuration
- **Debugging**: Correlate issues with specific builds
- **Compliance**: Meet audit and regulatory requirements

## Automatic Metadata Collection

By default, `rustbridge bundle create` automatically collects:

- Build timestamp
- Host platform
- Rust compiler version
- rustbridge CLI version
- Git information (if in a git repository)

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --output json-plugin-1.0.0.rbp
```

## View Build Metadata

```bash
rustbridge bundle info --show-build json-plugin-1.0.0.rbp
```

```
json-plugin 1.0.0
─────────────────

Build Information:
  Built at:    2025-01-30T14:32:00Z
  Built by:    jhoward
  Host:        x86_64-unknown-linux-gnu
  Compiler:    rustc 1.90.0
  rustbridge:  0.6.1

Git Information:
  Commit:      a1b2c3d4e5f6789012345678901234567890abcd
  Branch:      main
  Tag:         v1.0.0
  Dirty:       false
```

## Manifest Structure

The build information is stored in `manifest.json`:

```json
{
  "build_info": {
    "built_by": "jhoward",
    "built_at": "2025-01-30T14:32:00Z",
    "host": "x86_64-unknown-linux-gnu",
    "compiler": "rustc 1.90.0",
    "rustbridge_version": "0.6.1",
    "git": {
      "commit": "a1b2c3d4e5f6789012345678901234567890abcd",
      "branch": "main",
      "tag": "v1.0.0",
      "dirty": false
    }
  }
}
```

## Disable Automatic Metadata

For reproducible builds or privacy, skip metadata collection:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --no-metadata \
  --output json-plugin-1.0.0.rbp
```

## Git Information

### Clean vs Dirty Builds

The `dirty` flag indicates uncommitted changes:

| State | `dirty` | Meaning                              |
|-------|---------|--------------------------------------|
| Clean | `false` | Working directory matches the commit |
| Dirty | `true`  | Uncommitted changes present          |

**Best practice**: Only release bundles built from clean git state.

### Tag Detection

If the current commit has a tag, it's included:

```json
{
  "git": {
    "commit": "a1b2c3d4...",
    "tag": "v1.0.0"
  }
}
```

### Non-Git Projects

If not in a git repository, the `git` section is omitted:

```json
{
  "build_info": {
    "built_by": "jhoward",
    "built_at": "2025-01-30T14:32:00Z",
    "host": "x86_64-unknown-linux-gnu"
  }
}
```

## Custom Metadata

The `--metadata` flag lets you add arbitrary key/value pairs for informational purposes. This is useful for data that
rustbridge can't automatically detect, like source repository URLs.

### Adding Repository URL

Since git doesn't inherently know the remote URL (you might have multiple remotes, or build from a tarball), you can
explicitly add it:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --metadata repository=https://github.com/jrobhoward/rustbridge \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-1.0.0.rbp
```

### Multiple Custom Fields

You can add any key/value pairs that are useful for your workflow:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --metadata repository=https://github.com/jrobhoward/rustbridge \
  --metadata ci_job_id=12345 \
  --metadata pipeline=release \
  --output json-plugin-1.0.0.rbp
```

### Custom Metadata in Manifest

Custom metadata appears in `build_info.custom`:

```json
{
  "build_info": {
    "built_at": "2025-01-30T14:32:00Z",
    "compiler": "rustc 1.90.0",
    "git": {
      "commit": "a1b2c3d4...",
      "branch": "main"
    },
    "custom": {
      "repository": "https://github.com/jrobhoward/rustbridge",
      "ci_job_id": "12345",
      "pipeline": "release"
    }
  }
}
```

### Viewing Custom Metadata

```bash
rustbridge bundle list --show-build json-plugin-1.0.0.rbp
```

```
Build Info:
  Built at: 2025-01-30T14:32:00Z
  Compiler: rustc 1.90.0
  Git commit: a1b2c3d4...
  Custom metadata:
    repository: https://github.com/jrobhoward/rustbridge
    ci_job_id: 12345
```

> **Note**: Custom metadata is purely informational. rustbridge doesn't validate or use these values—they're for your
> own documentation and tooling purposes.

## Variant-Specific Build Info

Each library variant can have its own build metadata:

```json
{
  "platforms": {
    "linux-x86_64": {
      "variants": {
        "release": {
          "library": "lib/linux-x86_64/release/libplugin.so",
          "checksum": "sha256:...",
          "build": {
            "profile": "release",
            "opt_level": "3",
            "lto": true,
            "features": [
              "json"
            ]
          }
        },
        "debug": {
          "library": "lib/linux-x86_64/debug/libplugin.so",
          "checksum": "sha256:...",
          "build": {
            "profile": "debug",
            "opt_level": "0",
            "debug_assertions": true
          }
        }
      }
    }
  }
}
```

## Accessing Metadata in Consumer Code

### Java

```java
var bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .build();

var buildInfo = bundleLoader.getBuildInfo();
System.out.println("Built at: " + buildInfo.getBuiltAt());
System.out.println("Git commit: " + buildInfo.getGitCommit());

// Access custom metadata by key
Map<String, String> custom = buildInfo.getCustomMetadata();
String repository = custom.get("repository");
System.out.println("Repository: " + repository);

// Iterate over all custom metadata
for (var entry : custom.entrySet()) {
    System.out.println(entry.getKey() + ": " + entry.getValue());
}
```

### Python

```python
from rustbridge import BundleLoader

bundle = BundleLoader("json-plugin-1.0.0.rbp")
build_info = bundle.get_build_info()

print(f"Built at: {build_info['built_at']}")
print(f"Git commit: {build_info['git']['commit']}")

# Access custom metadata by key
custom = build_info.get('custom', {})
repository = custom.get('repository')
print(f"Repository: {repository}")

# Iterate over all custom metadata
for key, value in custom.items():
    print(f"{key}: {value}")
```

## JSON Output for Scripting

```bash
rustbridge bundle info --json json-plugin-1.0.0.rbp | jq '.build_info'
```

```json
{
  "built_by": "jhoward",
  "built_at": "2025-01-30T14:32:00Z",
  "host": "x86_64-unknown-linux-gnu",
  "compiler": "rustc 1.90.0",
  "rustbridge_version": "0.6.1",
  "git": {
    "commit": "a1b2c3d4e5f6789012345678901234567890abcd",
    "branch": "main",
    "dirty": false
  },
  "custom": {
    "repository": "https://github.com/jrobhoward/rustbridge",
    "ci_job_id": "12345"
  }
}
```

## Plugin License File

The `--license` flag lets you include your plugin's own LICENSE file in the bundle. This is separate from:

- The SPDX identifier in the manifest (`plugin.license`)
- Third-party notices (`--notices`)
- Dependency license info in the SBOM

### Including Your License

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --license LICENSE \
  --metadata repository=https://github.com/jrobhoward/rustbridge \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-1.0.0.rbp
```

The license file is stored in `legal/LICENSE` within the bundle:

```
json-plugin-1.0.0.rbp
├── manifest.json
├── legal/
│   └── LICENSE               ← Your plugin's license
├── lib/
│   └── linux-x86_64/
│       └── release/
│           └── libjson_plugin.so
└── ...
```

### Manifest Structure

The license file path is recorded in the manifest:

```json
{
  "plugin": {
    "name": "json-plugin",
    "version": "1.0.0",
    "license": "MIT"
  },
  "license_file": "legal/LICENSE"
}
```

### Accessing the License Programmatically

#### Java

```java
var bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .build();

// Get license file path from manifest
String licensePath = bundleLoader.getManifest().getLicenseFile();

// Read the license content
String licenseText = bundleLoader.readFileAsString(licensePath);
System.out.println(licenseText);
```

#### Python

```python
from rustbridge import BundleLoader

bundle = BundleLoader("json-plugin-1.0.0.rbp")

# Get license file path from manifest
license_path = bundle.manifest.get('license_file')

# Read the license content
if license_path:
    license_text = bundle.read_file(license_path).decode('utf-8')
    print(license_text)
```

### When to Use

| Flag        | Purpose                 | Content                                                |
|-------------|-------------------------|--------------------------------------------------------|
| `--license` | Plugin's own license    | Full license text (MIT, Apache-2.0, proprietary, etc.) |
| `--notices` | Third-party attribution | Apache NOTICE-style acknowledgments                    |
| `--sbom`    | Dependency inventory    | CycloneDX/SPDX with dependency licenses                |

> **Best practice**: Include `--license` for both open-source and proprietary plugins to make licensing terms clear to
> consumers.

## Summary

You've learned to:

- Understand automatic build metadata collection
- View metadata with `rustbridge bundle list --show-build`
- Add custom metadata with `--metadata KEY=VALUE` (e.g., repository URL)
- Control metadata with `--no-metadata`
- Include your plugin's license with `--license`
- Access metadata programmatically in consumer code

## What's Next?

In the next section, you'll add a Software Bill of Materials (SBOM) for dependency transparency.

[Continue to Section 4: SBOM →](./04-sbom.md)
