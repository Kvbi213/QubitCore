//! Qubit Core Daemon & Hub Library
//! Orchestrates storage, agent dispatch, and Axum HTTP/WebSocket communication.

pub mod server;

pub use server::start_server;
