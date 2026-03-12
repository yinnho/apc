//! Registry client for apc

use crate::config::Config;
use crate::error::{Error, Result};
use shared::protocol::AgentInfo;
use shared::url::AgentUrl;

/// Registry client for querying agent information
pub struct RegistryClient {
    config: Config,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Find agents matching a query
    pub async fn find(&self, query: &str) -> Result<Vec<AgentInfo>> {
        // Try mirrors first, then fallback to primary
        let mut urls_to_try: Vec<String> = vec![];

        for mirror in &self.config.registry.mirrors {
            urls_to_try.push(mirror.clone());
        }

        if self.config.registry.fallback_to_primary {
            urls_to_try.push(self.config.registry.primary.clone());
        }

        let mut last_error = None;

        for url_str in urls_to_try {
            match self.find_from_registry(&url_str, query).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    tracing::warn!("Registry {} failed: {}", url_str, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Registry("No registry available".to_string())))
    }

    /// Find agents from a specific registry
    async fn find_from_registry(&self, registry_url: &str, query: &str) -> Result<Vec<AgentInfo>> {
        let url = AgentUrl::parse(&format!("{}/find?{}", registry_url, query))?;

        // Connect to registry
        let tcp_client = crate::client::TcpClient::new(&self.config)?;

        // For now, we'll use a simple implementation
        // In a full implementation, this would use the FIND protocol message
        let mut stream = tokio::net::TcpStream::connect(url.address()).await
            .map_err(|e| Error::Connection(format!("Failed to connect to registry: {}", e)))?;

        // Send handshake
        let hello = shared::protocol::Message::Hello {
            client_name: "apc".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Use a simple send/receive pattern
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let hello_str = shared::protocol::serialize_message(&hello);
        stream.write_all(hello_str.as_bytes()).await
            .map_err(|e| Error::Connection(e.to_string()))?;
        stream.write_all(b"\n").await
            .map_err(|e| Error::Connection(e.to_string()))?;

        // Read response
        let mut buffer = vec![0u8; 4096];
        let n = stream.read(&mut buffer).await
            .map_err(|e| Error::Connection(e.to_string()))?;
        let response_str = String::from_utf8_lossy(&buffer[..n]);

        // Parse response
        let response = shared::protocol::parse_message(&response_str)
            .map_err(|e| Error::Protocol(e.to_string()))?;

        match response {
            shared::protocol::Message::HelloOk { .. } => {
                // Send FIND
                let find_msg = shared::protocol::Message::Find {
                    query: query.to_string(),
                };
                let find_str = shared::protocol::serialize_message(&find_msg);
                stream.write_all(find_str.as_bytes()).await
                    .map_err(|e| Error::Connection(e.to_string()))?;
                stream.write_all(b"\n").await
                    .map_err(|e| Error::Connection(e.to_string()))?;

                // Read FIND response
                let n = stream.read(&mut buffer).await
                    .map_err(|e| Error::Connection(e.to_string()))?;
                let response_str = String::from_utf8_lossy(&buffer[..n]);
                let find_response = shared::protocol::parse_message(&response_str)
                    .map_err(|e| Error::Protocol(e.to_string()))?;

                match find_response {
                    shared::protocol::Message::FindOk { results } => Ok(results),
                    shared::protocol::Message::Error { code, message } => {
                        Err(Error::from_protocol(code, message))
                    }
                    _ => Err(Error::Registry(format!(
                        "Unexpected response: {:?}",
                        find_response
                    ))),
                }
            }
            shared::protocol::Message::Error { code, message } => {
                Err(Error::from_protocol(code, message))
            }
            _ => Err(Error::Registry(format!(
                "Unexpected response: {:?}",
                response
            ))),
        }
    }

    /// Register this agent with the registry
    pub async fn register(
        &self,
        _agent_url: &str,
        _cert_path: &str,
        _key_path: &str,
    ) -> Result<()> {
        // In a full implementation, this would:
        // 1. Load the certificate
        // 2. Create an AgentRegistration
        // 3. Send REGISTER message to registry
        // 4. Handle the response
        Err(Error::Internal("Registration not implemented yet".to_string()))
    }
}
