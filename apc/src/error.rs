//! Error handling for apc

pub type Result<T> = std::result::Result<T, Error>;

/// apc error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("URL error: {0}")]
    Url(String),

    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout")]
    Timeout,

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Agent not found: {0}")]
    NotFound(String),
}

impl Error {
    /// Create error from protocol error code
    pub fn from_protocol(code: u16, message: String) -> Self {
        match code {
            401 => Error::Auth(message),
            404 => Error::NotFound(message),
            _ => Error::Connection(format!("Error {}: {}", code, message)),
        }
    }
}

impl From<shared::protocol::ProtocolError> for Error {
    fn from(e: shared::protocol::ProtocolError) -> Self {
        Error::Protocol(e.to_string())
    }
}

impl From<shared::url::UrlError> for Error {
    fn from(e: shared::url::UrlError) -> Self {
        Error::Url(e.to_string())
    }
}

impl From<shared::cert::CertificateError> for Error {
    fn from(e: shared::cert::CertificateError) -> Self {
        Error::Certificate(e.to_string())
    }
}

impl From<shared::error::Error> for Error {
    fn from(e: shared::error::Error) -> Self {
        Error::Config(e.to_string())
    }
}
