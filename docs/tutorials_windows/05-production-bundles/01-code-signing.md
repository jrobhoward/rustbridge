# Section 1: Code Signing

In this section, you'll set up code signing for your bundles using minisign.

## Install minisign

Download minisign for Windows from the [minisign releases](https://github.com/jedisct1/minisign/releases).

Or use Chocolatey:

```powershell
choco install minisign
```

Or use Scoop:

```powershell
scoop install minisign
```

## Generate a Signing Key

```powershell
# Create a directory for keys
mkdir $env:USERPROFILE\.rustbridge

# Generate a new key pair
rustbridge keygen --output $env:USERPROFILE\.rustbridge\signing.key
```

This creates two files:
- `signing.key` - Private key (keep secret!)
- `signing.key.pub` - Public key (distribute to users)

## Sign a Bundle

```powershell
cd $env:USERPROFILE\rustbridge-workspace\json-plugin

# Build the plugin
cargo build --release

# Create a signed bundle
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --sign-key $env:USERPROFILE\.rustbridge\signing.key `
  --output json-plugin-0.1.0.rbp
```

## Verify Bundle Contents

```powershell
rustbridge bundle list json-plugin-0.1.0.rbp
```

```
json-plugin-0.1.0.rbp
├── manifest.json
├── manifest.json.minisig        ← Signature file
└── lib\
    └── windows-x86_64\
        └── release\
            ├── json_plugin.dll
            └── json_plugin.dll.minisig  ← DLL signature
```

## Verify Signatures

```powershell
rustbridge bundle verify `
  --bundle json-plugin-0.1.0.rbp `
  --public-key $env:USERPROFILE\.rustbridge\signing.key.pub
```

Output:

```
✓ manifest.json: signature valid
✓ lib/windows-x86_64/release/json_plugin.dll: signature valid
Bundle verification: PASSED
```

## Configuring Consumers to Verify

In your consumer code:

```kotlin
// Kotlin
val bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-0.1.0.rbp")
    .verifySignatures(true)
    .publicKey(publicKeyPath)
    .build()
```

```csharp
// C#
var bundleLoader = BundleLoader.Create()
    .WithBundlePath("json-plugin-0.1.0.rbp")
    .WithSignatureVerification(true)
    .WithPublicKey(publicKeyPath)
    .Build();
```

```python
# Python
loader = BundleLoader(
    verify_signatures=True,
    public_key=public_key_path
)
```

## Key Management Best Practices

1. **Never commit private keys** - Add `*.key` to `.gitignore`
2. **Backup private keys securely** - Use a password manager or HSM
3. **Rotate keys periodically** - Generate new keys annually
4. **Distribute public keys via HTTPS** - Host on your website

## What's Next?

In the next section, you'll add JSON schemas to your bundle.

[Continue to Section 2: JSON Schemas →](./02-json-schemas.md)
