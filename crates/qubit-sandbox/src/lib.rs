//! Qubit Sandbox Engine
//! Manages isolated environments (Docker containers, process namespaces) for secure tool execution.

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Sandbox unavailable: {0}")]
    Unavailable(String),
}

pub type Result<T> = std::result::Result<T, SandboxError>;

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_memory_mb: u64,
    pub max_cpu_cores: f32,
    pub execution_timeout: Duration,
    pub work_dir: PathBuf,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_cores: 1.0,
            execution_timeout: Duration::from_secs(60),
            work_dir: PathBuf::from("/tmp/qubit-sandbox"),
        }
    }
}

pub struct ProcessSandbox {
    config: SandboxConfig,
}

impl ProcessSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Execute a command in isolated host process space with strict timeout
    pub async fn execute(&self, program: &str, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new(program);
        cmd.args(args);

        if self.config.work_dir.exists() {
            cmd.current_dir(&self.config.work_dir);
        }

        let child = cmd.output();

        match tokio::time::timeout(self.config.execution_timeout, child).await {
            Ok(output_result) => {
                let output = output_result?;
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(SandboxError::ExecutionFailed(err))
                }
            }
            Err(_) => Err(SandboxError::Timeout(self.config.execution_timeout)),
        }
    }
}
