//! Axum Web & WebSocket Server for QubitCore

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use qubit_agents::AgentEngine;
use qubit_mcp::McpRegistry;
use qubit_protocol::WebSocketEvent;
use qubit_storage::Storage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    pub mcp: McpRegistry,
    pub engine: Arc<AgentEngine>,
    pub tx: broadcast::Sender<WebSocketEvent>,
}

pub async fn start_server(addr: SocketAddr, db_path: &str) -> anyhow::Result<()> {
    let storage = Storage::open(db_path)?;
    let mcp = McpRegistry::new();
    let engine = Arc::new(AgentEngine::new(storage.clone(), mcp.clone()));
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState {
        storage,
        mcp,
        engine,
        tx,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(ws_handler))
        .route("/api/v1/tools", get(list_tools))
        .layer(CorsLayer::permissive())
        .with_state(state);

    info!("Starting QubitCore daemon on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_tools(State(state): State<AppState>) -> Json<serde_json::Value> {
    let tools = state.mcp.list_tools().await;
    Json(serde_json::to_value(tools).unwrap_or_default())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

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
