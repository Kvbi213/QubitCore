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
    TelemetryUpdate(SystemTelemetryDto),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct CreateAgentRequest {
    pub name: String,
    pub role: String,
    pub mode: Option<ExecutionMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct ExecuteTaskRequest {
    pub task: String,
    #[serde(default)]
    pub mode: Option<ExecutionMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct ExecuteTaskResponse {
    pub agent_id: Uuid,
    pub message: ChatMessage,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct CallToolRequest {
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub engine: String,
    pub memory_budget_mb: u32,
    pub uptime_secs: u64,
}

// =============================================================================
// PLASMA EDITION TELEMETRY & MANAGEMENT DTOs
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct SystemTelemetryDto {
    pub memory_rss_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_usage_percent: f32,
    pub tokio_active_threads: u32,
    pub tokio_blocking_threads: u32,
    pub tokio_tasks_total: u64,
    pub anon_pool_available: u32,
    pub anon_pool_total: u32,
    pub aio_connected: bool,
    pub mikrus_connected: bool,
    pub primary_node: String,
    pub network_rx_bps: u64,
    pub network_tx_bps: u64,
    pub wal_file_size_bytes: u64,
    pub ephemeral_window_active: bool,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct ProviderStatusDto {
    pub id: String,
    pub name: String,
    pub kind: String, // "cli_harness", "direct_api", "local_ipc"
    pub is_connected: bool,
    pub model_id: String,
    pub latency_ms: u32,
    pub last_heartbeat: String,
    pub rate_limit_remaining: u32,
    pub auth_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct RoutingConfigDto {
    pub reflex_threshold_ms: u32,
    pub deep_threshold_ms: u32,
    pub enable_auto_escalation: bool,
    pub max_subagent_depth: u32,
    pub cost_ceiling_usd: f32,
    pub fallback_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct RoutineDto {
    pub id: Uuid,
    pub title: String,
    pub cron_expression: String,
    pub target_agent: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub is_active: bool,
    pub execution_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct McpAclRuleDto {
    pub id: String,
    pub tool_pattern: String,
    pub access_level: String, // "allow", "prompt", "deny"
    pub sandbox_required: bool,
    pub max_memory_mb: u32,
    pub network_egress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "protocol/")]
pub struct WalCheckpointResultDto {
    pub success: bool,
    pub checkpoint_mode: String,
    pub wal_pages_checkpointed: i64,
    pub wal_pages_remaining: i64,
    pub duration_ms: u64,
}
