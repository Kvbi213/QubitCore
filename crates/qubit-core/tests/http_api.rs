use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use qubit_agents::{AgentEngine, FallbackProviderChain, OfflineEchoProvider};
use qubit_core::{build_router, AppState};
use qubit_mcp::{ElicitationGate, RootGuard};
use qubit_storage::StorageHandle;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower::ServiceExt;

async fn test_router() -> (Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("test_qubit.db");
    let storage = StorageHandle::open(&db_path).expect("open storage");
    let provider = Arc::new(FallbackProviderChain::new(vec![Arc::new(
        OfflineEchoProvider,
    )]));
    let engine = Arc::new(AgentEngine::new(storage.clone(), provider));
    let roots = Arc::new(RwLock::new(RootGuard::new(vec![]).unwrap()));
    let elicitation = Arc::new(ElicitationGate::new());
    let (events, _) = broadcast::channel(64);
    let router = build_router(AppState {
        storage,
        engine,
        roots,
        elicitation,
        events,
    });
    (router, dir)
}

#[tokio::test]
async fn health_and_empty_prompt_validation_work() {
    let (app, _dir) = test_router().await;

    let res = app
        .clone()
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bad_task = app
        .oneshot(
            Request::post("/api/tasks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"prompt":"   ","tainted":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad_task.status(), StatusCode::BAD_REQUEST);
}
