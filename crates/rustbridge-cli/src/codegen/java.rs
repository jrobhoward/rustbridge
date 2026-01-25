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
//! - Imports (Jackson annotations, List if needed)
//! - Class-level JavaDoc
//! - Public fields with JavaDoc
//! - `@JsonProperty` annotations for JSON compatibility
//! - Default constructor
//! - Full constructor with all fields
//! - Getters and setters for all fields
//!
//! # Naming Conventions
//!
//! - **Snake case → Camel case**: `user_name` becomes `userName`
//! - **Serde rename**: `#[serde(rename = "user")]` becomes `@JsonProperty("user")`
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
//! import com.fasterxml.jackson.annotation.JsonProperty;
//!
//! /**
//!  * User profile.
//!  */
//! public class UserProfile {
//!     /** User ID. */
//!     public long id;
//!
//!     /** Display name. */
//!     @JsonProperty("display_name")
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

use super::ir::MessageType;
use super::jvm_types::{map_field_type, needs_list_import};
use super::naming::{capitalize, to_camel_case};
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
    code.push_str("import com.fasterxml.jackson.annotation.JsonProperty;\n");

    if needs_list_import(&message.fields) {
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

        // Add JsonProperty if field name differs from original or has serde rename
        let needs_json_property = field.serde_rename.is_some() || java_field_name != field.name;
        if needs_json_property {
            let serialize_name = field.serde_rename.as_ref().unwrap_or(&field.name);
            code.push_str(&format!("    @JsonProperty(\"{}\")\n", serialize_name));
        }

        // Field declaration
        let jvm_type = map_field_type(&field.ty);
        let java_type = jvm_type.java_type(field.optional);
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
            let jvm_type = map_field_type(&field.ty);
            let java_type = jvm_type.java_type(field.optional);
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
        let jvm_type = map_field_type(&field.ty);
        let java_type = jvm_type.java_type(field.optional);
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

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::codegen::ir::{Field, FieldType};

    // Note: Tests for naming utilities (to_camel_case, capitalize) and type mapping
    // (map_field_type) are in their respective shared modules: naming.rs and jvm_types.rs

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

        assert!(code.contains("@JsonProperty(\"display_name\")"));
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

        assert!(code.contains("@JsonProperty(\"jsonName\")"));
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
}
