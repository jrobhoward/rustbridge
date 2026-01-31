# Section 3: Build Metadata

In this section, you'll embed build information in your bundles for traceability.

## Why Build Metadata?

Build metadata helps with:
- **Debugging** - Know exactly what code is running
- **Auditing** - Track which builds are deployed
- **Reproducibility** - Rebuild the exact same version
- **Compliance** - Satisfy audit requirements

## Add Build Metadata

```powershell
cd $env:USERPROFILE\rustbridge-workspace\json-plugin

# Get git info
$gitCommit = git rev-parse HEAD
$gitBranch = git rev-parse --abbrev-ref HEAD

# Create bundle with metadata
rustbridge bundle create `
  --name json-plugin `
  --version 1.0.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --sign-key $env:USERPROFILE\.rustbridge\signing.key `
  --meta git.commit=$gitCommit `
  --meta git.branch=$gitBranch `
  --meta build.time=$(Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ") `
  --meta build.host=$env:COMPUTERNAME `
  --output json-plugin-1.0.0.rbp
```

## View Metadata

```powershell
rustbridge bundle info json-plugin-1.0.0.rbp
```

```yaml
name: json-plugin
version: 1.0.0
platforms:
  - windows-x86_64
metadata:
  git.commit: abc123def456...
  git.branch: main
  build.time: 2024-01-15T10:30:00Z
  build.host: BUILD-PC-01
signatures:
  manifest: valid
  libraries: valid
```

## Automated Build Script

Create `build.ps1`:

```powershell
param(
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

# Get git information
$gitCommit = git rev-parse HEAD
$gitBranch = git rev-parse --abbrev-ref HEAD
$gitDirty = if ((git status --porcelain) -ne $null) { "-dirty" } else { "" }
$buildTime = Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ"

Write-Host "Building json-plugin v$Version"
Write-Host "  Commit: $gitCommit$gitDirty"
Write-Host "  Branch: $gitBranch"

# Build
cargo build --release
if ($LASTEXITCODE -ne 0) { exit 1 }

# Create bundle
$keyPath = "$env:USERPROFILE\.rustbridge\signing.key"

rustbridge bundle create `
  --name json-plugin `
  --version $Version `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --sign-key $keyPath `
  --meta git.commit="$gitCommit$gitDirty" `
  --meta git.branch=$gitBranch `
  --meta build.time=$buildTime `
  --meta build.host=$env:COMPUTERNAME `
  --meta build.user=$env:USERNAME `
  --output "json-plugin-$Version.rbp"

if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "`nCreated json-plugin-$Version.rbp"

# Show info
rustbridge bundle info "json-plugin-$Version.rbp"
```

## Run the Build Script

```powershell
.\build.ps1 -Version "1.0.1"
```

## Accessing Metadata in Consumers

```kotlin
// Kotlin
val bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .build()

val manifest = bundleLoader.manifest()
println("Version: ${manifest.version}")
println("Commit: ${manifest.metadata["git.commit"]}")
println("Built: ${manifest.metadata["build.time"]}")
```

## CI/CD Integration

In GitHub Actions:

```yaml
- name: Build Bundle
  run: |
    rustbridge bundle create `
      --name json-plugin `
      --version ${{ github.ref_name }} `
      --lib windows-x86_64:target/release/json_plugin.dll `
      --sign-key secrets/signing.key `
      --meta git.commit=${{ github.sha }} `
      --meta git.branch=${{ github.ref_name }} `
      --meta ci.run=${{ github.run_id }} `
      --output json-plugin-${{ github.ref_name }}.rbp
```

## What's Next?

In the next section, you'll generate a Software Bill of Materials.

[Continue to Section 4: SBOM →](./04-sbom.md)
