//! JSON Schema generation from message types.

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
}
