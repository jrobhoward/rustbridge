# Section 1: Code Signing

In this section, you'll generate a signing key and create signed bundles that consumers can verify.

## Why Sign Bundles?

Code signing provides:

- **Authenticity**: Verify the bundle came from a trusted source
- **Integrity**: Detect tampering or corruption
- **Non-repudiation**: Prove who built the bundle

rustbridge uses [minisign](https://jedisct1.github.io/minisign/), a simple and secure signature scheme.

## Generate a Signing Key

Create a new key pair (one-time setup):

```bash
rustbridge keygen
```

This creates two files:

- `~/.rustbridge/signing.key` - Secret key (keep secure!)
- `~/.rustbridge/signing.pub` - Public key (distribute to consumers)

You'll be prompted for a password to protect the secret key.

### Custom Key Location

```bash
rustbridge keygen --output my-project.key
```

This creates `my-project.key` (secret) and `my-project.pub` (public).

## Create a Signed Bundle

Use the `--sign-key` flag when creating bundles:

```bash
cd ~/rustbridge-workspace/json-plugin

cargo build --release

rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-1.0.0.rbp
```

You'll be prompted for the key password.

## What Gets Signed?

The bundle now includes signature files:

```bash
rustbridge bundle list json-plugin-1.0.0.rbp
```

```
json-plugin-1.0.0.rbp
├── manifest.json
├── manifest.json.minisig              ← Manifest signature
└── lib/
    └── linux-x86_64/
        └── release/
            ├── libjson_plugin.so
            └── libjson_plugin.so.minisig   ← Library signature
```

The manifest also includes the public key:

```json
{
  "public_key": "RWTxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  ...
}
```

## Verify a Signed Bundle

### Java

```java
// Verification enabled by default - throws if invalid
var bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .build();

// Explicit verification with known public key
var bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .verifySignatures(true)
    .publicKey("RWTxxxxxxxx...")  // Override embedded key
    .build();

// Disable verification
var bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .verifySignatures(false)
    .build();
```

### Kotlin

```kotlin
// Verification enabled by default
val plugin = BundleLoader.load("json-plugin-1.0.0.rbp")

// Disable verification
val plugin = BundleLoader.load("json-plugin-1.0.0.rbp", verifySignatures = false)
```

### Python

```python
from rustbridge import BundleLoader

# Verification enabled by default
plugin = BundleLoader.load("json-plugin-1.0.0.rbp")

# Disable verification
plugin = BundleLoader.load("json-plugin-1.0.0.rbp", verify_signatures=False)
```

### C#

```csharp
// Verification enabled by default
var plugin = BundleLoader.Load("json-plugin-1.0.0.rbp");

// Disable verification (development only!)
var plugin = BundleLoader.Load("json-plugin-1.0.0.rbp", verifySignatures: false);
```

## Distributing Your Public Key

Consumers need your public key to verify bundles. Options:

### 1. Embedded in Bundle

The public key is stored in `manifest.json`. Consumers either:

- Trust the first bundle they receive
- Compare `manifest.json`'s key against a public or out-of-band key

### 2. Out-of-Band Distribution

Distribute your public key through a trusted channel:

- Project README or documentation
- Package registry metadata
- HTTPS endpoint

```bash
# Display your public key
cat ~/.rustbridge/signing.pub
```

### 3. Key Pinning

Once consumers know your public key, they can pin it:

```java
var bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .publicKey("RWTxxxxxxxx...")  // Pinned key
    .build();
```

## Key Management Best Practices

| Practice                 | Description                               |
|--------------------------|-------------------------------------------|
| Protect the secret key   | Never commit to version control           |
| Use strong passwords     | Protect the key file itself               |
| Rotate keys periodically | Generate new keys for major versions      |
| Back up securely         | Store encrypted backups offline           |
| Separate dev/prod keys   | Use different keys for testing vs release |

## Summary

You've learned to:

- Generate minisign key pairs with `rustbridge keygen`
- Create signed bundles with `--sign-key`
- Verify signatures in consumer code
- Distribute public keys securely

## What's Next?

In the next section, you'll embed JSON schemas for message validation.

[Continue to Section 2: JSON Schemas →](./02-json-schemas.md)
