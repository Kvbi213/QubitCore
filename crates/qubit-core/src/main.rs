use qubit_agents::AgentEngine;
use qubit_core::{build_provider_from_env, build_router, AppState};
use qubit_mcp::{ElicitationGate, RootGuard};
use qubit_storage::StorageHandle;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qubit_core=info".into()),
        )
        .init();

    let db_path = std::env::var("QUBIT_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data/qubit_data.db"));

    let storage = StorageHandle::open(&db_path)?;
    let provider = build_provider_from_env();
    let engine = Arc::new(AgentEngine::new(storage.clone(), provider));
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let roots = Arc::new(RwLock::new(RootGuard::new(vec![cwd])?));
    let elicitation = Arc::new(ElicitationGate::new());
    let (events, _) = broadcast::channel(128);

    let state = AppState {
        storage: storage.clone(),
        engine,
        roots,
        elicitation,
        events,
    };

    let addr = std::env::var("QUBIT_BIND").unwrap_or_else(|_| "127.0.0.1:4000".to_string());
    let listener = TcpListener::bind(&addr).await?;
    info!("QubitCore daemon listening on http://{addr}");

    let app = build_router(state);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            info!("Shutdown signal received, running WAL checkpoint...");
        })
        .await?;

    storage.wal_checkpoint().await?;
    Ok(())
}
