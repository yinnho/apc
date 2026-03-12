//! Client implementation for apc

mod tcp;
mod local;

pub use tcp::TcpClient;
pub use local::LocalClient;

use crate::config::Config;
use crate::error::Result;
use crate::output::CapabilitiesInfo;
use shared::protocol::{Message, AgentInfo};
use shared::url::AgentUrl;

/// Client for connecting to agents
pub struct Client {
    config: Config,
    tcp_client: TcpClient,
}

impl Client {
    /// Create a new client
    pub fn new(config: &Config) -> Result<Self> {
        let tcp_client = TcpClient::new(config)?;
        Ok(Self {
            config: config.clone(),
            tcp_client,
        })
    }

    /// Send a message to an agent
    pub async fn send(&mut self, url: &AgentUrl, message: &str) -> Result<String> {
        if url.is_local() {
            // Use local client
            let local_client = LocalClient::new();
            local_client.send(&url.agent, message).await
        } else {
            // Use TCP client
            self.tcp_client.send(url, message).await
        }
    }

    /// Query agent capabilities
    pub async fn query_capabilities(&mut self, url: &AgentUrl) -> Result<CapabilitiesInfo> {
        self.tcp_client.query_capabilities(url).await
    }

    /// Query all agents on a server
    pub async fn query_all_agents(&mut self, url: &AgentUrl) -> Result<Vec<String>> {
        self.tcp_client.query_all_agents(url).await
    }
}

/// Agent capabilities
#[derive(Debug, Clone)]
pub struct Capabilities {
    pub name: String,
    pub capabilities: Vec<String>,
}

impl From<Capabilities> for CapabilitiesInfo {
    fn from(caps: Capabilities) -> Self {
        CapabilitiesInfo {
            name: caps.name,
            capabilities: caps.capabilities,
        }
    }
}
