//! Qubit Protocol
//! Shared data structures, RPC protocols, and type definitions for QubitCore.
//! Supports serialization to JSON and TypeScript type generation via ts-rs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Fast direct execution without persistent sub-agents
    Reflex,
    /// Complex multi-step reasoning with sub-agent delegation
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct ChatMessage {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct ToolResult {
    pub call_id: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Planning,
    Executing,
    WaitingForInput,
    Failed,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct AgentDescriptor {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub memory_usage_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
#[serde(tag = "type", content = "payload")]
pub enum WebSocketEvent {
    AgentStatusChanged(AgentDescriptor),
    TaskProgress {
        task_id: Uuid,
        percent: f32,
        status: String,
    },
    MessageReceived(ChatMessage),
    ToolInvoked(ToolCall),
    ToolCompleted(ToolResult),
    SystemAlert {
        level: String,
        message: String,
    },
}
