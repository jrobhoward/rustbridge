//! Intermediate representation for message types.
//!
//! This module defines the IR (Intermediate Representation) used by all code generators.
//! It provides a simplified, language-agnostic view of Rust message types.
//!
//! # Purpose
//!
//! The IR serves as a bridge between Rust's type system and target language code.
//! By parsing once into IR, we can generate multiple target languages without
//! re-parsing the Rust source.
//!
//! # Structure
//!
//! - [`MessageType`]: Represents a single struct with fields and documentation
//! - [`Field`]: Represents a field with its type, docs, and metadata
//! - [`FieldType`]: Simplified type representation (primitives, containers, custom)
//!
//! # Parsing
//!
//! The parser uses [`syn`] to parse Rust source files and extract structs marked with:
//! - `#[derive(Serialize, Deserialize)]`
//! - `#[derive(Message)]`
//!
//! # Examples
//!
//! ```rust,no_run
//! use rustbridge_cli::codegen::ir::MessageType;
//! use std::path::Path;
//!
//! // Parse from file
//! let messages = MessageType::parse_file(Path::new("src/messages.rs")).unwrap();
//!
//! for msg in &messages {
//!     println!("Struct: {}", msg.name);
//!     for field in &msg.fields {
//!         println!("  Field: {} ({:?})", field.name, field.ty);
//!     }
//! }
//! ```
//!
//! # Supported Features
//!
//! - Doc comments (`///`)
//! - `#[serde(rename = "...")]` attributes
//! - `Option<T>` for optional fields
//! - `Vec<T>` for arrays
//! - Nested custom types
//! - All primitive types
//!
//! # Limitations
//!
//! - Enums not supported (use tagged union pattern)
//! - Tuple structs not supported (use named fields)
//! - Generics not supported
//! - Fixed-size arrays not supported (use `Vec<T>`)

use anyhow::{Context, Result};
use std::path::Path;
use syn::{Attribute, Fields, Meta, Type};

/// A message type definition.
#[derive(Debug, Clone)]
pub struct MessageType {
    /// The struct name.
    pub name: String,

    /// Documentation comments.
    pub docs: Vec<String>,

    /// Fields in the struct.
    pub fields: Vec<Field>,
}

/// A field in a message type.
#[derive(Debug, Clone)]
pub struct Field {
    /// Field name.
    pub name: String,

    /// Field type.
    pub ty: FieldType,

    /// Documentation comments.
    pub docs: Vec<String>,

    /// Whether this field is optional (Option<T>).
    pub optional: bool,

    /// Serde rename attribute, if present.
    pub serde_rename: Option<String>,
}

/// Simplified field type representation.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Option variant is used for recursive parsing
pub enum FieldType {
    /// String type.
    String,

    /// Boolean type.
    Bool,

    /// Integer types.
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,

    /// Floating point types.
    F32,
    F64,

    /// Vector/array type.
    Vec(Box<FieldType>),

    /// Optional type (Option<T>).
    Option(Box<FieldType>),

    /// Custom type (struct/enum).
    Custom(String),
}

impl MessageType {
    /// Parse Rust source file and extract message types.
    ///
    /// Only extracts structs that derive `Message` or `Serialize + Deserialize`.
    pub fn parse_file(path: &Path) -> Result<Vec<MessageType>> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {path:?}"))?;

        Self::parse_source(&content)
    }

    /// Parse Rust source code and extract message types.
    pub fn parse_source(source: &str) -> Result<Vec<MessageType>> {
        let file = syn::parse_file(source).context("Failed to parse Rust source")?;

        let mut messages = Vec::new();

        for item in &file.items {
            if let syn::Item::Struct(s) = item {
                if is_message_struct(&s.attrs) {
                    messages.push(Self::from_struct(s)?);
                }
            }
        }

        Ok(messages)
    }

    /// Convert a syn::ItemStruct to a MessageType.
    fn from_struct(s: &syn::ItemStruct) -> Result<MessageType> {
        let name = s.ident.to_string();
        let docs = extract_docs(&s.attrs);

        let fields = match &s.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(Field::from_syn_field)
                .collect::<Result<Vec<_>>>()?,
            Fields::Unnamed(_) => {
                anyhow::bail!("Tuple structs are not supported for message generation")
            }
            Fields::Unit => Vec::new(),
        };

        Ok(MessageType { name, docs, fields })
    }
}

impl Field {
    /// Convert a syn::Field to a Field.
    fn from_syn_field(f: &syn::Field) -> Result<Field> {
        let name = f
            .ident
            .as_ref()
            .context("Field must have a name")?
            .to_string();

        let docs = extract_docs(&f.attrs);
        let serde_rename = extract_serde_rename(&f.attrs);
        let (ty, optional) = parse_field_type(&f.ty)?;

        Ok(Field {
            name,
            ty,
            docs,
            optional,
            serde_rename,
        })
    }
}

/// Check if a struct has derives that indicate it's a message type.
fn is_message_struct(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if let Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("derive") {
                let tokens = meta_list.tokens.to_string();
                // Check for Message derive or Serialize + Deserialize
                if tokens.contains("Message")
                    || (tokens.contains("Serialize") && tokens.contains("Deserialize"))
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Extract documentation comments from attributes.
fn extract_docs(attrs: &[Attribute]) -> Vec<String> {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let doc = lit_str.value();
                        let doc = doc.trim();
                        if !doc.is_empty() {
                            docs.push(doc.to_string());
                        }
                    }
                }
            }
        }
    }

    docs
}

/// Extract serde(rename = "...") attribute.
fn extract_serde_rename(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("serde") {
            if let Meta::List(meta_list) = &attr.meta {
                // Parse nested meta items
                let nested = meta_list.tokens.to_string();
                if let Some(rename) = nested.strip_prefix("rename = \"") {
                    if let Some(end) = rename.find('"') {
                        return Some(rename[..end].to_string());
                    }
                }
            }
        }
    }
    None
}

/// Parse a syn::Type into a FieldType.
fn parse_field_type(ty: &Type) -> Result<(FieldType, bool)> {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;

            // Simple types
            if path.is_ident("String") {
                return Ok((FieldType::String, false));
            }
            if path.is_ident("bool") {
                return Ok((FieldType::Bool, false));
            }
            if path.is_ident("i8") {
                return Ok((FieldType::I8, false));
            }
            if path.is_ident("i16") {
                return Ok((FieldType::I16, false));
            }
            if path.is_ident("i32") {
                return Ok((FieldType::I32, false));
            }
            if path.is_ident("i64") {
                return Ok((FieldType::I64, false));
            }
            if path.is_ident("u8") {
                return Ok((FieldType::U8, false));
            }
            if path.is_ident("u16") {
                return Ok((FieldType::U16, false));
            }
            if path.is_ident("u32") {
                return Ok((FieldType::U32, false));
            }
            if path.is_ident("u64") {
                return Ok((FieldType::U64, false));
            }
            if path.is_ident("f32") {
                return Ok((FieldType::F32, false));
            }
            if path.is_ident("f64") {
                return Ok((FieldType::F64, false));
            }

            // Generic types (Vec, Option)
            if let Some(segment) = path.segments.last() {
                let ident = &segment.ident;

                if ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let (inner_field_type, _) = parse_field_type(inner_ty)?;
                            return Ok((FieldType::Vec(Box::new(inner_field_type)), false));
                        }
                    }
                }

                if ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let (inner_field_type, _) = parse_field_type(inner_ty)?;
                            return Ok((inner_field_type, true)); // Return inner type with optional=true
                        }
                    }
                }

                // Custom type
                return Ok((FieldType::Custom(ident.to_string()), false));
            }

            Ok((
                FieldType::Custom(
                    path.segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                ),
                false,
            ))
        }
        _ => anyhow::bail!("Unsupported field type: {}", quote::quote!(#ty)),
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn parse_source___extracts_message_struct() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            /// A test message.
            #[derive(Serialize, Deserialize)]
            pub struct TestMessage {
                /// The name field.
                pub name: String,
                pub count: i32,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].name, "TestMessage");
        assert_eq!(messages[0].docs, vec!["A test message."]);
        assert_eq!(messages[0].fields.len(), 2);
        assert_eq!(messages[0].fields[0].name, "name");
        assert_eq!(messages[0].fields[0].ty, FieldType::String);
        assert_eq!(messages[0].fields[0].docs, vec!["The name field."]);
    }

    #[test]
    fn parse_source___handles_optional_fields() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct TestMessage {
                pub required: String,
                pub optional: Option<String>,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert!(!messages[0].fields[0].optional);
        assert!(messages[0].fields[1].optional);
        assert_eq!(messages[0].fields[1].ty, FieldType::String); // Inner type
    }

    #[test]
    fn parse_source___handles_serde_rename() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct TestMessage {
                #[serde(rename = "old_name")]
                pub new_name: String,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(messages[0].fields[0].name, "new_name");
        assert_eq!(
            messages[0].fields[0].serde_rename,
            Some("old_name".to_string())
        );
    }

    #[test]
    fn parse_source___handles_nested_vec() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct TestMessage {
                pub items: Vec<Vec<String>>,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(
            messages[0].fields[0].ty,
            FieldType::Vec(Box::new(FieldType::Vec(Box::new(FieldType::String))))
        );
    }

    #[test]
    fn parse_source___handles_custom_types() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct Address {
                pub street: String,
                pub city: String,
            }

            #[derive(Serialize, Deserialize)]
            pub struct Person {
                pub name: String,
                pub address: Address,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].name, "Person");
        assert_eq!(
            messages[1].fields[1].ty,
            FieldType::Custom("Address".to_string())
        );
    }

    #[test]
    fn parse_source___handles_vec_of_custom_types() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct Item {
                pub id: u64,
            }

            #[derive(Serialize, Deserialize)]
            pub struct Container {
                pub items: Vec<Item>,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(
            messages[1].fields[0].ty,
            FieldType::Vec(Box::new(FieldType::Custom("Item".to_string())))
        );
    }

    #[test]
    fn parse_source___handles_option_of_vec() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct TestMessage {
                pub tags: Option<Vec<String>>,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert!(messages[0].fields[0].optional);
        assert_eq!(
            messages[0].fields[0].ty,
            FieldType::Vec(Box::new(FieldType::String))
        );
    }

    #[test]
    fn parse_source___ignores_non_message_structs() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            // Should be ignored - no derives
            pub struct Ignored {
                pub data: String,
            }

            // Should be included
            #[derive(Serialize, Deserialize)]
            pub struct Included {
                pub data: String,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].name, "Included");
    }

    #[test]
    fn parse_source___handles_all_integer_types() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct AllInts {
                pub i8_val: i8,
                pub i16_val: i16,
                pub i32_val: i32,
                pub i64_val: i64,
                pub u8_val: u8,
                pub u16_val: u16,
                pub u32_val: u32,
                pub u64_val: u64,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(messages[0].fields[0].ty, FieldType::I8);
        assert_eq!(messages[0].fields[1].ty, FieldType::I16);
        assert_eq!(messages[0].fields[2].ty, FieldType::I32);
        assert_eq!(messages[0].fields[3].ty, FieldType::I64);
        assert_eq!(messages[0].fields[4].ty, FieldType::U8);
        assert_eq!(messages[0].fields[5].ty, FieldType::U16);
        assert_eq!(messages[0].fields[6].ty, FieldType::U32);
        assert_eq!(messages[0].fields[7].ty, FieldType::U64);
    }

    #[test]
    fn parse_source___handles_multiline_docs() {
        let source = r#"
            use serde::{Serialize, Deserialize};

            /// First line.
            /// Second line.
            /// Third line.
            #[derive(Serialize, Deserialize)]
            pub struct TestMessage {
                /// Field doc line 1.
                /// Field doc line 2.
                pub field: String,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(messages[0].docs.len(), 3);
        assert_eq!(messages[0].docs[0], "First line.");
        assert_eq!(messages[0].docs[1], "Second line.");
        assert_eq!(messages[0].docs[2], "Third line.");

        assert_eq!(messages[0].fields[0].docs.len(), 2);
        assert_eq!(messages[0].fields[0].docs[0], "Field doc line 1.");
        assert_eq!(messages[0].fields[0].docs[1], "Field doc line 2.");
    }

    #[test]
    fn parse_source___handles_message_derive() {
        let source = r#"
            #[derive(Message)]
            pub struct TestMessage {
                pub data: String,
            }
        "#;

        let messages = MessageType::parse_source(source).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].name, "TestMessage");
    }
}
