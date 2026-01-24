//! Java class generation from message types.
//!
//! This module generates Java POJOs (Plain Old Java Objects) from Rust message types.
//!
//! # Use Cases
//!
//! - Java/Kotlin client libraries for rustbridge plugins
//! - Android applications consuming plugin APIs
//! - JVM-based services integrating with Rust plugins
//!
//! # Generated Code
//!
//! For each message type, generates a Java class with:
//! - Package declaration
//! - Imports (Gson annotations, List if needed)
//! - Class-level JavaDoc
//! - Public fields with JavaDoc
//! - `@SerializedName` annotations for JSON compatibility
//! - Default constructor
//! - Full constructor with all fields
//! - Getters and setters for all fields
//!
//! # Naming Conventions
//!
//! - **Snake case → Camel case**: `user_name` becomes `userName`
//! - **Serde rename**: `#[serde(rename = "user")]` becomes `@SerializedName("user")`
//! - **Getters/Setters**: `getName()` / `setName(String name)`
//!
//! # Type Mappings
//!
//! | Rust | Java (required) | Java (optional) |
//! |------|----------------|-----------------|
//! | `String` | `String` | `String` |
//! | `bool` | `boolean` | `Boolean` |
//! | `i32` | `int` | `Integer` |
//! | `i64` | `long` | `Long` |
//! | `f64` | `double` | `Double` |
//! | `Vec<T>` | `List<T>` | `List<T>` |
//! | Custom | `CustomType` | `CustomType` |
//!
//! Note: Optional primitives use boxed types (`Integer`, `Boolean`, etc.)
//! to allow `null` values.
//!
//! # Examples
//!
//! Input Rust:
//! ```rust
//! use serde::{Serialize, Deserialize};
//!
//! /// User profile.
//! #[derive(Serialize, Deserialize)]
//! pub struct UserProfile {
//!     /// User ID.
//!     pub id: u64,
//!
//!     /// Display name.
//!     pub display_name: String,
//!
//!     /// Email (optional).
//!     pub email: Option<String>,
//! }
//! ```
//!
//! Output Java:
//! ```java
//! package com.example;
//!
//! import com.google.gson.annotations.SerializedName;
//!
//! /**
//!  * User profile.
//!  */
//! public class UserProfile {
//!     /** User ID. */
//!     public long id;
//!
//!     /** Display name. */
//!     @SerializedName("display_name")
//!     public String displayName;
//!
//!     /** Email (optional). */
//!     public String email;
//!
//!     public UserProfile() {}
//!
//!     public UserProfile(long id, String displayName, String email) {
//!         this.id = id;
//!         this.displayName = displayName;
//!         this.email = email;
//!     }
//!
//!     public long getId() { return id; }
//!     public void setId(long id) { this.id = id; }
//!     // ... (other getters/setters)
//! }
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use rustbridge_cli::codegen::{MessageType, generate_java};
//! use std::path::Path;
//!
//! let messages = MessageType::parse_file(Path::new("src/messages.rs")).unwrap();
//! generate_java(&messages, Path::new("src/main/java"), "com.example").unwrap();
//! ```

use super::ir::{FieldType, MessageType};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Generate Java classes from message types.
///
/// Creates one .java file per message type in the specified output directory.
pub fn generate_java(messages: &[MessageType], output_dir: &Path, package: &str) -> Result<()> {
    // Create output directory
    let package_dir = output_dir.join(package.replace('.', "/"));
    fs::create_dir_all(&package_dir)
        .with_context(|| format!("Failed to create directory: {package_dir:?}"))?;

    // Generate each message class
    for message in messages {
        let java_code = generate_java_class(message, package)?;
        let output_file = package_dir.join(format!("{}.java", message.name));

        fs::write(&output_file, java_code)
            .with_context(|| format!("Failed to write {output_file:?}"))?;
    }

    Ok(())
}

/// Generate a Java class for a message type.
fn generate_java_class(message: &MessageType, package: &str) -> Result<String> {
    let mut code = String::new();

    // Package declaration
    code.push_str(&format!("package {};\n\n", package));

    // Imports
    code.push_str("import com.google.gson.annotations.SerializedName;\n");

    let has_list = message
        .fields
        .iter()
        .any(|f| matches!(f.ty, FieldType::Vec(_)));
    if has_list {
        code.push_str("import java.util.List;\n");
    }
    code.push('\n');

    // Class documentation
    if !message.docs.is_empty() {
        code.push_str("/**\n");
        for doc in &message.docs {
            code.push_str(&format!(" * {}\n", doc));
        }
        code.push_str(" */\n");
    }

    // Class declaration
    code.push_str(&format!("public class {} {{\n", message.name));

    // Fields
    for field in &message.fields {
        code.push('\n');

        // Field documentation
        if !field.docs.is_empty() {
            code.push_str("    /**\n");
            for doc in &field.docs {
                code.push_str(&format!("     * {}\n", doc));
            }
            code.push_str("     */\n");
        }

        // Convert snake_case to camelCase for Java
        let java_field_name = to_camel_case(&field.name);

        // Add SerializedName if field name differs from original or has serde rename
        let needs_serialized_name = field.serde_rename.is_some() || java_field_name != field.name;
        if needs_serialized_name {
            let serialize_name = field.serde_rename.as_ref().unwrap_or(&field.name);
            code.push_str(&format!("    @SerializedName(\"{}\")\n", serialize_name));
        }

        // Field declaration
        let java_type = map_type_to_java(&field.ty, field.optional);
        code.push_str(&format!("    public {} {};\n", java_type, java_field_name));
    }

    // Default constructor
    code.push_str("\n    public ");
    code.push_str(&message.name);
    code.push_str("() {}\n");

    // Full constructor
    if !message.fields.is_empty() {
        code.push_str("\n    public ");
        code.push_str(&message.name);
        code.push('(');

        // Constructor parameters
        for (i, field) in message.fields.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            let java_type = map_type_to_java(&field.ty, field.optional);
            let java_field_name = to_camel_case(&field.name);
            code.push_str(&format!("{} {}", java_type, java_field_name));
        }
        code.push_str(") {\n");

        // Field assignments
        for field in &message.fields {
            let java_field_name = to_camel_case(&field.name);
            code.push_str(&format!(
                "        this.{} = {};\n",
                java_field_name, java_field_name
            ));
        }

        code.push_str("    }\n");
    }

    // Getters and setters
    for field in &message.fields {
        let java_type = map_type_to_java(&field.ty, field.optional);
        let java_field_name = to_camel_case(&field.name);
        let method_name_part = capitalize(&java_field_name);

        // Getter
        code.push_str(&format!(
            "\n    public {} get{}() {{\n",
            java_type, method_name_part
        ));
        code.push_str(&format!("        return {};\n", java_field_name));
        code.push_str("    }\n");

        // Setter
        code.push_str(&format!(
            "\n    public void set{}({} {}) {{\n",
            method_name_part, java_type, java_field_name
        ));
        code.push_str(&format!(
            "        this.{} = {};\n",
            java_field_name, java_field_name
        ));
        code.push_str("    }\n");
    }

    code.push_str("}\n");

    Ok(code)
}

/// Map a Rust type to a Java type.
fn map_type_to_java(ty: &FieldType, optional: bool) -> String {
    match ty {
        FieldType::String => "String".to_string(),
        FieldType::Bool => if optional { "Boolean" } else { "boolean" }.to_string(),
        FieldType::I8 | FieldType::I16 | FieldType::I32 => {
            if optional { "Integer" } else { "int" }.to_string()
        }
        FieldType::I64 => if optional { "Long" } else { "long" }.to_string(),
        FieldType::U8 | FieldType::U16 | FieldType::U32 => {
            if optional { "Integer" } else { "int" }.to_string()
        }
        FieldType::U64 => if optional { "Long" } else { "long" }.to_string(),
        FieldType::F32 => if optional { "Float" } else { "float" }.to_string(),
        FieldType::F64 => if optional { "Double" } else { "double" }.to_string(),
        FieldType::Vec(inner) => {
            let inner_type = map_type_to_java(inner, false);
            // Use boxed types for List elements
            let boxed_inner = box_primitive(&inner_type);
            format!("List<{}>", boxed_inner)
        }
        FieldType::Option(inner) => {
            // This shouldn't happen as optional is handled at the field level
            map_type_to_java(inner, true)
        }
        FieldType::Custom(name) => name.clone(),
    }
}

/// Box a primitive type for use in generics.
fn box_primitive(ty: &str) -> String {
    match ty {
        "boolean" => "Boolean".to_string(),
        "byte" => "Byte".to_string(),
        "short" => "Short".to_string(),
        "int" => "Integer".to_string(),
        "long" => "Long".to_string(),
        "float" => "Float".to_string(),
        "double" => "Double".to_string(),
        "char" => "Character".to_string(),
        other => other.to_string(),
    }
}

/// Convert snake_case to camelCase.
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push_str(&c.to_uppercase().to_string());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::codegen::ir::Field;

    #[test]
    fn map_type_to_java___handles_primitives() {
        assert_eq!(map_type_to_java(&FieldType::String, false), "String");
        assert_eq!(map_type_to_java(&FieldType::Bool, false), "boolean");
        assert_eq!(map_type_to_java(&FieldType::Bool, true), "Boolean");
        assert_eq!(map_type_to_java(&FieldType::I32, false), "int");
        assert_eq!(map_type_to_java(&FieldType::I32, true), "Integer");
    }

    #[test]
    fn map_type_to_java___handles_arrays() {
        let vec_string = FieldType::Vec(Box::new(FieldType::String));
        assert_eq!(map_type_to_java(&vec_string, false), "List<String>");

        let vec_int = FieldType::Vec(Box::new(FieldType::I32));
        assert_eq!(map_type_to_java(&vec_int, false), "List<Integer>");
    }

    #[test]
    fn to_camel_case___converts_snake_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("display_name"), "displayName");
        assert_eq!(to_camel_case("foo_bar_baz"), "fooBarBaz");
        assert_eq!(to_camel_case("simple"), "simple");
        assert_eq!(to_camel_case(""), "");
    }

    #[test]
    fn capitalize___capitalizes_first_letter() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("world"), "World");
        assert_eq!(capitalize("a"), "A");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn generate_java_class___creates_valid_class() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec!["A test message".to_string()],
            fields: vec![Field {
                name: "name".to_string(),
                ty: FieldType::String,
                docs: vec!["The name field".to_string()],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        assert!(code.contains("package com.example;"));
        assert!(code.contains("public class TestMessage"));
        assert!(code.contains("public String name;"));
        assert!(code.contains("public String getName()"));
        assert!(code.contains("public void setName(String name)"));
    }

    #[test]
    fn generate_java_class___converts_snake_case_to_camel_case() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "display_name".to_string(),
                ty: FieldType::String,
                docs: vec![],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        assert!(code.contains("@SerializedName(\"display_name\")"));
        assert!(code.contains("public String displayName;"));
        assert!(code.contains("public String getDisplayName()"));
        assert!(code.contains("public void setDisplayName(String displayName)"));
    }

    #[test]
    fn generate_java_class___handles_optional_primitives() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "count".to_string(),
                ty: FieldType::I32,
                docs: vec![],
                optional: true,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        // Optional primitives should use boxed types
        assert!(code.contains("public Integer count;"));
        assert!(code.contains("public Integer getCount()"));
        assert!(code.contains("public void setCount(Integer count)"));
    }

    #[test]
    fn generate_java_class___handles_required_primitives() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "count".to_string(),
                ty: FieldType::I32,
                docs: vec![],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        // Required primitives should use primitive types
        assert!(code.contains("public int count;"));
        assert!(code.contains("public int getCount()"));
        assert!(code.contains("public void setCount(int count)"));
    }

    #[test]
    fn generate_java_class___includes_list_import_when_needed() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "items".to_string(),
                ty: FieldType::Vec(Box::new(FieldType::String)),
                docs: vec![],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        assert!(code.contains("import java.util.List;"));
        assert!(code.contains("public List<String> items;"));
    }

    #[test]
    fn generate_java_class___no_list_import_when_not_needed() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "name".to_string(),
                ty: FieldType::String,
                docs: vec![],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        assert!(!code.contains("import java.util.List;"));
    }

    #[test]
    fn generate_java_class___handles_vec_of_primitives() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "numbers".to_string(),
                ty: FieldType::Vec(Box::new(FieldType::I32)),
                docs: vec![],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        // Vec of primitives should use boxed types in List
        assert!(code.contains("public List<Integer> numbers;"));
    }

    #[test]
    fn generate_java_class___handles_custom_types() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "address".to_string(),
                ty: FieldType::Custom("Address".to_string()),
                docs: vec![],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        assert!(code.contains("public Address address;"));
        assert!(code.contains("public Address getAddress()"));
    }

    #[test]
    fn generate_java_class___handles_serde_rename() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![Field {
                name: "rust_name".to_string(),
                ty: FieldType::String,
                docs: vec![],
                optional: false,
                serde_rename: Some("jsonName".to_string()),
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        assert!(code.contains("@SerializedName(\"jsonName\")"));
        assert!(code.contains("public String rustName;"));
    }

    #[test]
    fn generate_java_class___handles_multiline_docs() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec!["Line 1".to_string(), "Line 2".to_string()],
            fields: vec![Field {
                name: "field".to_string(),
                ty: FieldType::String,
                docs: vec!["Field doc 1".to_string(), "Field doc 2".to_string()],
                optional: false,
                serde_rename: None,
            }],
        };

        let code = generate_java_class(&message, "com.example").unwrap();

        assert!(code.contains("/**\n * Line 1\n * Line 2\n */"));
        assert!(code.contains("/**\n     * Field doc 1\n     * Field doc 2\n     */"));
    }

    #[test]
    fn box_primitive___boxes_all_primitive_types() {
        assert_eq!(box_primitive("boolean"), "Boolean");
        assert_eq!(box_primitive("byte"), "Byte");
        assert_eq!(box_primitive("short"), "Short");
        assert_eq!(box_primitive("int"), "Integer");
        assert_eq!(box_primitive("long"), "Long");
        assert_eq!(box_primitive("float"), "Float");
        assert_eq!(box_primitive("double"), "Double");
        assert_eq!(box_primitive("char"), "Character");
    }

    #[test]
    fn box_primitive___leaves_reference_types_unchanged() {
        assert_eq!(box_primitive("String"), "String");
        assert_eq!(box_primitive("CustomType"), "CustomType");
    }

    #[test]
    fn map_type_to_java___handles_all_float_types() {
        assert_eq!(map_type_to_java(&FieldType::F32, false), "float");
        assert_eq!(map_type_to_java(&FieldType::F32, true), "Float");
        assert_eq!(map_type_to_java(&FieldType::F64, false), "double");
        assert_eq!(map_type_to_java(&FieldType::F64, true), "Double");
    }
}
