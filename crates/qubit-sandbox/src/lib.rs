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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxBackend {
    Native,
    Docker {
        image: String,
        memory_limit: String,
        network: bool,
    },
    Podman {
        image: String,
        memory_limit: String,
        network: bool,
    },
}

impl Default for SandboxBackend {
    fn default() -> Self {
        Self::Native
    }
}

pub fn detect_available_backend() -> SandboxBackend {
    if which_binary("docker") {
        SandboxBackend::Docker {
            image: "alpine:latest".to_string(),
            memory_limit: "128m".to_string(),
            network: false,
        }
    } else if which_binary("podman") {
        SandboxBackend::Podman {
            image: "alpine:latest".to_string(),
            memory_limit: "128m".to_string(),
            network: false,
        }
    } else {
        SandboxBackend::Native
    }
}

pub fn which_binary(bin: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        #[cfg(unix)]
        let separator = ':';
        #[cfg(windows)]
        let separator = ';';

        for dir in path_var.split(separator) {
            let p = Path::new(dir).join(bin);
            if p.is_file() {
                return true;
            }
            #[cfg(windows)]
            {
                let p_exe = Path::new(dir).join(format!("{}.exe", bin));
                if p_exe.is_file() {
                    return true;
                }
            }
        }
    }
    false
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

    pub async fn run_with_backend(
        backend: &SandboxBackend,
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        cfg: &SandboxConfig,
    ) -> Result<SandboxOutput, SandboxError> {
        match backend {
            SandboxBackend::Native => Self::run(cmd, args, cwd, cfg).await,
            SandboxBackend::Docker { image, memory_limit, network } => {
                let mut docker_args = vec!["run", "--rm", "-i"];
                if !network {
                    docker_args.push("--network");
                    docker_args.push("none");
                }
                docker_args.push("-m");
                docker_args.push(memory_limit.as_str());

                let cwd_str = cwd.to_string_lossy().to_string();
                let vol_spec = format!("{}:/workspace", cwd_str);
                docker_args.push("-v");
                docker_args.push(&vol_spec);
                docker_args.push("-w");
                docker_args.push("/workspace");

                docker_args.push(image.as_str());
                docker_args.push(cmd);
                for a in args {
                    docker_args.push(a);
                }

                Self::run("docker", &docker_args, cwd, cfg).await
            }
            SandboxBackend::Podman { image, memory_limit, network } => {
                let mut podman_args = vec!["run", "--rm", "-i"];
                if !network {
                    podman_args.push("--network");
                    podman_args.push("none");
                }
                podman_args.push("-m");
                podman_args.push(memory_limit.as_str());

                let cwd_str = cwd.to_string_lossy().to_string();
                let vol_spec = format!("{}:/workspace", cwd_str);
                podman_args.push("-v");
                podman_args.push(&vol_spec);
                podman_args.push("-w");
                podman_args.push("/workspace");

                podman_args.push(image.as_str());
                podman_args.push(cmd);
                for a in args {
                    podman_args.push(a);
                }

                Self::run("podman", &podman_args, cwd, cfg).await
            }
        }
    }
}
