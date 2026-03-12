//! Message parser for Agent Protocol (JSON-RPC 2.0)

use super::{Message, ProtocolError};
use super::{JsonRpcRequest, JsonRpcResponse, RequestId};
use super::methods;
use super::sse::StreamEvent;

/// Parse a message from JSON-RPC format
pub fn parse_message(input: &str) -> Result<Message, ProtocolError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ProtocolError::InvalidFormat("Empty message".to_string()));
    }

    // Try to parse as JSON-RPC request first
    if input.starts_with('{') {
        return parse_jsonrpc(input);
    }

    Err(ProtocolError::InvalidFormat(
        "Expected JSON-RPC message (must start with '{')".to_string(),
    ))
}

/// Parse JSON-RPC request or response
fn parse_jsonrpc(input: &str) -> Result<Message, ProtocolError> {
    // Try to parse as request
    if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(input) {
        return request_to_message(&req);
    }

    // Try to parse as response
    if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(input) {
        return Message::from_jsonrpc_response(&resp);
    }

    // Try to parse as stream event
    if let Ok(event) = serde_json::from_str::<StreamEvent>(input) {
        return Ok(Message::StreamEvent { event });
    }

    Err(ProtocolError::ParseError("Invalid JSON-RPC format".to_string()))
}

/// Convert JSON-RPC request to Message
fn request_to_message(req: &JsonRpcRequest) -> Result<Message, ProtocolError> {
    match req.method.as_str() {
        "hello" => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let client_name = params.get("clientName")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProtocolError::MissingField("clientName".to_string()))?
                .to_string();
            let client_version = params.get("clientVersion")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProtocolError::MissingField("clientVersion".to_string()))?
                .to_string();
            Ok(Message::Hello { client_name, client_version })
        }

        methods::GET_AGENT_CARD => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let agent = params.get("agentId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProtocolError::MissingField("agentId".to_string()))?
                .to_string();
            Ok(Message::Capa { agent })
        }

        methods::LIST_AGENTS => {
            Ok(Message::CapaAll)
        }

        methods::GET_SERVER_INFO => {
            Ok(Message::CapaServer)
        }

        methods::SEND_MESSAGE => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let agent = params.get("agentId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProtocolError::MissingField("agentId".to_string()))?
                .to_string();
            let message = params.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Ok(Message::Send { agent, message })
        }

        methods::FIND_AGENTS => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let query = params.get("query")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Ok(Message::Find { query })
        }

        methods::REGISTER_AGENT => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let agent: super::AgentRegistration = serde_json::from_value(params.clone())
                .map_err(|e| ProtocolError::ParseError(e.to_string()))?;
            Ok(Message::Register { agent })
        }

        methods::SYNC_CHANGES => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let since_version = params.get("sinceVersion")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| ProtocolError::MissingField("sinceVersion".to_string()))?;
            Ok(Message::Sync { since_version })
        }

        "bye" => {
            Ok(Message::Bye)
        }

        methods::SUBSCRIBE_TASK => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let task_id = params.get("taskId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProtocolError::MissingField("taskId".to_string()))?
                .to_string();
            Ok(Message::SubscribeTask { task_id })
        }

        methods::CANCEL_TASK => {
            let params = req.params.as_ref().ok_or_else(|| {
                ProtocolError::MissingField("params".to_string())
            })?;
            let task_id = params.get("taskId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ProtocolError::MissingField("taskId".to_string()))?
                .to_string();
            Ok(Message::CancelTask { task_id })
        }

        _ => Err(ProtocolError::UnknownType(format!("Unknown method: {}", req.method))),
    }
}

/// Parse a JSON-RPC request
pub fn parse_request(input: &str) -> Result<JsonRpcRequest, ProtocolError> {
    serde_json::from_str(input)
        .map_err(|e| ProtocolError::ParseError(format!("Invalid JSON-RPC request: {}", e)))
}

/// Parse a JSON-RPC response
pub fn parse_response(input: &str) -> Result<JsonRpcResponse, ProtocolError> {
    serde_json::from_str(input)
        .map_err(|e| ProtocolError::ParseError(format!("Invalid JSON-RPC response: {}", e)))
}

/// Extract request ID from a JSON-RPC message
pub fn extract_request_id(input: &str) -> Option<RequestId> {
    if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(input) {
        return req.id;
    }
    if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(input) {
        return resp.id;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_send_message_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"sendMessage","params":{"agentId":"hotel","message":"查询房间"}}"#;
        let msg = parse_message(json).unwrap();
        assert_eq!(
            msg,
            Message::Send {
                agent: "hotel".to_string(),
                message: "查询房间".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_get_agent_card() {
        let json = r#"{"jsonrpc":"2.0","id":2,"method":"getAgentCard","params":{"agentId":"hotel"}}"#;
        let msg = parse_message(json).unwrap();
        assert_eq!(
            msg,
            Message::Capa {
                agent: "hotel".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_list_agents() {
        let json = r#"{"jsonrpc":"2.0","id":3,"method":"listAgents"}"#;
        let msg = parse_message(json).unwrap();
        assert_eq!(msg, Message::CapaAll);
    }

    #[test]
    fn test_parse_response() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"response":"查询结果"}}"#;
        let msg = parse_message(json).unwrap();
        assert_eq!(
            msg,
            Message::SendOk {
                response: "查询结果".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_error_response() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
        let msg = parse_message(json).unwrap();
        assert!(msg.is_error());
    }

    #[test]
    fn test_parse_hello() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"hello","params":{"clientName":"apc","clientVersion":"0.2.0"}}"#;
        let msg = parse_message(json).unwrap();
        assert_eq!(
            msg,
            Message::Hello {
                client_name: "apc".to_string(),
                client_version: "0.2.0".to_string(),
            }
        );
    }

    #[test]
    fn test_extract_request_id() {
        let json = r#"{"jsonrpc":"2.0","id":"abc123","method":"sendMessage","params":{"agentId":"test"}}"#;
        let id = extract_request_id(json);
        assert_eq!(id, Some(RequestId::String("abc123".to_string())));
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"not valid json"#;
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_message() {
        let result = parse_message("");
        assert!(result.is_err());
    }
}
