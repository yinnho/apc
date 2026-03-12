//! Shared library for Agent Protocol
//!
//! This crate provides common types, utilities, and protocol implementations
//! shared between apc (client), agentd (server), and registry.

pub mod protocol;
pub mod url;
pub mod cert;
pub mod error;
pub mod util;

// Re-export commonly used types
pub use protocol::{Message, ProtocolError};
pub use url::{AgentUrl, UrlError};
pub use cert::{AgentCertificate, CertificateError};
pub use error::{Error, Result};
