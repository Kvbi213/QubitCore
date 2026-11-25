use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("failed to spawn process: {0}")]
    Spawn(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct SandboxConfig {
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 10_000,
            max_output_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SandboxOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureLevel {
    Normal,
    Throttle,
    Critical,
}

pub fn classify_memory_pressure(anon_bytes: u64, limit_bytes: u64) -> BackpressureLevel {
    if limit_bytes == 0 {
        return BackpressureLevel::Normal;
    }
    let pct = (anon_bytes as f64 / limit_bytes as f64) * 100.0;
    if pct >= 85.0 {
        BackpressureLevel::Critical
    } else if pct >= 80.0 {
        BackpressureLevel::Throttle
    } else {
        BackpressureLevel::Normal
    }
}

pub fn current_process_rss_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(rss_pages_str) = statm.split_whitespace().nth(1) {
                if let Ok(rss_pages) = rss_pages_str.parse::<u64>() {
                    let bytes = rss_pages * 4096;
                    return (bytes as f64) / (1024.0 * 1024.0);
                }
            }
        }
    }
    0.0
}

pub struct SandboxRunner;

impl SandboxRunner {
    pub async fn run(
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        cfg: &SandboxConfig,
    ) -> Result<SandboxOutput, SandboxError> {
        let mut child = Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let mut stdout_pipe = child.stdout.take();
        let mut stderr_pipe = child.stderr.take();
        let max_bytes = cfg.max_output_bytes;

        let read_stdout = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut truncated = false;
            if let Some(ref mut out) = stdout_pipe {
                let mut limited = out.take((max_bytes + 1) as u64);
                let _ = limited.read_to_end(&mut buf).await;
                if buf.len() > max_bytes {
                    buf.truncate(max_bytes);
                    truncated = true;
                }
            }
            (String::from_utf8_lossy(&buf).to_string(), truncated)
        });

        let read_stderr = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut truncated = false;
            if let Some(ref mut err) = stderr_pipe {
                let mut limited = err.take((max_bytes + 1) as u64);
                let _ = limited.read_to_end(&mut buf).await;
                if buf.len() > max_bytes {
                    buf.truncate(max_bytes);
                    truncated = true;
                }
            }
            (String::from_utf8_lossy(&buf).to_string(), truncated)
        });

        let wait_res = tokio::time::timeout(Duration::from_millis(cfg.timeout_ms), child.wait()).await;

        let (exit_code, timed_out) = match wait_res {
            Ok(Ok(status)) => (status.code().unwrap_or(-1), false),
            Ok(Err(e)) => return Err(SandboxError::Spawn(e)),
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                (-1, true)
            }
        };

        let (stdout, out_trunc) = read_stdout.await.unwrap_or_default();
        let (stderr, err_trunc) = read_stderr.await.unwrap_or_default();

        Ok(SandboxOutput {
            exit_code,
            stdout,
            stderr,
            timed_out,
            truncated: out_trunc || err_trunc,
        })
    }
}
