//! Generator for TypeScript definitions using ts-rs

use qubit_protocol::*;
use ts_rs::TS;

fn main() {
    println!("Exporting TypeScript types from qubit-protocol...");
    let _ = ExecutionMode::decl();
    let _ = MessageRole::decl();
    let _ = ChatMessage::decl();
    let _ = ToolCall::decl();
    let _ = ToolResult::decl();
    let _ = AgentStatus::decl();
    let _ = AgentDescriptor::decl();
    let _ = WebSocketEvent::decl();
    let _ = CreateAgentRequest::decl();
    let _ = ExecuteTaskRequest::decl();
    let _ = ExecuteTaskResponse::decl();
    let _ = CallToolRequest::decl();
    let _ = HealthResponse::decl();
    let _ = SystemTelemetryDto::decl();
    let _ = ProviderStatusDto::decl();
    let _ = RoutingConfigDto::decl();
    let _ = RoutineDto::decl();
    let _ = McpAclRuleDto::decl();
    let _ = WalCheckpointResultDto::decl();
    println!("TypeScript types successfully declared.");
}
