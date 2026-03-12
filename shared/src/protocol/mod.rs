//! Agent Protocol types and utilities
//!
//! This module provides protocol types for the Agent Protocol.

mod message;
mod parser;
mod serializer;
mod jsonrpc;
mod agent_card;
mod sse;

pub use message::*;
pub use parser::*;
pub use serializer::*;
pub use jsonrpc::*;
pub use agent_card::*;
pub use sse::*;

/// Serialize a message to JSON-RPC format
pub fn serialize_message(msg: &Message) -> String {
    serializer::serialize_message(msg)
}

/// Parse a message from JSON-RPC format
pub fn parse_message(input: &str) -> Result<Message, ProtocolError> {
    parser::parse_message(input)
}
