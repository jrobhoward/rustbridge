//! rustbridge-transport - JSON codec and serialization layer
//!
//! This crate provides:
//! - [`Codec`] trait for encoding/decoding messages
//! - [`JsonCodec`] implementation for JSON transport
//! - [`RequestEnvelope`] and [`ResponseEnvelope`] for message framing

mod codec;
mod envelope;

pub use codec::{Codec, CodecError, JsonCodec};
pub use envelope::{RequestEnvelope, ResponseEnvelope, ResponseStatus};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        Codec, CodecError, JsonCodec, RequestEnvelope, ResponseEnvelope, ResponseStatus,
    };
}
