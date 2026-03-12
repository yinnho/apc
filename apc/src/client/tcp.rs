//! TCP client implementation for JSON-RPC protocol

use std::time::Duration;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::output::CapabilitiesInfo;
use shared::protocol::{parse_message, parse_response, serialize_request_with_id, Message, RequestId};
use shared::url::AgentUrl;

/// Global request ID counter
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Get next request ID
fn next_request_id() -> RequestId {
    RequestId::Number(REQUEST_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as i64)
}

/// TCP client for connecting to remote agents
pub struct TcpClient {
    timeout: Duration,
}

impl TcpClient {
    /// Create a new TCP client
    pub fn new(config: &Config) -> Result<Self> {
        let timeout = Duration::from_secs(
            config.connection.as_ref().map(|c| c.timeout).unwrap_or(30),
        );
        Ok(Self { timeout })
    }

    /// Connect to an agent
    async fn connect(&self, url: &AgentUrl) -> Result<TcpStream> {
        let addr = url.to_socket_addr()
            .map_err(|e| Error::Connection(format!("Failed to resolve address: {}", e)))?;

        let stream = tokio::time::timeout(self.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|e| Error::Connection(format!("Failed to connect: {}", e)))?;

        Ok(stream)
    }

    /// Perform handshake with the server
    async fn handshake(&self, stream: &mut TcpStream) -> Result<()> {
        // Send HELLO as JSON-RPC
        let hello = Message::Hello {
            client_name: "apc".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let id = next_request_id();
        self.send_message(stream, &hello, id.clone()).await?;

        // Wait for response
        let (response, _) = self.read_message(stream).await?;
        match response {
            Message::HelloOk { .. } => Ok(()),
            Message::Error { code, message } => Err(Error::from_protocol(code, message)),
            _ => Err(Error::Protocol(format!(
                "Unexpected message: {}",
                response.type_name()
            ))),
        }
    }

    /// Send a message to an agent
    pub async fn send(&self, url: &AgentUrl, message: &str) -> Result<String> {
        let mut stream = self.connect(url).await?;
        self.handshake(&mut stream).await?;

        // Send message
        let send_msg = Message::Send {
            agent: url.agent.clone(),
            message: message.to_string(),
        };
        let id = next_request_id();
        self.send_message(&mut stream, &send_msg, id.clone()).await?;

        // Read response
        let (response, _) = self.read_message(&mut stream).await?;
        match response {
            Message::SendOk { response } => {
                // Send BYE
                let bye_id = next_request_id();
                self.send_message(&mut stream, &Message::Bye, bye_id).await?;
                Ok(response)
            }
            Message::Error { code, message } => Err(Error::from_protocol(code, message)),
            _ => Err(Error::Protocol(format!(
                "Unexpected message: {}",
                response.type_name()
            ))),
        }
    }

    /// Query agent capabilities
    pub async fn query_capabilities(&self, url: &AgentUrl) -> Result<CapabilitiesInfo> {
        let mut stream = self.connect(url).await?;
        self.handshake(&mut stream).await?;

        // Send CAPA
        let capa_msg = Message::Capa {
            agent: url.agent.clone(),
        };
        let id = next_request_id();
        self.send_message(&mut stream, &capa_msg, id.clone()).await?;

        // Read response
        let (response, _) = self.read_message(&mut stream).await?;
        match response {
            Message::CapaOk { name, capabilities } => {
                self.send_message(&mut stream, &Message::Bye, next_request_id()).await?;
                Ok(CapabilitiesInfo { name, capabilities })
            }
            Message::Error { code, message } => Err(Error::from_protocol(code, message)),
            _ => Err(Error::Protocol(format!(
                "Unexpected message: {}",
                response.type_name()
            ))),
        }
    }

    /// Query all agents on a server
    pub async fn query_all_agents(&self, url: &AgentUrl) -> Result<Vec<String>> {
        let mut stream = self.connect(url).await?;
        self.handshake(&mut stream).await?;

        // Send CAPA @all
        let id = next_request_id();
        self.send_message(&mut stream, &Message::CapaAll, id.clone()).await?;

        // Read response
        let (response, _) = self.read_message(&mut stream).await?;
        match response {
            Message::CapaAllOk { agents } => {
                self.send_message(&mut stream, &Message::Bye, next_request_id()).await?;
                Ok(agents)
            }
            Message::Error { code, message } => Err(Error::from_protocol(code, message)),
            _ => Err(Error::Protocol(format!(
                "Unexpected message: {}",
                response.type_name()
            ))),
        }
    }

    /// Send a message over the stream
    async fn send_message(&self, stream: &mut TcpStream, msg: &Message, id: RequestId) -> Result<()> {
        let data = serialize_request_with_id(msg, id);
        tracing::debug!("Sending: {}", data.trim());
        stream
            .write_all(data.as_bytes())
            .await
            .map_err(|e| Error::Connection(format!("Failed to send: {}", e)))?;
        stream
            .flush()
            .await
            .map_err(|e| Error::Connection(format!("Failed to flush: {}", e)))?;
        Ok(())
    }

    /// Read a message from the stream
    async fn read_message(&self, stream: &mut TcpStream) -> Result<(Message, Option<RequestId>)> {
        let (reader, _) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        let fut = reader.read_line(&mut line);
        tokio::time::timeout(self.timeout, fut)
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|e| Error::Connection(format!("Failed to read: {}", e)))?;

        tracing::debug!("Received: {}", line.trim());

        // Extract request ID from response
        let request_id = parse_response(&line).ok().and_then(|resp| resp.id);

        // Parse message
        let msg = parse_message(&line).map_err(Error::from)?;

        Ok((msg, request_id))
    }
}
