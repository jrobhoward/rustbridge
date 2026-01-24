//! C header generation from Rust `#[repr(C)]` structs
//!
//! This module parses Rust source files and generates C header files
//! containing equivalent struct definitions for FFI binary transport.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::{Attribute, Fields, Item, Type};

/// Type mapping from Rust to C
struct TypeMapping {
    rust_type: &'static str,
    c_type: &'static str,
}

const TYPE_MAPPINGS: &[TypeMapping] = &[
    TypeMapping {
        rust_type: "u8",
        c_type: "uint8_t",
    },
    TypeMapping {
        rust_type: "i8",
        c_type: "int8_t",
    },
    TypeMapping {
        rust_type: "u16",
        c_type: "uint16_t",
    },
    TypeMapping {
        rust_type: "i16",
        c_type: "int16_t",
    },
    TypeMapping {
        rust_type: "u32",
        c_type: "uint32_t",
    },
    TypeMapping {
        rust_type: "i32",
        c_type: "int32_t",
    },
    TypeMapping {
        rust_type: "u64",
        c_type: "uint64_t",
    },
    TypeMapping {
        rust_type: "i64",
        c_type: "int64_t",
    },
    TypeMapping {
        rust_type: "usize",
        c_type: "size_t",
    },
    TypeMapping {
        rust_type: "isize",
        c_type: "ptrdiff_t",
    },
    TypeMapping {
        rust_type: "f32",
        c_type: "float",
    },
    TypeMapping {
        rust_type: "f64",
        c_type: "double",
    },
    TypeMapping {
        rust_type: "bool",
        c_type: "bool",
    },
];

/// A parsed `#[repr(C)]` struct
#[derive(Debug)]
struct CStruct {
    name: String,
    fields: Vec<CField>,
    doc_comment: Option<String>,
}

/// A field within a C struct
#[derive(Debug)]
struct CField {
    name: String,
    c_type: String,
    doc_comment: Option<String>,
}

/// A constant definition (e.g., message IDs)
#[derive(Debug)]
struct CConstant {
    name: String,
    c_type: String,
    value: String,
    doc_comment: Option<String>,
}

/// Parse a Rust source file and extract `#[repr(C)]` structs and constants
fn parse_rust_file(source_path: &Path) -> Result<(Vec<CStruct>, Vec<CConstant>)> {
    let source = fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read source file: {}", source_path.display()))?;

    let ast = syn::parse_file(&source)
        .with_context(|| format!("Failed to parse Rust file: {}", source_path.display()))?;

    let mut structs = Vec::new();
    let mut constants = Vec::new();

    for item in ast.items {
        match item {
            Item::Struct(s) => {
                if is_repr_c(&s.attrs) {
                    if let Some(c_struct) = parse_struct(&s) {
                        structs.push(c_struct);
                    }
                }
            }
            Item::Const(c) => {
                if let Some(constant) = parse_constant(&c) {
                    constants.push(constant);
                }
            }
            _ => {}
        }
    }

    Ok((structs, constants))
}

/// Check if a struct has `#[repr(C)]` attribute
fn is_repr_c(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("repr") {
            if let Ok(nested) = attr.parse_args::<syn::Ident>() {
                return nested == "C";
            }
        }
        false
    })
}

/// Extract doc comment from attributes
fn extract_doc_comment(attrs: &[Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &meta.value
                    {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect();

    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

/// Parse a syn struct into our CStruct representation
fn parse_struct(s: &syn::ItemStruct) -> Option<CStruct> {
    let name = s.ident.to_string();
    let doc_comment = extract_doc_comment(&s.attrs);

    let fields = match &s.fields {
        Fields::Named(named) => named
            .named
            .iter()
            .filter_map(|field| {
                let field_name = field.ident.as_ref()?.to_string();
                let c_type = rust_type_to_c(&field.ty)?;
                let doc_comment = extract_doc_comment(&field.attrs);

                Some(CField {
                    name: field_name,
                    c_type,
                    doc_comment,
                })
            })
            .collect(),
        _ => return None, // Only support named fields
    };

    Some(CStruct {
        name,
        fields,
        doc_comment,
    })
}

/// Parse a constant definition
fn parse_constant(c: &syn::ItemConst) -> Option<CConstant> {
    let name = c.ident.to_string();

    // Only export MSG_ prefixed constants (message IDs)
    if !name.starts_with("MSG_") {
        return None;
    }

    let c_type = rust_type_to_c(&c.ty)?;

    // Extract the literal value
    let value = match c.expr.as_ref() {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(i),
            ..
        }) => i.base10_digits().to_string(),
        _ => return None,
    };

    let doc_comment = extract_doc_comment(&c.attrs);

    Some(CConstant {
        name,
        c_type,
        value,
        doc_comment,
    })
}

/// Convert a Rust type to its C equivalent
fn rust_type_to_c(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => {
            let ident = path.path.segments.last()?.ident.to_string();
            TYPE_MAPPINGS
                .iter()
                .find(|m| m.rust_type == ident)
                .map(|m| m.c_type.to_string())
        }
        Type::Array(arr) => {
            // Handle fixed-size arrays like [u8; 64]
            let elem_type = rust_type_to_c(&arr.elem)?;
            let len = match &arr.len {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(i),
                    ..
                }) => i.base10_digits().to_string(),
                _ => return None,
            };
            Some(format!("{elem_type}[{len}]"))
        }
        Type::Ptr(ptr) => {
            let elem_type = rust_type_to_c(&ptr.elem)?;
            if ptr.mutability.is_some() {
                Some(format!("{elem_type}*"))
            } else {
                Some(format!("const {elem_type}*"))
            }
        }
        _ => None,
    }
}

/// Generate C header content from parsed structs and constants
fn generate_header(structs: &[CStruct], constants: &[CConstant], source_name: &str) -> String {
    let mut output = String::new();

    // Header guard
    let guard_name = source_name.to_uppercase().replace(['.', '-'], "_");
    output.push_str("// Auto-generated by rustbridge generate-header\n");
    output.push_str(&format!("// Source: {source_name}\n"));
    output.push_str("// DO NOT EDIT - regenerate with: rustbridge generate-header\n\n");
    output.push_str(&format!("#ifndef {guard_name}_H\n"));
    output.push_str(&format!("#define {guard_name}_H\n\n"));
    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdbool.h>\n");
    output.push_str("#include <stddef.h>\n\n");
    output.push_str("#ifdef __cplusplus\n");
    output.push_str("extern \"C\" {\n");
    output.push_str("#endif\n\n");

    // Constants
    if !constants.is_empty() {
        output.push_str("// Message IDs\n");
        for constant in constants {
            if let Some(doc) = &constant.doc_comment {
                output.push_str(&format!("/** {} */\n", doc));
            }
            output.push_str(&format!(
                "#define {} (({}){})\n",
                constant.name, constant.c_type, constant.value
            ));
        }
        output.push('\n');
    }

    // Structs
    for c_struct in structs {
        if let Some(doc) = &c_struct.doc_comment {
            output.push_str("/**\n");
            for line in doc.lines() {
                output.push_str(&format!(" * {line}\n"));
            }
            output.push_str(" */\n");
        }
        output.push_str(&format!("typedef struct {} {{\n", c_struct.name));

        for field in &c_struct.fields {
            if let Some(doc) = &field.doc_comment {
                output.push_str(&format!("    /** {} */\n", doc));
            }

            // Handle array types specially (C syntax: type name[size])
            if field.c_type.contains('[') {
                let parts: Vec<&str> = field.c_type.splitn(2, '[').collect();
                output.push_str(&format!("    {} {}[{};\n", parts[0], field.name, parts[1]));
            } else {
                output.push_str(&format!("    {} {};\n", field.c_type, field.name));
            }
        }

        output.push_str(&format!("}} {};\n\n", c_struct.name));
    }

    // Footer
    output.push_str("#ifdef __cplusplus\n");
    output.push_str("}\n");
    output.push_str("#endif\n\n");
    output.push_str(&format!("#endif // {guard_name}_H\n"));

    output
}

/// Run the header generation command
pub fn run(source: &str, output: &str, verify: bool) -> Result<()> {
    let source_path = Path::new(source);
    let output_path = Path::new(output);

    println!("Parsing Rust source: {}", source_path.display());
    let (structs, constants) = parse_rust_file(source_path)?;

    if structs.is_empty() {
        anyhow::bail!("No #[repr(C)] structs found in {}", source_path.display());
    }

    println!(
        "Found {} struct(s) and {} constant(s)",
        structs.len(),
        constants.len()
    );

    let source_name = source_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let header = generate_header(&structs, &constants, &source_name);

    fs::write(output_path, &header)
        .with_context(|| format!("Failed to write header file: {}", output_path.display()))?;

    println!("Generated header: {}", output_path.display());

    if verify {
        verify_header(output_path)?;
    }

    Ok(())
}

/// Verify the generated header compiles with a C compiler.
///
/// Uses the `cc` crate to find an available C compiler (gcc, clang, MSVC)
/// in a cross-platform way, then invokes it with syntax-check-only flags.
fn verify_header(header_path: &Path) -> Result<()> {
    use std::io::Write;
    use std::process::Command;

    println!("Verifying header with C compiler...");

    // Set minimal environment variables the cc crate expects
    // SAFETY: This is a single-threaded CLI tool, so modifying environment
    // variables is safe.
    setup_cc_env();

    // Use cc crate to find the C compiler
    let compiler = cc::Build::new()
        .cargo_metadata(false)
        .opt_level(0)
        .try_get_compiler()
        .with_context(|| "Failed to find C compiler. Install gcc, clang, or MSVC.")?;

    let cc_path = compiler.path();
    println!("Using compiler: {}", cc_path.display());

    // Create a temporary directory for the verification
    let temp_dir = std::env::temp_dir().join("rustbridge-header-verify");
    fs::create_dir_all(&temp_dir)
        .with_context(|| format!("Failed to create temp dir: {}", temp_dir.display()))?;

    // Create a minimal C file that includes the header
    let test_c_path = temp_dir.join("verify_header.c");
    let header_abs = header_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve header path: {}", header_path.display()))?;

    let test_c_content = format!(
        r#"// Auto-generated verification file
#include "{}"

// Ensure structs are usable
int main(void) {{
    return 0;
}}
"#,
        header_abs.display()
    );

    let mut file = fs::File::create(&test_c_path)
        .with_context(|| format!("Failed to create test file: {}", test_c_path.display()))?;
    file.write_all(test_c_content.as_bytes())?;

    // Build compiler command with appropriate syntax-check flags
    let mut cmd = Command::new(cc_path);

    // Add compiler-specific flags for syntax checking only
    if compiler.is_like_msvc() {
        // MSVC: /Zs for syntax check only
        cmd.arg("/Zs");
    } else {
        // GCC/Clang: -fsyntax-only
        cmd.arg("-fsyntax-only");
    }

    cmd.arg(&test_c_path);

    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute compiler: {}", cc_path.display()))?;

    // Clean up temp files (best effort)
    let _ = fs::remove_file(&test_c_path);
    let _ = fs::remove_dir_all(&temp_dir);

    if output.status.success() {
        println!("Header verification passed");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Header verification failed:\n{}", stderr);
    }
}

/// Set up minimal environment variables required by the cc crate.
///
/// The cc crate expects certain Cargo environment variables to be present.
/// This function sets them to reasonable defaults for the current platform.
fn setup_cc_env() {
    // SAFETY: This is a single-threaded CLI tool, so modifying environment
    // variables is safe. We only set them if they're not already present.
    unsafe {
        let target = get_current_target();

        if std::env::var("TARGET").is_err() {
            std::env::set_var("TARGET", &target);
        }
        if std::env::var("HOST").is_err() {
            std::env::set_var("HOST", &target);
        }
        if std::env::var("OPT_LEVEL").is_err() {
            std::env::set_var("OPT_LEVEL", "0");
        }
        if std::env::var("DEBUG").is_err() {
            std::env::set_var("DEBUG", "false");
        }
    }
}

/// Get the current target triple based on the platform.
fn get_current_target() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (arch, os) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_string(),
        ("x86_64", "macos") => "x86_64-apple-darwin".to_string(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_string(),
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu".to_string(),
        ("aarch64", "macos") => "aarch64-apple-darwin".to_string(),
        ("aarch64", "windows") => "aarch64-pc-windows-msvc".to_string(),
        _ => format!("{arch}-unknown-{os}"),
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn rust_type_to_c___primitive_types___maps_correctly() {
        let ty: Type = syn::parse_quote!(u32);
        assert_eq!(rust_type_to_c(&ty), Some("uint32_t".to_string()));

        let ty: Type = syn::parse_quote!(i64);
        assert_eq!(rust_type_to_c(&ty), Some("int64_t".to_string()));

        let ty: Type = syn::parse_quote!(f32);
        assert_eq!(rust_type_to_c(&ty), Some("float".to_string()));
    }

    #[test]
    fn rust_type_to_c___array_types___maps_correctly() {
        let ty: Type = syn::parse_quote!([u8; 64]);
        assert_eq!(rust_type_to_c(&ty), Some("uint8_t[64]".to_string()));

        let ty: Type = syn::parse_quote!([i32; 10]);
        assert_eq!(rust_type_to_c(&ty), Some("int32_t[10]".to_string()));
    }

    #[test]
    fn rust_type_to_c___pointer_types___maps_correctly() {
        let ty: Type = syn::parse_quote!(*const u8);
        assert_eq!(rust_type_to_c(&ty), Some("const uint8_t*".to_string()));

        let ty: Type = syn::parse_quote!(*mut u8);
        assert_eq!(rust_type_to_c(&ty), Some("uint8_t*".to_string()));
    }

    #[test]
    fn generate_header___structs___produces_valid_c() {
        let structs = vec![CStruct {
            name: "TestStruct".to_string(),
            fields: vec![
                CField {
                    name: "value".to_string(),
                    c_type: "uint32_t".to_string(),
                    doc_comment: Some("The value".to_string()),
                },
                CField {
                    name: "data".to_string(),
                    c_type: "uint8_t[64]".to_string(),
                    doc_comment: None,
                },
            ],
            doc_comment: Some("A test struct".to_string()),
        }];

        let constants = vec![CConstant {
            name: "MSG_TEST".to_string(),
            c_type: "uint32_t".to_string(),
            value: "42".to_string(),
            doc_comment: Some("Test message ID".to_string()),
        }];

        let header = generate_header(&structs, &constants, "test.rs");

        assert!(header.contains("#ifndef TEST_RS_H"));
        assert!(header.contains("#define TEST_RS_H"));
        assert!(header.contains("typedef struct TestStruct"));
        assert!(header.contains("uint32_t value;"));
        assert!(header.contains("uint8_t data[64];"));
        assert!(header.contains("#define MSG_TEST ((uint32_t)42)"));
        assert!(header.contains("/** The value */"));
        assert!(header.contains(" * A test struct"));
    }
}
