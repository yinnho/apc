//! Message types for Agent Protocol
//!
//! Protocol message types with JSON-RPC 2.0 mapping.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{JsonRpcRequest, JsonRpcResponse, JsonRpcErrorObject, RequestId};
use super::sse::StreamEvent;

/// Protocol version
pub const PROTOCOL_VERSION: &str = "0.2";
pub const DEFAULT_PORT: u16 = 86;

/// JSON-RPC method names
pub mod methods {
    pub const SEND_MESSAGE: &str = "sendMessage";
    pub const GET_AGENT_CARD: &str = "getAgentCard";
    pub const LIST_AGENTS: &str = "listAgents";
    pub const FIND_AGENTS: &str = "findAgents";
    pub const REGISTER_AGENT: &str = "registerAgent";
    pub const UNREGISTER_AGENT: &str = "unregisterAgent";
    pub const SYNC_CHANGES: &str = "syncChanges";
    pub const GET_SERVER_INFO: &str = "getServerInfo";
    pub const SUBSCRIBE_TASK: &str = "subscribeTask";
    pub const CANCEL_TASK: &str = "cancelTask";
}

/// Agent Protocol message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Message {
    /// Client hello
    Hello {
        client_name: String,
        client_version: String,
    },

    /// Server hello response
    HelloOk {
        server_name: String,
        server_version: String,
    },

    /// Error response
    Error {
        code: u16,
        message: String,
    },

    /// Query capabilities
    Capa {
        agent: String,
    },

    /// Capabilities response
    CapaOk {
        name: String,
        capabilities: Vec<String>,
    },

    /// List all agents
    CapaAll,

    /// List all agents response
    CapaAllOk {
        agents: Vec<String>,
    },

    /// Server info
    CapaServer,

    /// Server info response
    CapaServerOk {
        version: String,
        agent_count: usize,
    },

    /// Send message to agent
    Send {
        agent: String,
        message: String,
    },

    /// Send response
    SendOk {
        response: String,
    },

    /// Find agents
    Find {
        query: String,
    },

    /// Find response
    FindOk {
        results: Vec<AgentInfo>,
    },

    /// Goodbye
    Bye,

    /// Goodbye response
    ByeOk,

    /// Register agent (to registry)
    Register {
        agent: AgentRegistration,
    },

    /// Register response
    RegisterOk {
        agent_id: String,
    },

    /// Sync request (mirror to main)
    Sync {
        since_version: u64,
    },

    /// Sync response
    SyncOk {
        current_version: u64,
        changes: Vec<Change>,
    },

    // Streaming messages

    /// Subscribe to task updates
    SubscribeTask {
        task_id: String,
    },

    /// Task subscription confirmed
    SubscribeTaskOk {
        task_id: String,
    },

    /// Cancel a running task
    CancelTask {
        task_id: String,
    },

    /// Task cancelled confirmation
    CancelTaskOk {
        task_id: String,
    },

    /// Stream event (server -> client streaming)
    StreamEvent {
        event: StreamEvent,
    },
}

impl Message {
    /// Get the message type name
    pub fn type_name(&self) -> &'static str {
        match self {
            Message::Hello { .. } => "HELLO",
            Message::HelloOk { .. } => "HELLO_OK",
            Message::Error { .. } => "ERROR",
            Message::Capa { .. } => "CAPA",
            Message::CapaOk { .. } => "CAPA_OK",
            Message::CapaAll => "CAPA_ALL",
            Message::CapaAllOk { .. } => "CAPA_ALL_OK",
            Message::CapaServer => "CAPA_SERVER",
            Message::CapaServerOk { .. } => "CAPA_SERVER_OK",
            Message::Send { .. } => "SEND",
            Message::SendOk { .. } => "SEND_OK",
            Message::Find { .. } => "FIND",
            Message::FindOk { .. } => "FIND_OK",
            Message::Bye => "BYE",
            Message::ByeOk => "BYE_OK",
            Message::Register { .. } => "REGISTER",
            Message::RegisterOk { .. } => "REGISTER_OK",
            Message::Sync { .. } => "SYNC",
            Message::SyncOk { .. } => "SYNC_OK",
            Message::SubscribeTask { .. } => "SUBSCRIBE_TASK",
            Message::SubscribeTaskOk { .. } => "SUBSCRIBE_TASK_OK",
            Message::CancelTask { .. } => "CANCEL_TASK",
            Message::CancelTaskOk { .. } => "CANCEL_TASK_OK",
            Message::StreamEvent { .. } => "STREAM_EVENT",
        }
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        matches!(self, Message::Error { .. })
    }

    /// Check if this is an OK response
    pub fn is_ok(&self) -> bool {
        matches!(
            self,
            Message::HelloOk { .. }
                | Message::CapaOk { .. }
                | Message::CapaAllOk { .. }
                | Message::CapaServerOk { .. }
                | Message::SendOk { .. }
                | Message::FindOk { .. }
                | Message::RegisterOk { .. }
                | Message::SyncOk { .. }
                | Message::SubscribeTaskOk { .. }
                | Message::CancelTaskOk { .. }
        )
    }

    /// Check if this is a streaming event
    pub fn is_stream_event(&self) -> bool {
        matches!(self, Message::StreamEvent { .. })
    }

    /// Convert to JSON-RPC request
    pub fn to_jsonrpc_request(&self, id: impl Into<RequestId>) -> JsonRpcRequest {
        let id = id.into();
        match self {
            Message::Hello { client_name, client_version } => {
                // Hello is handled at connection level, not as JSON-RPC
                JsonRpcRequest::new(id, "hello", Some(serde_json::json!({
                    "clientName": client_name,
                    "clientVersion": client_version
                })))
            }

            Message::Capa { agent } => {
                JsonRpcRequest::new(id, methods::GET_AGENT_CARD, Some(serde_json::json!({
                    "agentId": agent
                })))
            }

            Message::CapaAll => {
                JsonRpcRequest::new(id, methods::LIST_AGENTS, None)
            }

            Message::CapaServer => {
                JsonRpcRequest::new(id, methods::GET_SERVER_INFO, None)
            }

            Message::Send { agent, message } => {
                JsonRpcRequest::new(id, methods::SEND_MESSAGE, Some(serde_json::json!({
                    "agentId": agent,
                    "message": message
                })))
            }

            Message::Find { query } => {
                JsonRpcRequest::new(id, methods::FIND_AGENTS, Some(serde_json::json!({
                    "query": query
                })))
            }

            Message::Register { agent } => {
                JsonRpcRequest::new(id, methods::REGISTER_AGENT, Some(serde_json::to_value(agent).unwrap_or_default()))
            }

            Message::Sync { since_version } => {
                JsonRpcRequest::new(id, methods::SYNC_CHANGES, Some(serde_json::json!({
                    "sinceVersion": since_version
                })))
            }

            Message::Bye => {
                JsonRpcRequest::notification("bye", None)
            }

            Message::SubscribeTask { task_id } => {
                JsonRpcRequest::new(id, methods::SUBSCRIBE_TASK, Some(serde_json::json!({
                    "taskId": task_id
                })))
            }

            Message::CancelTask { task_id } => {
                JsonRpcRequest::new(id, methods::CANCEL_TASK, Some(serde_json::json!({
                    "taskId": task_id
                })))
            }

            // Response types should not be converted to requests
            _ => JsonRpcRequest::new(id, "unknown", None),
        }
    }

    /// Convert from JSON-RPC response
    pub fn from_jsonrpc_response(response: &JsonRpcResponse) -> Result<Self, ProtocolError> {
        if let Some(error) = &response.error {
            return Ok(Message::Error {
                code: error.code as u16,
                message: error.message.clone(),
            });
        }

        let result = response.result.as_ref().ok_or_else(|| {
            ProtocolError::InvalidFormat("Response has no result".to_string())
        })?;

        // Try to parse based on result structure
        if let Some(response) = result.get("response") {
            return Ok(Message::SendOk {
                response: response.as_str().unwrap_or_default().to_string(),
            });
        }

        if let Some(name) = result.get("name") {
            let capabilities = result.get("capabilities")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            return Ok(Message::CapaOk {
                name: name.as_str().unwrap_or_default().to_string(),
                capabilities,
            });
        }

        if let Some(agents) = result.get("agents") {
            let agent_list = agents.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            return Ok(Message::CapaAllOk { agents: agent_list });
        }

        if let Some(version) = result.get("version") {
            let agent_count = result.get("agentCount")
                .and_then(|c| c.as_u64())
                .unwrap_or(0) as usize;
            return Ok(Message::CapaServerOk {
                version: version.as_str().unwrap_or_default().to_string(),
                agent_count,
            });
        }

        if let Some(results) = result.get("results") {
            let agent_results: Vec<AgentInfo> = serde_json::from_value(results.clone())
                .unwrap_or_default();
            return Ok(Message::FindOk { results: agent_results });
        }

        if let Some(agent_id) = result.get("agentId") {
            return Ok(Message::RegisterOk {
                agent_id: agent_id.as_str().unwrap_or_default().to_string(),
            });
        }

        if let Some(current_version) = result.get("currentVersion") {
            let changes = result.get("changes")
                .and_then(|c| serde_json::from_value(c.clone()).ok())
                .unwrap_or_default();
            return Ok(Message::SyncOk {
                current_version: current_version.as_u64().unwrap_or(0),
                changes,
            });
        }

        if let Some(server_name) = result.get("serverName") {
            let server_version = result.get("serverVersion")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            return Ok(Message::HelloOk {
                server_name: server_name.as_str().unwrap_or_default().to_string(),
                server_version: server_version.to_string(),
            });
        }

        // Streaming responses
        if let Some(task_id) = result.get("taskId") {
            let task_id = task_id.as_str().unwrap_or_default().to_string();

            // SubscribeTaskOk
            if result.get("subscribed").is_some() {
                return Ok(Message::SubscribeTaskOk { task_id });
            }

            // CancelTaskOk
            if result.get("cancelled").is_some() {
                return Ok(Message::CancelTaskOk { task_id });
            }
        }

        // StreamEvent
        if result.get("type").is_some() {
            if let Ok(event) = serde_json::from_value::<StreamEvent>(result.clone()) {
                return Ok(Message::StreamEvent { event });
            }
        }

        // Default: treat as SendOk
        Ok(Message::SendOk {
            response: result.to_string(),
        })
    }
}

/// Agent information in registry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub address: String,
    pub capabilities: Vec<String>,
    pub rating: Option<f32>,
    pub location: Option<Location>,
}

/// Agent location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub city: Option<String>,
    pub country: Option<String>,
    pub coordinates: Option<[f64; 2]>,
}

/// Agent registration data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub version: String,
    pub address: AgentAddress,
    pub owner: Owner,
    pub capabilities: Vec<Capability>,
    pub metadata: serde_yaml::Value,
    pub public_key: String,
}

/// Agent address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentAddress {
    pub host: String,
    pub port: u16,
    pub url: String,
}

/// Owner information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owner {
    pub id: String,
    pub name: String,
    pub verified: bool,
}

/// Capability definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Change for sync protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub action: ChangeAction,
    pub path: String,
    pub content: Option<String>,
}

/// Change action type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeAction {
    Add,
    Update,
    Delete,
}

/// Protocol errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Unknown message type: {0}")]
    UnknownType(String),

    #[error("Invalid version: expected {expected}, got {actual}")]
    InvalidVersion { expected: String, actual: String },

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unexpected message: {0}")]
    UnexpectedMessage(String),

    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_name() {
        let msg = Message::Send {
            agent: "hotel".to_string(),
            message: "hello".to_string(),
        };
        assert_eq!(msg.type_name(), "SEND");
    }

    #[test]
    fn test_message_to_jsonrpc() {
        let msg = Message::Send {
            agent: "hotel".to_string(),
            message: "查询房间".to_string(),
        };
        let req = msg.to_jsonrpc_request(1);
        assert_eq!(req.method, "sendMessage");
        assert_eq!(req.id, Some(RequestId::Number(1)));
    }

    #[test]
    fn test_message_from_jsonrpc_response() {
        let response = JsonRpcResponse::success(
            Some(1),
            serde_json::json!({"response": "查询结果"}),
        );
        let msg = Message::from_jsonrpc_response(&response).unwrap();
        assert_eq!(msg, Message::SendOk {
            response: "查询结果".to_string(),
        });
    }

    #[test]
    fn test_message_from_jsonrpc_error() {
        let response = JsonRpcResponse::error(
            Some(1),
            JsonRpcErrorObject::new(404, "Agent not found", None),
        );
        let msg = Message::from_jsonrpc_response(&response).unwrap();
        assert!(msg.is_error());
    }
}
