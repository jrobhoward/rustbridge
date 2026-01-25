//! Code generation from Rust message types.
//!
//! This module provides functionality to generate language bindings from Rust structs
//! marked with `#[derive(Serialize, Deserialize)]` or `#[derive(Message)]`.
//!
//! # Supported Generators
//!
//! - **JSON Schema**: Generate JSON Schema Draft-07 definitions for API documentation
//! - **Java**: Generate Java POJOs with Gson annotations for JVM languages
//! - **Future**: C#, Kotlin, TypeScript
//!
//! # Architecture
//!
//! The code generation uses a two-stage pipeline:
//!
//! ```text
//! Rust Source
//!     ↓
//!  [Parser]
//!     ↓
//!    IR (MessageType)
//!     ↓
//!  ├─→ [JSON Schema Generator] → schema.json
//!  └─→ [Java Generator] → *.java
//! ```
//!
//! This architecture provides:
//! - **Shared parsing logic** across all generators
//! - **Easy language additions** without re-implementing parsing
//! - **Consistent transformations** applied to all targets
//!
//! # Usage
//!
//! ## Command Line
//!
//! Generate JSON Schema:
//! ```bash
//! rustbridge generate json-schema -i src/messages.rs -o schema.json
//! ```
//!
//! Generate Java classes:
//! ```bash
//! rustbridge generate java -i src/messages.rs -o src/main/java -p com.example
//! ```
//!
//! ## Programmatic
//!
//! ```rust,no_run
//! use rustbridge_cli::codegen::{MessageType, generate_json_schema};
//! use std::path::Path;
//!
//! // Parse Rust source
//! let messages = MessageType::parse_file(Path::new("src/messages.rs")).unwrap();
//!
//! // Generate JSON Schema
//! let schema = generate_json_schema(&messages).unwrap();
//! println!("{}", serde_json::to_string_pretty(&schema).unwrap());
//! ```
//!
//! # Supported Types
//!
//! - **Primitives**: `String`, `bool`, `i8..i64`, `u8..u64`, `f32`, `f64`
//! - **Containers**: `Vec<T>`, `Option<T>`
//! - **Custom types**: Other `#[derive(Serialize, Deserialize)]` structs
//!
//! # Examples
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//!
//! /// A user profile.
//! #[derive(Serialize, Deserialize)]
//! pub struct UserProfile {
//!     /// User ID.
//!     pub id: u64,
//!
//!     /// Display name.
//!     #[serde(rename = "displayName")]
//!     pub display_name: String,
//!
//!     /// Email address (optional).
//!     pub email: Option<String>,
//!
//!     /// User tags.
//!     pub tags: Vec<String>,
//! }
//! ```
//!
//! Generates JSON Schema with proper types, documentation, and required fields.
//! Generates Java with camelCase fields, `@SerializedName` annotations, getters/setters.
//!
//! # See Also
//!
//! - [`ir`] module for the intermediate representation
//! - [`json_schema`] module for JSON Schema generation
//! - [`java`] module for Java class generation
//! - [Code Generation Guide](../../../docs/CODE_GENERATION.md) for comprehensive docs

pub mod ir;
pub mod java;
pub mod json_schema;
pub mod jvm_types;
pub mod naming;

pub use ir::MessageType;
pub use java::generate_java;
pub use json_schema::generate_json_schema;
