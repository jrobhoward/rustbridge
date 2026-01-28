//! JSON Schema generation from Rust message types.
//!
//! This module provides functionality to generate JSON Schema from Rust structs
//! marked with `#[derive(Serialize, Deserialize)]` or `#[derive(Message)]`.
//!
//! # Use Cases
//!
//! - API documentation and contract definition
//! - Validation schemas for JSON payloads
//! - OpenAPI/Swagger integration
//! - Self-documenting plugin bundles
//!
//! # Usage
//!
//! ## Command Line
//!
//! ```bash
//! rustbridge generate json-schema -i src/messages.rs -o schema.json
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
//! The generated JSON Schema will include proper types, documentation from doc comments,
//! and required field annotations.

pub mod ir;
pub mod json_schema;

pub use ir::MessageType;
pub use json_schema::generate_json_schema;
