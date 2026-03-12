//! Server-Sent Events (SSE) types for Agent Protocol
//!
//! SSE is used for streaming task progress and artifacts.

use serde::{Deserialize, Serialize};

/// SSE event types for streaming task updates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// Task status update
    #[serde(rename = "taskStatus")]
    TaskStatusUpdate {
        /// Task ID
        task_id: String,
        /// New status
        status: TaskStatus,
    },

    /// Artifact produced by task
    #[serde(rename = "artifact")]
    ArtifactUpdate {
        /// Task ID
        task_id: String,
        /// Artifact
        artifact: Artifact,
    },

    /// Task completed with result
    #[serde(rename = "taskCompleted")]
    TaskCompleted {
        /// Task ID
        task_id: String,
        /// Final result
        result: serde_json::Value,
    },

    /// Task failed with error
    #[serde(rename = "taskError")]
    TaskError {
        /// Task ID
        task_id: String,
        /// Error details
        error: TaskError,
    },

    /// Heartbeat / keepalive
    #[serde(rename = "heartbeat")]
    Heartbeat {
        /// Timestamp
        timestamp: u64,
    },
}

impl StreamEvent {
    /// Create a task status update event
    pub fn status_update(task_id: &str, status: TaskStatus) -> Self {
        Self::TaskStatusUpdate {
            task_id: task_id.to_string(),
            status,
        }
    }

    /// Create an artifact event
    pub fn artifact(task_id: &str, artifact: Artifact) -> Self {
        Self::ArtifactUpdate {
            task_id: task_id.to_string(),
            artifact,
        }
    }

    /// Create a completion event
    pub fn completed(task_id: &str, result: serde_json::Value) -> Self {
        Self::TaskCompleted {
            task_id: task_id.to_string(),
            result,
        }
    }

    /// Create an error event
    pub fn error(task_id: &str, error: TaskError) -> Self {
        Self::TaskError {
            task_id: task_id.to_string(),
            error,
        }
    }

    /// Create a heartbeat event
    pub fn heartbeat() -> Self {
        Self::Heartbeat {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Get the task ID if this event is associated with a task
    pub fn task_id(&self) -> Option<&str> {
        match self {
            Self::TaskStatusUpdate { task_id, .. } => Some(task_id),
            Self::ArtifactUpdate { task_id, .. } => Some(task_id),
            Self::TaskCompleted { task_id, .. } => Some(task_id),
            Self::TaskError { task_id, .. } => Some(task_id),
            Self::Heartbeat { .. } => None,
        }
    }

    /// Serialize to SSE format
    pub fn to_sse(&self) -> String {
        let event_type = match self {
            Self::TaskStatusUpdate { .. } => "taskStatus",
            Self::ArtifactUpdate { .. } => "artifact",
            Self::TaskCompleted { .. } => "taskCompleted",
            Self::TaskError { .. } => "taskError",
            Self::Heartbeat { .. } => "heartbeat",
        };
        let data = serde_json::to_string(self).unwrap_or_default();
        format!("event: {}\ndata: {}\n\n", event_type, data)
    }

    /// Serialize to JSON line format (for TCP transport)
    pub fn to_json_line(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        format!("{}\n", json)
    }
}

/// Task status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,

    /// Task is running
    Running,

    /// Task is paused (e.g., waiting for input)
    Paused,

    /// Task completed successfully
    Completed,

    /// Task failed
    Failed,

    /// Task was cancelled
    Cancelled,
}

impl TaskStatus {
    /// Check if task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if task is still active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::Running | Self::Paused)
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Artifact produced by a task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact ID
    pub id: String,

    /// Artifact name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// MIME type
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Artifact content (base64 for binary)
    pub content: ArtifactContent,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Artifact {
    /// Create a text artifact
    pub fn text(id: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            mime_type: Some("text/plain".to_string()),
            content: ArtifactContent::Text(content.to_string()),
            metadata: None,
        }
    }

    /// Create a JSON artifact
    pub fn json(id: &str, content: serde_json::Value) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            mime_type: Some("application/json".to_string()),
            content: ArtifactContent::Json(content),
            metadata: None,
        }
    }

    /// Create a binary artifact (base64 encoded)
    pub fn binary(id: &str, mime_type: &str, base64_data: &str) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            mime_type: Some(mime_type.to_string()),
            content: ArtifactContent::Binary(base64_data.to_string()),
            metadata: None,
        }
    }

    /// Create a reference artifact (URL to external resource)
    pub fn reference(id: &str, url: &str) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            mime_type: None,
            content: ArtifactContent::Reference(url.to_string()),
            metadata: None,
        }
    }
}

/// Artifact content variants
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ArtifactContent {
    /// Text content
    #[serde(rename = "text")]
    Text(String),

    /// JSON content
    #[serde(rename = "json")]
    Json(serde_json::Value),

    /// Binary content (base64 encoded)
    #[serde(rename = "binary")]
    Binary(String),

    /// Reference to external resource
    #[serde(rename = "reference")]
    Reference(String),
}

/// Task error details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Error type (e.g., "validation", "timeout", "internal")
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl TaskError {
    /// Create a new task error
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            error_type: None,
            details: None,
        }
    }

    /// Create a validation error
    pub fn validation(message: &str) -> Self {
        Self {
            code: 400,
            message: message.to_string(),
            error_type: Some("validation".to_string()),
            details: None,
        }
    }

    /// Create a timeout error
    pub fn timeout(message: &str) -> Self {
        Self {
            code: 408,
            message: message.to_string(),
            error_type: Some("timeout".to_string()),
            details: None,
        }
    }

    /// Create an internal error
    pub fn internal(message: &str) -> Self {
        Self {
            code: 500,
            message: message.to_string(),
            error_type: Some("internal".to_string()),
            details: None,
        }
    }

    /// Add error type
    pub fn with_type(mut self, error_type: &str) -> Self {
        self.error_type = Some(error_type.to_string());
        self
    }

    /// Add details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref error_type) = self.error_type {
            write!(f, "[{}] {} (code: {})", error_type, self.message, self.code)
        } else {
            write!(f, "{} (code: {})", self.message, self.code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_stream_event_serialization() {
        let event = StreamEvent::status_update("task-123", TaskStatus::Running);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"taskStatus""#));
        assert!(json.contains(r#""status":"running""#));
    }

    #[test]
    fn test_stream_event_sse_format() {
        let event = StreamEvent::heartbeat();
        let sse = event.to_sse();
        assert!(sse.starts_with("event: heartbeat\n"));
        assert!(sse.contains("data:"));
        assert!(sse.ends_with("\n\n"));
    }

    #[test]
    fn test_artifact_text() {
        let artifact = Artifact::text("art-1", "Hello, World!");
        assert_eq!(artifact.id, "art-1");
        assert_eq!(artifact.mime_type, Some("text/plain".to_string()));
        assert_eq!(artifact.content, ArtifactContent::Text("Hello, World!".to_string()));
    }

    #[test]
    fn test_artifact_json() {
        let artifact = Artifact::json("art-2", json!({"result": "ok"}));
        let json = serde_json::to_string(&artifact).unwrap();
        assert!(json.contains(r#""id":"art-2""#));
        assert!(json.contains(r#""mimeType":"application/json""#));
    }

    #[test]
    fn test_task_status() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Running.is_active());
        assert!(!TaskStatus::Completed.is_active());
    }

    #[test]
    fn test_task_error() {
        let error = TaskError::validation("Invalid input")
            .with_details(json!({"field": "email"}));

        assert_eq!(error.code, 400);
        assert_eq!(error.error_type, Some("validation".to_string()));
        assert!(error.details.is_some());
    }

    #[test]
    fn test_event_task_id() {
        let event = StreamEvent::status_update("task-1", TaskStatus::Running);
        assert_eq!(event.task_id(), Some("task-1"));

        let heartbeat = StreamEvent::heartbeat();
        assert_eq!(heartbeat.task_id(), None);
    }
}
