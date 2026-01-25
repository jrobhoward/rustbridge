//! JVM type mappings for Java and Kotlin code generation.
//!
//! This module provides shared type mapping logic for JVM languages.
//! Both Java and Kotlin use similar type systems with some differences
//! in syntax and nullability handling.
//!
//! # Type Mappings
//!
//! | Rust | Java (required) | Java (optional) | Kotlin |
//! |------|-----------------|-----------------|--------|
//! | `String` | `String` | `String` | `String` / `String?` |
//! | `bool` | `boolean` | `Boolean` | `Boolean` |
//! | `i32` | `int` | `Integer` | `Int` / `Int?` |
//! | `i64` | `long` | `Long` | `Long` / `Long?` |
//! | `f64` | `double` | `Double` | `Double` / `Double?` |
//! | `Vec<T>` | `List<T>` | `List<T>` | `List<T>` |

use super::ir::FieldType;

/// Represents a JVM type with both primitive and boxed forms.
#[derive(Debug, Clone, PartialEq)]
pub struct JvmType {
    /// The primitive type name (e.g., "int", "boolean") or reference type name.
    pub primitive: String,
    /// The boxed/nullable type name (e.g., "Integer", "Boolean").
    pub boxed: String,
    /// Whether this type is a primitive (affects Kotlin nullability).
    pub is_primitive: bool,
}

impl JvmType {
    /// Create a new JVM type with the same primitive and boxed form.
    pub fn reference(name: &str) -> Self {
        Self {
            primitive: name.to_string(),
            boxed: name.to_string(),
            is_primitive: false,
        }
    }

    /// Create a new JVM primitive type.
    pub fn primitive(primitive: &str, boxed: &str) -> Self {
        Self {
            primitive: primitive.to_string(),
            boxed: boxed.to_string(),
            is_primitive: true,
        }
    }

    /// Get the appropriate Java type string.
    ///
    /// For optional fields, returns the boxed type to allow null.
    /// For required fields, returns the primitive type for efficiency.
    pub fn java_type(&self, optional: bool) -> &str {
        if optional && self.is_primitive {
            &self.boxed
        } else {
            &self.primitive
        }
    }

    /// Get the appropriate Kotlin type string.
    ///
    /// Kotlin always uses the same type name, but adds `?` for optional.
    #[allow(dead_code)] // Prepared for future Kotlin generator
    pub fn kotlin_type(&self, optional: bool) -> String {
        if optional {
            format!("{}?", self.boxed)
        } else {
            self.boxed.clone()
        }
    }
}

/// Map a Rust field type to a JVM type.
pub fn map_field_type(ty: &FieldType) -> JvmType {
    match ty {
        FieldType::String => JvmType::reference("String"),
        FieldType::Bool => JvmType::primitive("boolean", "Boolean"),
        FieldType::I8 | FieldType::I16 | FieldType::I32 => JvmType::primitive("int", "Integer"),
        FieldType::I64 => JvmType::primitive("long", "Long"),
        FieldType::U8 | FieldType::U16 | FieldType::U32 => JvmType::primitive("int", "Integer"),
        FieldType::U64 => JvmType::primitive("long", "Long"),
        FieldType::F32 => JvmType::primitive("float", "Float"),
        FieldType::F64 => JvmType::primitive("double", "Double"),
        FieldType::Vec(inner) => {
            let inner_type = map_field_type(inner);
            // List elements must always be boxed
            JvmType::reference(&format!("List<{}>", inner_type.boxed))
        }
        FieldType::Option(inner) => {
            // Option is handled at the field level
            map_field_type(inner)
        }
        FieldType::Custom(name) => JvmType::reference(name),
    }
}

/// Check if a field type requires a List import.
pub fn needs_list_import(fields: &[super::ir::Field]) -> bool {
    fields.iter().any(|f| matches!(f.ty, FieldType::Vec(_)))
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn jvm_type___reference___same_primitive_and_boxed() {
        let ty = JvmType::reference("String");

        assert_eq!(ty.primitive, "String");
        assert_eq!(ty.boxed, "String");
        assert!(!ty.is_primitive);
    }

    #[test]
    fn jvm_type___primitive___different_primitive_and_boxed() {
        let ty = JvmType::primitive("int", "Integer");

        assert_eq!(ty.primitive, "int");
        assert_eq!(ty.boxed, "Integer");
        assert!(ty.is_primitive);
    }

    #[test]
    fn jvm_type___java_type___required_uses_primitive() {
        let ty = JvmType::primitive("int", "Integer");

        assert_eq!(ty.java_type(false), "int");
    }

    #[test]
    fn jvm_type___java_type___optional_uses_boxed() {
        let ty = JvmType::primitive("int", "Integer");

        assert_eq!(ty.java_type(true), "Integer");
    }

    #[test]
    fn jvm_type___java_type___reference_same_for_both() {
        let ty = JvmType::reference("String");

        assert_eq!(ty.java_type(false), "String");
        assert_eq!(ty.java_type(true), "String");
    }

    #[test]
    fn jvm_type___kotlin_type___required_no_question_mark() {
        let ty = JvmType::primitive("int", "Int");

        assert_eq!(ty.kotlin_type(false), "Int");
    }

    #[test]
    fn jvm_type___kotlin_type___optional_has_question_mark() {
        let ty = JvmType::primitive("int", "Int");

        assert_eq!(ty.kotlin_type(true), "Int?");
    }

    #[test]
    fn map_field_type___handles_primitives() {
        assert_eq!(
            map_field_type(&FieldType::String).java_type(false),
            "String"
        );
        assert_eq!(map_field_type(&FieldType::Bool).java_type(false), "boolean");
        assert_eq!(map_field_type(&FieldType::I32).java_type(false), "int");
        assert_eq!(map_field_type(&FieldType::I64).java_type(false), "long");
        assert_eq!(map_field_type(&FieldType::F64).java_type(false), "double");
    }

    #[test]
    fn map_field_type___handles_optional_primitives() {
        assert_eq!(map_field_type(&FieldType::Bool).java_type(true), "Boolean");
        assert_eq!(map_field_type(&FieldType::I32).java_type(true), "Integer");
        assert_eq!(map_field_type(&FieldType::I64).java_type(true), "Long");
    }

    #[test]
    fn map_field_type___handles_vec() {
        let vec_string = FieldType::Vec(Box::new(FieldType::String));

        assert_eq!(map_field_type(&vec_string).java_type(false), "List<String>");
    }

    #[test]
    fn map_field_type___handles_vec_of_primitives() {
        let vec_int = FieldType::Vec(Box::new(FieldType::I32));

        // List elements must be boxed
        assert_eq!(map_field_type(&vec_int).java_type(false), "List<Integer>");
    }

    #[test]
    fn map_field_type___handles_custom_types() {
        let custom = FieldType::Custom("Address".to_string());

        assert_eq!(map_field_type(&custom).java_type(false), "Address");
    }

    #[test]
    fn map_field_type___handles_nested_vec() {
        let nested = FieldType::Vec(Box::new(FieldType::Vec(Box::new(FieldType::String))));

        assert_eq!(
            map_field_type(&nested).java_type(false),
            "List<List<String>>"
        );
    }
}
