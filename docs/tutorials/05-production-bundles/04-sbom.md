# Section 4: SBOM (Software Bill of Materials)

In this section, you'll add a Software Bill of Materials to your bundle for dependency transparency and compliance.

## What is SBOM?

A Software Bill of Materials (SBOM) is a formal record of all components in your software:

- Direct dependencies
- Transitive dependencies
- Versions and licenses
- Known vulnerabilities (optional)

## Why Include SBOM?

- **Security**: Identify vulnerable dependencies
- **Compliance**: Meet regulatory requirements (US Executive Order 14028, EU CRA)
- **Supply Chain**: Understand what's in your software
- **License Audit**: Track open-source license obligations

## Supported Formats

rustbridge supports two industry-standard SBOM formats:

| Format                              | File             | Use Case                                 |
|-------------------------------------|------------------|------------------------------------------|
| [CycloneDX](https://cyclonedx.org/) | `sbom.cdx.json`  | Security-focused, vulnerability tracking |
| [SPDX](https://spdx.dev/)           | `sbom.spdx.json` | License-focused, compliance              |

## Generate SBOM for Rust Projects

### Using cargo-sbom (CycloneDX)

Install the tool:

```bash
cargo install cargo-sbom
```

Generate SBOM:

```bash
cd ~/rustbridge-workspace/json-plugin
cargo sbom --output-format cyclone_dx_json_1_6 > sbom.cdx.json
```

> **Note**: Use `cargo sbom --help` to see all available formats. CycloneDX 1.6 is recommended.

### Using cargo-sbom (SPDX)

`cargo-sbom` can also generate SPDX format (it's actually the default):

```bash
cargo sbom --output-format spdx_json_2_3 > sbom.spdx.json
```

> **Note**: You only need `cargo-sbom` installed—it supports both CycloneDX and SPDX formats.

## Include SBOM in Bundle

Use the `--sbom` flag:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --sbom sbom.cdx.json:sbom.cdx.json \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-1.0.0.rbp
```

Include both formats:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --sbom sbom.cdx.json:sbom.cdx.json \
  --sbom sbom.spdx.json:sbom.spdx.json \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-1.0.0.rbp
```

## Verify SBOM Inclusion

```bash
rustbridge bundle list json-plugin-1.0.0.rbp
```

```
json-plugin-1.0.0.rbp
├── manifest.json
├── manifest.json.minisig
├── lib/
│   └── linux-x86_64/
│       └── release/
│           ├── libjson_plugin.so
│           └── libjson_plugin.so.minisig
└── sbom/
    ├── sbom.cdx.json                    ← CycloneDX SBOM
    └── sbom.spdx.json                   ← SPDX SBOM
```

The manifest references the SBOM files:

```json
{
  "sbom": {
    "cyclonedx": "sbom/sbom.cdx.json",
    "spdx": "sbom/sbom.spdx.json"
  }
}
```

## Extract SBOM from Bundle

### Command Line

```bash
unzip -j json-plugin-1.0.0.rbp "sbom/*" -d ./
```

### Java

```java
import java.util.zip.ZipFile;
import java.util.zip.ZipEntry;

try (var zipFile = new ZipFile("json-plugin-1.0.0.rbp")) {
    // Check manifest for SBOM path
    ZipEntry sbomEntry = zipFile.getEntry("sbom/sbom.cdx.json");
    if (sbomEntry != null) {
        try (var stream = zipFile.getInputStream(sbomEntry)) {
            String sbom = new String(stream.readAllBytes());
            System.out.println(sbom);
        }
    }
}
```

### Python

```python
import zipfile

with zipfile.ZipFile("json-plugin-1.0.0.rbp", "r") as zf:
    # List SBOM files
    sbom_files = [f for f in zf.namelist() if f.startswith("sbom/")]

    for sbom_file in sbom_files:
        sbom_content = zf.read(sbom_file).decode("utf-8")
        print(f"{sbom_file}:\n{sbom_content}")
```

## CycloneDX Format Example

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "version": 1,
  "metadata": {
    "timestamp": "2025-01-30T14:32:00Z",
    "tools": [
      {
        "vendor": "CycloneDX",
        "name": "cargo-sbom",
        "version": "0.9.0"
      }
    ],
    "component": {
      "type": "application",
      "name": "json-plugin",
      "version": "1.0.0"
    }
  },
  "components": [
    {
      "type": "library",
      "name": "serde",
      "version": "1.0.217",
      "purl": "pkg:cargo/serde@1.0.217",
      "licenses": [
        {
          "license": {
            "id": "MIT"
          }
        },
        {
          "license": {
            "id": "Apache-2.0"
          }
        }
      ]
    },
    {
      "type": "library",
      "name": "serde_json",
      "version": "1.0.138",
      "purl": "pkg:cargo/serde_json@1.0.138"
    }
  ]
}
```

## Notices File

For license compliance, include a NOTICES file with full license texts:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --sbom sbom.cdx.json:sbom.cdx.json \
  --notices NOTICES.txt \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-1.0.0.rbp
```

Generate NOTICES.txt with cargo-about:

```bash
cargo install cargo-about
cargo about generate about.hbs > NOTICES.txt
```

## Summary

You've learned to:

- Generate SBOM files with cargo-sbom or cargo-spdx
- Include SBOM in bundles with `--sbom`
- Extract and analyze SBOM from bundles
- Scan for vulnerabilities using SBOM
- Audit license compliance

## Chapter Summary

In this chapter, you've built a production-ready bundle with:

| Feature        | Purpose                                |
|----------------|----------------------------------------|
| Code signing   | Verify authenticity and integrity      |
| JSON schemas   | Document and validate messages         |
| Build metadata | Track provenance and reproducibility   |
| SBOM           | Dependency transparency and compliance |

## Next Steps

Continue to [Chapter 6: Cross-Compilation](../06-cross-compilation/README.md) to learn about building multi-platform
bundles for Linux, macOS, and Windows.
