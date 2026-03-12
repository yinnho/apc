//! Certificate types and utilities for Agent Protocol
//!
//! Provides certificate generation, verification, and management

mod types;
mod ca;
mod verify;

pub use types::*;
pub use ca::*;
pub use verify::*;

use thiserror::Error;

/// Certificate errors
#[derive(Error, Debug)]
pub enum CertificateError {
    #[error("Certificate generation failed: {0}")]
    GenerationFailed(String),

    #[error("Certificate parsing failed: {0}")]
    ParsingFailed(String),

    #[error("Certificate verification failed: {0}")]
    VerificationFailed(String),

    #[error("Certificate expired")]
    Expired,

    #[error("Certificate not yet valid")]
    NotYetValid,

    #[error("Certificate revoked")]
    Revoked,

    #[error("Certificate not registered")]
    NotRegistered,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PEM error: {0}")]
    PemError(String),
}
