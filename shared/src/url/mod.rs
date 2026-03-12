//! URL parsing for Agent Protocol
//!
//! URL format: agent://<host>[:<port>]/<agent-name>
//!
//! Examples:
//! - agent://hotel.example.com/booking
//! - agent://airline.example.com:86/ticket
//! - agent://localhost/claude
//! - agent://local/claude (local CLI execution)
//! - agent://registry/find?type=hotel

mod parser;

pub use parser::*;

use thiserror::Error;

/// Agent URL
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUrl {
    /// Host (e.g., "hotel.example.com" or "local" or "localhost")
    pub host: String,

    /// Port (default: 86)
    pub port: u16,

    /// Agent name (e.g., "booking" or "claude")
    pub agent: String,

    /// Query parameters (for registry queries)
    pub query: Option<String>,
}

/// URL parsing errors
#[derive(Error, Debug)]
pub enum UrlError {
    #[error("Invalid URL format: {0}")]
    InvalidFormat(String),

    #[error("Invalid scheme: expected 'agent://', got '{0}'")]
    InvalidScheme(String),

    #[error("Missing host")]
    MissingHost,

    #[error("Missing agent name")]
    MissingAgent,

    #[error("Invalid port: {0}")]
    InvalidPort(String),

    #[error("URL parse error: {0}")]
    ParseError(String),
}

impl AgentUrl {
    /// Default port for Agent Protocol
    pub const DEFAULT_PORT: u16 = 86;

    /// Create a new Agent URL
    pub fn new(host: String, port: Option<u16>, agent: String) -> Self {
        Self {
            host,
            port: port.unwrap_or(Self::DEFAULT_PORT),
            agent,
            query: None,
        }
    }

    /// Parse an agent URL string
    pub fn parse(url: &str) -> Result<Self, UrlError> {
        parser::parse_agent_url(url)
    }

    /// Check if this is a local URL (agent://local/...)
    pub fn is_local(&self) -> bool {
        self.host == "local"
    }

    /// Check if this is localhost
    pub fn is_localhost(&self) -> bool {
        self.host == "localhost" || self.host == "127.0.0.1" || self.host == "::1"
    }

    /// Check if this is a registry URL
    pub fn is_registry(&self) -> bool {
        self.host == "registry" || self.agent == "find"
    }

    /// Get the full address string (host:port)
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get the full URL string
    pub fn to_string_with_port(&self) -> String {
        if self.query.is_some() {
            format!("agent://{}:{}/{}?{}", self.host, self.port, self.agent, self.query.as_ref().unwrap())
        } else {
            format!("agent://{}:{}/{}", self.host, self.port, self.agent)
        }
    }

    /// Convert to TCP address (first resolved)
    pub fn to_socket_addr(&self) -> Result<std::net::SocketAddr, std::io::Error> {
        use std::net::ToSocketAddrs;
        let addr = format!("{}:{}", self.host, self.port);
        addr.to_socket_addrs()?
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Could not resolve address"))
    }

    /// Get all resolved socket addresses
    pub fn to_socket_addrs(&self) -> Result<Vec<std::net::SocketAddr>, std::io::Error> {
        use std::net::ToSocketAddrs;
        let addr = format!("{}:{}", self.host, self.port);
        addr.to_socket_addrs().map(|iter| iter.collect())
    }
}

impl std::fmt::Display for AgentUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.port == Self::DEFAULT_PORT {
            if let Some(ref query) = self.query {
                write!(f, "agent://{}/{}?{}", self.host, self.agent, query)
            } else {
                write!(f, "agent://{}/{}", self.host, self.agent)
            }
        } else {
            if let Some(ref query) = self.query {
                write!(f, "agent://{}:{}/{}?{}", self.host, self.port, self.agent, query)
            } else {
                write!(f, "agent://{}:{}/{}", self.host, self.port, self.agent)
            }
        }
    }
}

impl std::str::FromStr for AgentUrl {
    type Err = UrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let url = AgentUrl::parse("agent://hotel.example.com/booking").unwrap();
        assert_eq!(url.host, "hotel.example.com");
        assert_eq!(url.port, 86);
        assert_eq!(url.agent, "booking");
        assert!(!url.is_local());
    }

    #[test]
    fn test_parse_with_port() {
        let url = AgentUrl::parse("agent://hotel.example.com:8086/booking").unwrap();
        assert_eq!(url.host, "hotel.example.com");
        assert_eq!(url.port, 8086);
        assert_eq!(url.agent, "booking");
    }

    #[test]
    fn test_parse_local() {
        let url = AgentUrl::parse("agent://local/claude").unwrap();
        assert_eq!(url.host, "local");
        assert_eq!(url.agent, "claude");
        assert!(url.is_local());
    }

    #[test]
    fn test_parse_localhost() {
        let url = AgentUrl::parse("agent://localhost/test").unwrap();
        assert_eq!(url.host, "localhost");
        assert!(url.is_localhost());
    }

    #[test]
    fn test_parse_with_query() {
        let url = AgentUrl::parse("agent://registry/find?type=hotel&location=Beijing").unwrap();
        assert_eq!(url.host, "registry");
        assert_eq!(url.agent, "find");
        assert_eq!(url.query, Some("type=hotel&location=Beijing".to_string()));
        assert!(url.is_registry());
    }

    #[test]
    fn test_display() {
        let url = AgentUrl::parse("agent://hotel.example.com/booking").unwrap();
        assert_eq!(url.to_string(), "agent://hotel.example.com/booking");

        let url = AgentUrl::parse("agent://hotel.example.com:8086/booking").unwrap();
        assert_eq!(url.to_string(), "agent://hotel.example.com:8086/booking");
    }
}
