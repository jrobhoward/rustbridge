//! Code generation from Rust message types.
//!
//! This module provides functionality to generate:
//! - JSON Schema definitions
//! - Java/Kotlin classes
//! - (Future) C# classes
//!
//! The code generation pipeline:
//! 1. Parse Rust source files into an IR (Intermediate Representation)
//! 2. Generate target language code from the IR
//!
//! This two-stage approach allows us to:
//! - Share parsing logic across generators
//! - Add new target languages without re-parsing
//! - Apply transformations consistently

pub mod ir;
pub mod java;
pub mod json_schema;

pub use ir::MessageType;
pub use java::generate_java;
pub use json_schema::generate_json_schema;
