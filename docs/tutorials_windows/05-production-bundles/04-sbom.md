# Section 4: SBOM

In this section, you'll generate a Software Bill of Materials (SBOM) for your plugin.

## Why SBOM?

An SBOM lists all dependencies in your software:
- **Security** - Identify vulnerable dependencies
- **Compliance** - Meet regulatory requirements
- **Transparency** - Know what's in your software
- **License tracking** - Ensure license compliance

## Install cargo-sbom

```powershell
cargo install cargo-sbom
```

## Generate SBOM

```powershell
cd $env:USERPROFILE\rustbridge-workspace\json-plugin

# Generate CycloneDX format
cargo sbom --output-format cyclonedx > sbom.json
```

## Example SBOM Content

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.4",
  "version": 1,
  "metadata": {
    "component": {
      "name": "json-plugin",
      "version": "0.1.0",
      "type": "library"
    }
  },
  "components": [
    {
      "name": "rustbridge",
      "version": "0.7.0",
      "type": "library",
      "purl": "pkg:cargo/rustbridge@0.7.0"
    },
    {
      "name": "serde",
      "version": "1.0.193",
      "type": "library",
      "purl": "pkg:cargo/serde@1.0.193"
    }
    // ... more dependencies
  ]
}
```

## Embed SBOM in Bundle

```powershell
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --sbom sbom.json `
  --sign-key $env:USERPROFILE\.rustbridge\signing.key `
  --output json-plugin-0.1.0.rbp
```

## View SBOM

```powershell
rustbridge bundle list json-plugin-0.1.0.rbp
```

```
json-plugin-0.1.0.rbp
├── manifest.json
├── manifest.json.minisig
├── sbom.json                    ← SBOM file
├── sbom.json.minisig           ← SBOM signature
└── lib\
    └── ...
```

## Extract and Analyze SBOM

```powershell
# Extract SBOM
rustbridge bundle extract json-plugin-0.1.0.rbp --file sbom.json --output .

# View with jq (if installed)
Get-Content sbom.json | ConvertFrom-Json | ConvertTo-Json -Depth 10
```

## Vulnerability Scanning

Use tools like `grype` to scan the SBOM:

```powershell
# Install grype
scoop install grype

# Scan for vulnerabilities
grype sbom:sbom.json
```

## Complete Production Build Script

Create `build-production.ps1`:

```powershell
param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

Write-Host "=== Production Build: json-plugin v$Version ===" -ForegroundColor Cyan

# Get git info
$gitCommit = git rev-parse HEAD
$gitBranch = git rev-parse --abbrev-ref HEAD
$buildTime = Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ"

# Build
Write-Host "`nBuilding..." -ForegroundColor Yellow
cargo build --release

# Generate SBOM
Write-Host "`nGenerating SBOM..." -ForegroundColor Yellow
cargo sbom --output-format cyclonedx > sbom.json

# Generate schemas
Write-Host "`nGenerating schemas..." -ForegroundColor Yellow
if (-not (Test-Path schemas)) { mkdir schemas }
rustbridge schema generate --crate . --output schemas\

# Create bundle
Write-Host "`nCreating signed bundle..." -ForegroundColor Yellow
$keyPath = "$env:USERPROFILE\.rustbridge\signing.key"

rustbridge bundle create `
  --name json-plugin `
  --version $Version `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --schemas schemas\ `
  --sbom sbom.json `
  --sign-key $keyPath `
  --meta git.commit=$gitCommit `
  --meta git.branch=$gitBranch `
  --meta build.time=$buildTime `
  --output "json-plugin-$Version.rbp"

# Verify
Write-Host "`nVerifying bundle..." -ForegroundColor Yellow
$pubKey = "$env:USERPROFILE\.rustbridge\signing.key.pub"
rustbridge bundle verify --bundle "json-plugin-$Version.rbp" --public-key $pubKey

Write-Host "`n=== Build Complete ===" -ForegroundColor Green
rustbridge bundle info "json-plugin-$Version.rbp"
```

## Summary

You've learned to create production-ready bundles with:

1. **Code signing** - Cryptographic integrity verification
2. **JSON schemas** - API documentation and validation
3. **Build metadata** - Traceability and auditing
4. **SBOM** - Dependency transparency

## What's Next?

Continue to [Chapter 6: Native Builds](../06-native-builds/README.md) to build for different Windows platforms.
