# Chapter 5: Production Bundles

In this chapter, you'll prepare your plugins for production deployment with code signing, schemas, and metadata.

## What You'll Build

Production-ready bundles with:
- Cryptographic signatures (minisign)
- JSON schemas for validation
- Build metadata (git commit, build time)
- Software Bill of Materials (SBOM)

## Prerequisites

- Completed [Chapter 3: JSON Plugin](../03-json-plugin/README.md)
- `minisign` tool installed (for signing)

## Sections

### [01: Code Signing](./01-code-signing.md)
Sign bundles with minisign for integrity verification.

### [02: JSON Schemas](./02-json-schemas.md)
Generate and embed JSON schemas.

### [03: Build Metadata](./03-build-metadata.md)
Embed git commit and build information.

### [04: SBOM](./04-sbom.md)
Generate Software Bill of Materials.

## What You'll Learn

- Setting up code signing keys
- Verifying bundle signatures
- Schema generation and validation
- Build reproducibility

## Next Steps

After completing this chapter, continue to [Chapter 6: Native Builds](../06-native-builds/README.md) to build for different Windows platforms.
