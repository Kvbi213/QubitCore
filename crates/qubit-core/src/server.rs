//! Axum Web & WebSocket Dynamic Routing Server for QubitCore
//! Handles in-process routing for /agents/* and /tools/* without Caddy overhead.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        DefaultBodyLimit, Path, State,
    },
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use qubit_agents::AgentEngine;
use qubit_mcp::McpRegistry;
use qubit_protocol::{
    AgentDescriptor, AgentStatus, CallToolRequest, CreateAgentRequest, ExecuteTaskRequest,
    ExecuteTaskResponse, ExecutionMode, HealthResponse, ToolCall, ToolResult, WebSocketEvent,
};
use qubit_storage::Storage;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    pub mcp: McpRegistry,
    pub engine: Arc<AgentEngine>,
    pub tx: broadcast::Sender<WebSocketEvent>,
    pub start_time: Instant,
}

pub async fn start_server(addr: SocketAddr, db_path: &str) -> anyhow::Result<()> {
    let storage = Storage::open(db_path)?;
    let mcp = McpRegistry::new();
    let engine = Arc::new(AgentEngine::new(storage.clone(), mcp.clone()));
    let (tx, _rx) = broadcast::channel(1024);

    let state = AppState {
        storage,
        mcp,
        engine,
        tx,
        start_time: Instant::now(),
    };

    let app = build_router(state);

    info!("Starting QubitCore daemon (Axum 0.8) on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Constructs the entire application routing tree
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Core System Health & Telemetry
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        // Real-Time Event Stream
        .route("/ws", get(ws_handler))
        // Dynamic Agent Routing (/api/v1/agents/*)
        .route("/api/v1/agents", get(list_agents).post(create_agent))
        .route("/api/v1/agents/:id", get(get_agent))
        .route("/api/v1/agents/:id/execute", post(execute_agent))
        .route("/api/v1/agents/:id/messages", get(get_agent_messages))
        .route("/api/v1/agents/:id/checkpoints", get(get_agent_checkpoints))
        // Dynamic Tool & MCP Routing (/api/v1/tools/*)
        .route("/api/v1/tools", get(list_tools))
        .route("/api/v1/tools/:name/call", post(call_tool))
        .layer(DefaultBodyLimit::max(4 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// =============================================================================
// HEALTH & TELEMETRY HANDLERS
// =============================================================================

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        engine: "qubit-core".to_string(),
        memory_budget_mb: 60,
        uptime_secs: state.start_time.elapsed().as_secs(),
    })
}

// =============================================================================
// AGENT MANAGEMENT HANDLERS (/api/v1/agents/*)
// =============================================================================

/// List all registered agents
async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<AgentDescriptor>>, (StatusCode, Json<serde_json::Value>)> {
    let records = state.storage.list_agents().map_err(|e| {
        error!("Failed to list agents: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Internal server error" })),
        )
    })?;

    let descriptors = records
        .into_iter()
        .map(|r| AgentDescriptor {
            id: Uuid::parse_str(&r.id).unwrap_or_else(|_| Uuid::new_v4()),
            name: r.name,
            role: r.role,
            status: parse_agent_status(&r.status),
            current_task: None,
            memory_usage_bytes: 0,
        })
        .collect();

    Ok(Json(descriptors))
}

/// Create and register a new agent
async fn create_agent(
    State(state): State<AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<AgentDescriptor>), (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::new_v4();
    let id_str = id.to_string();

    state
        .storage
        .upsert_agent(&id_str, &payload.name, &payload.role, "idle")
        .await
        .map_err(|e| {
            error!("Failed to create agent: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal server error" })),
            )
        })?;

    let descriptor = AgentDescriptor {
        id,
        name: payload.name,
        role: payload.role,
        status: AgentStatus::Idle,
        current_task: None,
        memory_usage_bytes: 0,
    };

    // Broadcast status change over WebSocket
    let _ = state
        .tx
        .send(WebSocketEvent::AgentStatusChanged(descriptor.clone()));

    Ok((StatusCode::CREATED, Json(descriptor)))
}

/// Retrieve single agent by UUID
async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentDescriptor>, (StatusCode, Json<serde_json::Value>)> {
    let id_str = id.to_string();
    let record = state
        .storage
        .get_agent(&id_str)
        .map_err(|e| {
            error!("Database error fetching agent {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal server error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Agent '{}' not found", id) })),
            )
        })?;

    Ok(Json(AgentDescriptor {
        id,
        name: record.name,
        role: record.role,
        status: parse_agent_status(&record.status),
        current_task: None,
        memory_usage_bytes: 0,
    }))
}

/// Execute a task for an agent
async fn execute_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ExecuteTaskRequest>,
) -> Result<Json<ExecuteTaskResponse>, (StatusCode, Json<serde_json::Value>)> {
    let mode = payload.mode.unwrap_or(ExecutionMode::Reflex);

    // Notify task progress on WebSocket
    let _ = state.tx.send(WebSocketEvent::TaskProgress {
        task_id: Uuid::new_v4(),
        percent: 0.1,
        status: format!("Executing task for agent {}", id),
    });

    let message = state
        .engine
        .execute_agent_task(id, mode, payload.task)
        .await
        .map_err(|e| {
            error!("Agent execution failed for agent {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to execute agent task" })),
            )
        })?;

    // Notify message received
    let _ = state
        .tx
        .send(WebSocketEvent::MessageReceived(message.clone()));

    // Notify task completed
    let _ = state.tx.send(WebSocketEvent::TaskProgress {
        task_id: Uuid::new_v4(),
        percent: 1.0,
        status: "Completed".to_string(),
    });

    Ok(Json(ExecuteTaskResponse {
        agent_id: id,
        message,
        status: AgentStatus::Idle,
    }))
}

/// Retrieve message history for an agent
async fn get_agent_messages(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<qubit_storage::ChatMessageRecord>>, (StatusCode, Json<serde_json::Value>)> {
    let id_str = id.to_string();
    let messages = state
        .storage
        .get_messages(Some(&id_str), 100)
        .map_err(|e| {
            error!("Database error fetching messages for agent {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal server error" })),
            )
        })?;

    Ok(Json(messages))
}

/// Retrieve checkpoint history for an agent
async fn get_agent_checkpoints(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<qubit_storage::CheckpointRecord>>, (StatusCode, Json<serde_json::Value>)> {
    let id_str = id.to_string();
    let checkpoints = state.storage.get_checkpoints(&id_str).map_err(|e| {
        error!("Database error fetching checkpoints for agent {}: {}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Internal server error" })),
        )
    })?;

    Ok(Json(checkpoints))
}

// =============================================================================
// TOOL & MCP ROUTING HANDLERS (/api/v1/tools/*)
// =============================================================================

/// List all available MCP tools
async fn list_tools(State(state): State<AppState>) -> Json<Vec<qubit_mcp::ToolDefinition>> {
    let tools = state.mcp.list_tools().await;
    Json(tools)
}

/// Execute an MCP tool call dynamically
async fn call_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<CallToolRequest>,
) -> Json<ToolResult> {
    let call_id = Uuid::new_v4().to_string();
    let call = ToolCall {
        id: call_id.clone(),
        name: name.clone(),
        arguments: payload.arguments,
    };

    // Broadcast tool invoked event
    let _ = state.tx.send(WebSocketEvent::ToolInvoked(call.clone()));

    let result = state.mcp.execute(&call).await;

    // Broadcast tool completion event
    let _ = state
        .tx
        .send(WebSocketEvent::ToolCompleted(result.clone()));

    Json(result)
}

// =============================================================================
// REAL-TIME WEBSOCKET HANDLERS (/ws)
// =============================================================================

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    // Spawn task to send server broadcast events to websocket client
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if socket.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });
}

fn parse_agent_status(status_str: &str) -> AgentStatus {
    match status_str.to_lowercase().as_str() {
        "planning" => AgentStatus::Planning,
        "executing" => AgentStatus::Executing,
        "waiting_for_input" => AgentStatus::WaitingForInput,
        "failed" => AgentStatus::Failed,
        "completed" => AgentStatus::Completed,
        _ => AgentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        let storage = Storage::open_in_memory().expect("in-memory db");
        let mcp = McpRegistry::new();
        let engine = Arc::new(AgentEngine::new(storage.clone(), mcp.clone()));
        let (tx, _rx) = broadcast::channel(100);
        AppState {
            storage,
            mcp,
            engine,
            tx,
            start_time: Instant::now(),
        }
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let state = create_test_state();
        let app = build_router(state);

        // Test root health endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test /api/health endpoint
        let api_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(api_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_and_list_agents() {
        let state = create_test_state();
        let app = build_router(state);

        let create_payload = serde_json::json!({
            "name": "AuditAgent",
            "role": "Code Reviewer"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/agents")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let list_resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/agents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_resp.status(), StatusCode::OK);
    }
}

