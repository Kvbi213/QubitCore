use crate::{McpError, RootGuard, SecurityContext, ToolRisk};
use qubit_sandbox::{detect_available_backend, SandboxBackend, SandboxConfig, SandboxOutput, SandboxRunner};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MAX_READ_BYTES: usize = 64 * 1024; // 64 KB

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

pub struct StandardTools;

impl StandardTools {
    pub fn read_file(
        guard: &RootGuard,
        path: &Path,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<String, McpError> {
        let safe_path = guard.resolve_safe(path)?;
        let data = fs::read(&safe_path).map_err(|e| McpError::InvalidRoot(e.to_string()))?;

        let off = offset.unwrap_or(0);
        if off >= data.len() {
            return Ok(String::new());
        }

        let max_len = limit.unwrap_or(DEFAULT_MAX_READ_BYTES).min(DEFAULT_MAX_READ_BYTES);
        let end = (off + max_len).min(data.len());
        let slice = &data[off..end];

        Ok(String::from_utf8_lossy(slice).to_string())
    }

    pub fn write_file(
        guard: &RootGuard,
        security: &SecurityContext,
        path: &Path,
        content: &str,
        overwrite: bool,
    ) -> Result<PathBuf, McpError> {
        security.authorize_tool("write_file", ToolRisk::Mutating)?;
        let safe_path = guard.resolve_safe(path)?;

        if safe_path.exists() && !overwrite {
            return Err(McpError::InvalidRoot(format!(
                "file already exists and overwrite is false: {}",
                safe_path.display()
            )));
        }

        if let Some(parent) = safe_path.parent() {
            fs::create_dir_all(parent).map_err(|e| McpError::InvalidRoot(e.to_string()))?;
        }

        fs::write(&safe_path, content).map_err(|e| McpError::InvalidRoot(e.to_string()))?;
        Ok(safe_path)
    }

    pub fn edit_file(
        guard: &RootGuard,
        security: &SecurityContext,
        path: &Path,
        target: &str,
        replacement: &str,
    ) -> Result<(), McpError> {
        security.authorize_tool("edit_file", ToolRisk::Mutating)?;
        let safe_path = guard.resolve_safe(path)?;

        let current = fs::read_to_string(&safe_path).map_err(|e| McpError::InvalidRoot(e.to_string()))?;
        if !current.contains(target) {
            return Err(McpError::InvalidRoot(format!(
                "target string not found in {}",
                safe_path.display()
            )));
        }

        let updated = current.replace(target, replacement);
        fs::write(&safe_path, updated).map_err(|e| McpError::InvalidRoot(e.to_string()))?;
        Ok(())
    }

    pub fn list_dir(
        guard: &RootGuard,
        path: &Path,
        max_entries: Option<usize>,
    ) -> Result<Vec<FileEntry>, McpError> {
        let safe_path = guard.resolve_safe(path)?;
        let read_dir = fs::read_dir(&safe_path).map_err(|e| McpError::InvalidRoot(e.to_string()))?;

        let limit = max_entries.unwrap_or(200);
        let mut entries = Vec::new();

        for entry in read_dir.flatten() {
            if entries.len() >= limit {
                break;
            }
            let meta = entry.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let name = entry.file_name().to_string_lossy().to_string();
            let path_str = entry.path().to_string_lossy().to_string();

            entries.push(FileEntry {
                name,
                path: path_str,
                is_dir,
                size,
            });
        }

        Ok(entries)
    }

    pub async fn run_command(
        guard: &RootGuard,
        security: &SecurityContext,
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        backend_override: Option<&SandboxBackend>,
        cfg: &SandboxConfig,
    ) -> Result<SandboxOutput, McpError> {
        security.authorize_tool("run_command", ToolRisk::Mutating)?;
        let safe_cwd = guard.resolve_safe(cwd)?;

        let detected = detect_available_backend();
        let backend = backend_override.unwrap_or(&detected);

        SandboxRunner::run_with_backend(backend, cmd, args, &safe_cwd, cfg)
            .await
            .map_err(|e| McpError::InvalidRoot(e.to_string()))
    }
}
