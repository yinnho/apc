//! Message serializer for Agent Protocol (JSON-RPC 2.0)

use super::Message;
use super::{JsonRpcRequest, JsonRpcResponse, JsonRpcErrorObject, RequestId};

/// Request ID counter for generating sequential IDs
static REQUEST_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Get next request ID
fn next_request_id() -> RequestId {
    RequestId::Number(REQUEST_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as i64)
}

/// Serialize a message to JSON-RPC format
pub fn serialize_message(msg: &Message) -> String {
    match msg {
        // Request types - serialize as JSON-RPC request
        Message::Hello { client_name, client_version } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                "hello",
                Some(serde_json::json!({
                    "clientName": client_name,
                    "clientVersion": client_version
                })),
            );
            req.to_json()
        }

        Message::Capa { agent } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::GET_AGENT_CARD,
                Some(serde_json::json!({
                    "agentId": agent
                })),
            );
            req.to_json()
        }

        Message::CapaAll => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::LIST_AGENTS,
                None,
            );
            req.to_json()
        }

        Message::CapaServer => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::GET_SERVER_INFO,
                None,
            );
            req.to_json()
        }

        Message::Send { agent, message } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::SEND_MESSAGE,
                Some(serde_json::json!({
                    "agentId": agent,
                    "message": message
                })),
            );
            req.to_json()
        }

        Message::Find { query } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::FIND_AGENTS,
                Some(serde_json::json!({
                    "query": query
                })),
            );
            req.to_json()
        }

        Message::Register { agent } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::REGISTER_AGENT,
                serde_json::to_value(agent).ok(),
            );
            req.to_json()
        }

        Message::Sync { since_version } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::SYNC_CHANGES,
                Some(serde_json::json!({
                    "sinceVersion": since_version
                })),
            );
            req.to_json()
        }

        Message::Bye => {
            let req = JsonRpcRequest::notification("bye", None);
            req.to_json()
        }

        // Response types - serialize as JSON-RPC response
        Message::HelloOk { server_name, server_version } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "serverName": server_name,
                    "serverVersion": server_version
                }),
            );
            resp.to_json()
        }

        Message::Error { code, message } => {
            let resp = JsonRpcResponse::error(
                None::<i64>,
                JsonRpcErrorObject::new(*code as i32, message, None),
            );
            resp.to_json()
        }

        Message::CapaOk { name, capabilities } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "name": name,
                    "capabilities": capabilities
                }),
            );
            resp.to_json()
        }

        Message::CapaAllOk { agents } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "agents": agents
                }),
            );
            resp.to_json()
        }

        Message::CapaServerOk { version, agent_count } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "version": version,
                    "agentCount": agent_count
                }),
            );
            resp.to_json()
        }

        Message::SendOk { response } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "response": response
                }),
            );
            resp.to_json()
        }

        Message::FindOk { results } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "results": results
                }),
            );
            resp.to_json()
        }

        Message::RegisterOk { agent_id } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "agentId": agent_id
                }),
            );
            resp.to_json()
        }

        Message::SyncOk { current_version, changes } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "currentVersion": current_version,
                    "changes": changes
                }),
            );
            resp.to_json()
        }

        Message::ByeOk => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "bye": true
                }),
            );
            resp.to_json()
        }

        // Streaming messages
        Message::SubscribeTask { task_id } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::SUBSCRIBE_TASK,
                Some(serde_json::json!({
                    "taskId": task_id
                })),
            );
            req.to_json()
        }

        Message::SubscribeTaskOk { task_id } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "taskId": task_id,
                    "subscribed": true
                }),
            );
            resp.to_json()
        }

        Message::CancelTask { task_id } => {
            let req = JsonRpcRequest::new(
                next_request_id(),
                super::methods::CANCEL_TASK,
                Some(serde_json::json!({
                    "taskId": task_id
                })),
            );
            req.to_json()
        }

        Message::CancelTaskOk { task_id } => {
            let resp = JsonRpcResponse::success(
                None::<i64>,
                serde_json::json!({
                    "taskId": task_id,
                    "cancelled": true
                }),
            );
            resp.to_json()
        }

        Message::StreamEvent { event } => {
            // Stream events are sent as raw JSON lines (not JSON-RPC)
            event.to_json_line()
        }
    }
}

/// Serialize a message as a JSON-RPC request with a specific ID
pub fn serialize_request_with_id(msg: &Message, id: impl Into<RequestId>) -> String {
    let id = id.into();
    let req = msg.to_jsonrpc_request(id);
    req.to_json()
}

/// Serialize a message as a JSON-RPC response with a specific request ID
pub fn serialize_response(msg: &Message, request_id: Option<RequestId>) -> String {
    match msg {
        Message::HelloOk { server_name, server_version } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "serverName": server_name,
                    "serverVersion": server_version
                }),
            );
            resp.to_json()
        }

        Message::Error { code, message } => {
            let resp = JsonRpcResponse::error(
                request_id,
                JsonRpcErrorObject::new(*code as i32, message, None),
            );
            resp.to_json()
        }

        Message::CapaOk { name, capabilities } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "name": name,
                    "capabilities": capabilities
                }),
            );
            resp.to_json()
        }

        Message::CapaAllOk { agents } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "agents": agents
                }),
            );
            resp.to_json()
        }

        Message::CapaServerOk { version, agent_count } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "version": version,
                    "agentCount": agent_count
                }),
            );
            resp.to_json()
        }

        Message::SendOk { response } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "response": response
                }),
            );
            resp.to_json()
        }

        Message::FindOk { results } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "results": results
                }),
            );
            resp.to_json()
        }

        Message::RegisterOk { agent_id } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "agentId": agent_id
                }),
            );
            resp.to_json()
        }

        Message::SyncOk { current_version, changes } => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "currentVersion": current_version,
                    "changes": changes
                }),
            );
            resp.to_json()
        }

        Message::ByeOk => {
            let resp = JsonRpcResponse::success(
                request_id,
                serde_json::json!({
                    "bye": true
                }),
            );
            resp.to_json()
        }

        // For non-response types, just use regular serialization
        _ => serialize_message(msg),
    }
}

/// Serialize a message with newline for network transmission
pub fn serialize_message_line(msg: &Message) -> String {
    let mut s = serialize_message(msg);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Serialize a response with newline for network transmission
pub fn serialize_response_line(msg: &Message, request_id: Option<RequestId>) -> String {
    let mut s = serialize_response(msg, request_id);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_send_request() {
        let msg = Message::Send {
            agent: "hotel".to_string(),
            message: "查询房间".to_string(),
        };
        let json = serialize_message(&msg);
        assert!(json.contains(r#""method":"sendMessage""#));
        assert!(json.contains(r#""agentId":"hotel""#));
        assert!(json.contains(r#""message":"查询房间""#));
    }

    #[test]
    fn test_serialize_send_ok_response() {
        let msg = Message::SendOk {
            response: "查询结果".to_string(),
        };
        let json = serialize_message(&msg);
        assert!(json.contains(r#""result""#));
        assert!(json.contains(r#""response":"查询结果""#));
    }

    #[test]
    fn test_serialize_error_response() {
        let msg = Message::Error {
            code: 404,
            message: "Agent not found".to_string(),
        };
        let json = serialize_message(&msg);
        assert!(json.contains(r#""error""#));
        assert!(json.contains(r#""code":404"#));
        assert!(json.contains(r#""message":"Agent not found""#));
    }

    #[test]
    fn test_serialize_with_newline() {
        let msg = Message::Bye;
        let line = serialize_message_line(&msg);
        assert!(line.ends_with('\n'));
    }

    #[test]
    fn test_serialize_response_with_request_id() {
        let msg = Message::SendOk {
            response: "ok".to_string(),
        };
        let json = serialize_response(&msg, Some(RequestId::Number(42)));
        assert!(json.contains(r#""id":42"#));
    }

    #[test]
    fn test_roundtrip() {
        let original = Message::Send {
            agent: "hotel".to_string(),
            message: "查询房间".to_string(),
        };
        let serialized = serialize_message(&original);
        let parsed = super::super::parse_message(&serialized).unwrap();
        assert_eq!(original, parsed);
    }
}
