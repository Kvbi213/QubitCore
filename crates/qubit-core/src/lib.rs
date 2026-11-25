use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use qubit_agents::{AgentEngine, FallbackProviderChain, HttpApiProvider, ModelProvider, OfflineEchoProvider};
use qubit_mcp::{ElicitationGate, RootGuard};
use qubit_protocol::{
    ElicitationStatus, ExecuteTaskRequest, ExecuteTaskResponse, HealthResponse, WebSocketEvent,
};
use qubit_sandbox::current_process_rss_mb;
use qubit_storage::StorageHandle;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub storage: StorageHandle,
    pub engine: Arc<AgentEngine>,
    pub roots: Arc<RwLock<RootGuard>>,
    pub elicitation: Arc<ElicitationGate>,
    pub events: broadcast::Sender<WebSocketEvent>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRootRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterRootResponse {
    pub registered: String,
}

#[derive(Debug, Deserialize)]
pub struct ResolveElicitationRequest {
    pub decision: ElicitationStatus,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/sessions", post(create_session_handler))
        .route("/api/roots", post(register_root_handler))
        .route("/api/tasks", post(execute_task_handler))
        .route("/api/elicitation/{id}", post(resolve_elicitation_handler))
        .route("/ws", get(ws_handler))
        .with_state(state)
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let wal_ok = state.storage.wal_checkpoint().await.is_ok();
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        rss_mb: current_process_rss_mb(),
        wal_ok,
    })
}

async fn create_session_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<CreateSessionResponse>), (StatusCode, String)> {
    let title = req.title.unwrap_or_else(|| "default-session".to_string());
    let id = state
        .storage
        .create_session(&title)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(CreateSessionResponse { id })))
}

async fn register_root_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRootRequest>,
) -> Result<Json<RegisterRootResponse>, (StatusCode, String)> {
    let path = PathBuf::from(&req.path);
    let abs = state
        .roots
        .write()
        .await
        .add_root(&path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(RegisterRootResponse {
        registered: abs.display().to_string(),
    }))
}

async fn execute_task_handler(
    State(state): State<AppState>,
    Json(req): Json<ExecuteTaskRequest>,
) -> Result<Json<ExecuteTaskResponse>, (StatusCode, String)> {
    if req.prompt.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "prompt cannot be empty".to_string()));
    }

    let session_id = match req.session_id {
        Some(sid) => sid,
        None => state
            .storage
            .create_session("auto-session")
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };

    let task_id = Uuid::new_v4();
    let (mode, reply) = state
        .engine
        .execute(session_id, &req.prompt, req.mode, req.tainted)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state.events.send(WebSocketEvent::TaskCompleted {
        session_id,
        task_id,
        reply: reply.clone(),
    });

    Ok(Json(ExecuteTaskResponse {
        session_id,
        task_id,
        mode,
        reply,
    }))
}

async fn resolve_elicitation_handler(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(req): Json<ResolveElicitationRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .elicitation
        .resolve(id, req.decision)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.events.subscribe();
    while let Ok(ev) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&ev) {
            if socket.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    }
}

pub fn build_provider_from_env() -> Arc<FallbackProviderChain> {
    let mut providers: Vec<Arc<dyn ModelProvider>> = Vec::new();
    if let (Ok(url), Ok(key)) = (std::env::var("QUBIT_API_URL"), std::env::var("QUBIT_API_KEY")) {
        let model = std::env::var("QUBIT_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        providers.push(Arc::new(HttpApiProvider::new(url, key, model)));
    }
    providers.push(Arc::new(OfflineEchoProvider));
    Arc::new(FallbackProviderChain::new(providers))
}
