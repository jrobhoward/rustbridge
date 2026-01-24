//! JSON Schema generation from message types.
//!
//! This module generates [JSON Schema Draft-07](https://json-schema.org/draft-07/schema)
//! from Rust message types.
//!
//! # Use Cases
//!
//! - API documentation and contract definition
//! - Validation schemas for JSON payloads
//! - OpenAPI/Swagger integration
//! - Self-documenting plugin bundles
//!
//! # Output Format
//!
//! Generates a schema with `definitions` object containing all message types:
//!
//! ```json
//! {
//!   "$schema": "http://json-schema.org/draft-07/schema#",
//!   "definitions": {
//!     "MessageType1": { "type": "object", "properties": {...} },
//!     "MessageType2": { "type": "object", "properties": {...} }
//!   }
//! }
//! ```
//!
//! # Type Mappings
//!
//! - `String` → `{"type": "string"}`
//! - `bool` → `{"type": "boolean"}`
//! - `i8..i64` → `{"type": "integer"}`
//! - `u8..u64` → `{"type": "integer", "minimum": 0}`
//! - `f32`, `f64` → `{"type": "number"}`
//! - `Vec<T>` → `{"type": "array", "items": {...}}`
//! - Custom types → `{"$ref": "#/definitions/TypeName"}`
//!
//! # Required vs Optional
//!
//! - Required fields appear in the `required` array
//! - `Option<T>` fields are omitted from `required`
//!
//! # Documentation
//!
//! Doc comments (`///`) are preserved as `description` fields in the schema.
//!
//! # Examples
//!
//! ```rust,no_run
//! use rustbridge_cli::codegen::{MessageType, generate_json_schema};
//! use std::path::Path;
//!
//! let messages = MessageType::parse_file(Path::new("src/messages.rs")).unwrap();
//! let schema = generate_json_schema(&messages).unwrap();
//! let json = serde_json::to_string_pretty(&schema).unwrap();
//! println!("{}", json);
//! ```

use super::ir::{FieldType, MessageType};
use anyhow::Result;
use serde_json::{Value, json};

/// Generate JSON Schema from message types.
pub fn generate_json_schema(messages: &[MessageType]) -> Result<Value> {
    let mut definitions = serde_json::Map::new();

    for message in messages {
        let schema = generate_message_schema(message);
        definitions.insert(message.name.clone(), schema);
    }

    Ok(json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "definitions": definitions,
    }))
}

/// Generate JSON Schema for a single message type.
fn generate_message_schema(message: &MessageType) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for field in &message.fields {
        let field_name = field.serde_rename.as_ref().unwrap_or(&field.name).clone();

        let field_schema = generate_field_schema(&field.ty);
        properties.insert(field_name.clone(), field_schema);

        if !field.optional {
            required.push(field_name);
        }
    }

    let mut schema = json!({
        "type": "object",
        "properties": properties,
    });

    if !required.is_empty() {
        schema["required"] = json!(required);
    }

    if !message.docs.is_empty() {
        schema["description"] = json!(message.docs.join("\n"));
    }

    schema
}

/// Generate JSON Schema for a field type.
fn generate_field_schema(ty: &FieldType) -> Value {
    match ty {
        FieldType::String => json!({"type": "string"}),
        FieldType::Bool => json!({"type": "boolean"}),
        FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::I64 => {
            json!({"type": "integer"})
        }
        FieldType::U8 | FieldType::U16 | FieldType::U32 | FieldType::U64 => {
            json!({"type": "integer", "minimum": 0})
        }
        FieldType::F32 | FieldType::F64 => json!({"type": "number"}),
        FieldType::Vec(inner) => json!({
            "type": "array",
            "items": generate_field_schema(inner),
        }),
        FieldType::Option(inner) => {
            // Option is handled at the field level (not in required array)
            generate_field_schema(inner)
        }
        FieldType::Custom(name) => json!({
            "$ref": format!("#/definitions/{}", name)
        }),
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn generate_message_schema___creates_object_schema() {
        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec!["A test message".to_string()],
            fields: vec![],
        };

        let schema = generate_message_schema(&message);

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["description"], "A test message");
    }

    #[test]
    fn generate_field_schema___handles_primitives() {
        assert_eq!(
            generate_field_schema(&FieldType::String),
            json!({"type": "string"})
        );
        assert_eq!(
            generate_field_schema(&FieldType::Bool),
            json!({"type": "boolean"})
        );
        assert_eq!(
            generate_field_schema(&FieldType::I32),
            json!({"type": "integer"})
        );
    }

    #[test]
    fn generate_field_schema___handles_arrays() {
        let vec_string = FieldType::Vec(Box::new(FieldType::String));
        let schema = generate_field_schema(&vec_string);

        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
    }

    #[test]
    fn generate_field_schema___handles_nested_arrays() {
        let vec_vec_int = FieldType::Vec(Box::new(FieldType::Vec(Box::new(FieldType::I32))));
        let schema = generate_field_schema(&vec_vec_int);

        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "array");
        assert_eq!(schema["items"]["items"]["type"], "integer");
    }

    #[test]
    fn generate_field_schema___handles_custom_types() {
        let custom = FieldType::Custom("CustomType".to_string());
        let schema = generate_field_schema(&custom);

        assert_eq!(schema["$ref"], "#/definitions/CustomType");
    }

    #[test]
    fn generate_field_schema___handles_vec_of_custom() {
        let vec_custom = FieldType::Vec(Box::new(FieldType::Custom("Item".to_string())));
        let schema = generate_field_schema(&vec_custom);

        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["$ref"], "#/definitions/Item");
    }

    #[test]
    fn generate_message_schema___handles_required_fields() {
        use crate::codegen::ir::Field;

        let message = MessageType {
            name: "TestMessage".to_string(),
            docs: vec![],
            fields: vec![
                Field {
                    name: "required".to_string(),
                    ty: FieldType::String,
                    docs: vec![],
                    optional: false,
                    serde_rename: None,
                },
                Field {
                    name: "optional".to_string(),
                    ty: FieldType::String,
                    docs: vec![],
                    optional: true,
                    serde_rename: None,
                },
            ],
        };

        let schema = generate_message_schema(&message);

        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "required");
    }

    #[test]
    fn generate_message_schema___handles_serde_rename() {
        use crate::codegen::ir::Field;

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

        let schema = generate_message_schema(&message);

        assert!(schema["properties"]["jsonName"].is_object());
        assert!(schema["properties"]["rust_name"].is_null());
    }

    #[test]
    fn generate_json_schema___handles_multiple_messages() {
        let messages = vec![
            MessageType {
                name: "Message1".to_string(),
                docs: vec![],
                fields: vec![],
            },
            MessageType {
                name: "Message2".to_string(),
                docs: vec![],
                fields: vec![],
            },
        ];

        let schema = generate_json_schema(&messages).unwrap();

        assert_eq!(schema["$schema"], "http://json-schema.org/draft-07/schema#");
        assert!(schema["definitions"]["Message1"].is_object());
        assert!(schema["definitions"]["Message2"].is_object());
    }

    #[test]
    fn generate_field_schema___unsigned_integers_have_minimum() {
        let u32_type = FieldType::U32;
        let schema = generate_field_schema(&u32_type);

        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["minimum"], 0);
    }

    #[test]
    fn generate_field_schema___signed_integers_no_minimum() {
        let i32_type = FieldType::I32;
        let schema = generate_field_schema(&i32_type);

        assert_eq!(schema["type"], "integer");
        assert!(schema["minimum"].is_null());
    }
}
