//! Qubit Core Daemon & Hub Library
//! Orchestrates storage, agent dispatch, and Axum HTTP/WebSocket communication.

pub mod server;

pub use server::{build_router, start_server, AppState};
