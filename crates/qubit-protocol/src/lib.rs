use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Reflex,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub tainted: bool,
    pub created_at: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(
        session_id: Uuid,
        role: MessageRole,
        content: impl Into<String>,
        tainted: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            role,
            content: content.into(),
            tainted,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub rss_mb: f64,
    pub wal_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecuteTaskRequest {
    pub session_id: Option<Uuid>,
    pub prompt: String,
    pub mode: Option<ExecutionMode>,
    #[serde(default)]
    pub tainted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecuteTaskResponse {
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub mode: ExecutionMode,
    pub reply: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ElicitationStatus {
    Pending,
    Approved,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ElicitationRequest {
    pub id: Uuid,
    pub session_id: Uuid,
    pub tool_name: String,
    pub args_preview: String,
    pub status: ElicitationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketEvent {
    TaskStarted { session_id: Uuid, task_id: Uuid, mode: ExecutionMode },
    TokenDelta { task_id: Uuid, delta: String },
    ElicitationRequired { request: ElicitationRequest },
    TaskCompleted { session_id: Uuid, task_id: Uuid, reply: String },
    Error { message: String },
}
