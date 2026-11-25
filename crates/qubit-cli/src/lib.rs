use qubit_protocol::{ExecuteTaskRequest, ExecuteTaskResponse, ExecutionMode, HealthResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("QubitCore daemon unreachable at {0} (start it with `cargo run -p qubit-core`)")]
    DaemonUnavailable(String),
    #[error("HTTP error {0}: {1}")]
    Http(u16, String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub enum CliCommand {
    Status,
    MountCurrentDir,
    Chat { prompt: String, deep: bool },
}

pub async fn run_cli(cmd: CliCommand, base_url: &str) -> Result<String, CliError> {
    let client = reqwest::Client::new();
    let base = base_url.trim_end_matches('/');

    match cmd {
        CliCommand::Status => {
            let url = format!("{base}/health");
            let res = client
                .get(&url)
                .send()
                .await
                .map_err(|e| CliError::DaemonUnavailable(format!("{url}: {e}")))?;
            if !res.status().is_success() {
                let code = res.status().as_u16();
                return Err(CliError::Http(code, res.text().await.unwrap_or_default()));
            }
            let health: HealthResponse = res
                .json()
                .await
                .map_err(|e| CliError::DaemonUnavailable(e.to_string()))?;
            Ok(format!(
                "QubitCore {} | status={} | rss={:.2} MB | wal_ok={}",
                health.version, health.status, health.rss_mb, health.wal_ok
            ))
        }
        CliCommand::MountCurrentDir => {
            let cwd = std::env::current_dir()?;
            let url = format!("{base}/api/roots");
            let res = client
                .post(&url)
                .json(&json!({ "path": cwd.display().to_string() }))
                .send()
                .await
                .map_err(|e| CliError::DaemonUnavailable(format!("{url}: {e}")))?;
            if !res.status().is_success() {
                let code = res.status().as_u16();
                return Err(CliError::Http(code, res.text().await.unwrap_or_default()));
            }
            Ok(format!("Mounted MCP Root: {}", cwd.display()))
        }
        CliCommand::Chat { prompt, deep } => {
            let url = format!("{base}/api/tasks");
            let req = ExecuteTaskRequest {
                session_id: None,
                prompt,
                mode: if deep { Some(ExecutionMode::Deep) } else { None },
                tainted: false,
            };
            let res = client
                .post(&url)
                .json(&req)
                .send()
                .await
                .map_err(|e| CliError::DaemonUnavailable(format!("{url}: {e}")))?;
            if !res.status().is_success() {
                let code = res.status().as_u16();
                return Err(CliError::Http(code, res.text().await.unwrap_or_default()));
            }
            let parsed: ExecuteTaskResponse = res
                .json()
                .await
                .map_err(|e| CliError::DaemonUnavailable(e.to_string()))?;
            Ok(format!("[{:?}] {}", parsed.mode, parsed.reply))
        }
    }
}
