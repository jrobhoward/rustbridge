# Chapter 5: Production Bundles

In this chapter, you'll learn how to create production-ready plugin bundles with code signing, schemas, build metadata,
and Software Bill of Materials (SBOM).

## What You'll Learn

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Production Bundle Features                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  my-plugin-1.0.0.rbp                                                        │
│  ────────────────────                                                       │
│  ├── manifest.json              ← Plugin metadata + checksums               │
│  ├── manifest.json.minisig      ← Code signature (minisign)                 │
│  ├── lib/                                                                   │
│  │   └── linux-x86_64/                                                      │
│  │       └── release/                                                       │
│  │           ├── libplugin.so                                               │
│  │           └── libplugin.so.minisig                                       │
│  ├── schema/                                                                │
│  │   └── messages.json          ← JSON Schema for validation                │
│  └── sbom/                                                                  │
│      └── sbom.cdx.json          ← Software Bill of Materials                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Sections

### [01: Code Signing](./01-code-signing.md)

Generate a signing key and create signed bundles for authenticity verification.

### [02: JSON Schemas](./02-json-schemas.md)

Embed JSON schemas for message validation and documentation.

### [03: Build Metadata](./03-build-metadata.md)

Include build provenance: git commit, compiler version, timestamps.

### [04: SBOM](./04-sbom.md)

Add Software Bill of Materials for dependency transparency.

## Prerequisites

- Completed Chapter 3 (json-plugin) or have a working plugin
- rustbridge CLI installed
- Basic familiarity with command-line tools

## Why Production Bundles?

| Feature                | Development | Production              |
|------------------------|-------------|-------------------------|
| Code signing           | Skip        | Required                |
| Signature verification | Disabled    | Enabled                 |
| Build metadata         | Optional    | Included                |
| SBOM                   | Optional    | Required for compliance |
| JSON schemas           | Optional    | Recommended             |

## Next Steps

After completing this chapter, continue to [Chapter 6: Cross-Compilation](../06-cross-compilation/README.md) to build
multi-platform bundles for Linux, macOS, and Windows.
