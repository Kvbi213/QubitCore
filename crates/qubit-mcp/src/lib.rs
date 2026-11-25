use qubit_protocol::{ElicitationRequest, ElicitationStatus};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("path escapes configured MCP roots")]
    OutsideRoot,
    #[error("mutating tool blocked in TaintedContext")]
    TaintedBlocked,
    #[error("elicitation denied by operator")]
    ElicitationDenied,
    #[error("elicitation pending operator approval: {0}")]
    ElicitationPending(Uuid),
    #[error("elicitation request not found: {0}")]
    ElicitationNotFound(Uuid),
    #[error("invalid MCP root configuration: {0}")]
    InvalidRoot(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolRisk {
    ReadOnly,
    Mutating,
}

#[derive(Debug, Clone)]
pub struct RootGuard {
    roots: Vec<PathBuf>,
}

impl RootGuard {
    pub fn new(roots: Vec<PathBuf>) -> Result<Self, McpError> {
        let mut canonical = Vec::with_capacity(roots.len());
        for root in roots {
            let abs = std::fs::canonicalize(&root)
                .map_err(|e| McpError::InvalidRoot(format!("{}: {e}", root.display())))?;
            canonical.push(abs);
        }
        Ok(Self { roots: canonical })
    }

    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    pub fn add_root(&mut self, root: &Path) -> Result<PathBuf, McpError> {
        let abs = std::fs::canonicalize(root)
            .map_err(|e| McpError::InvalidRoot(format!("{}: {e}", root.display())))?;
        if !self.roots.contains(&abs) {
            self.roots.push(abs.clone());
        }
        Ok(abs)
    }

    pub fn resolve_safe(&self, candidate: &Path) -> Result<PathBuf, McpError> {
        if self.roots.is_empty() {
            return Err(McpError::OutsideRoot);
        }

        if candidate
            .components()
            .any(|c| matches!(c, Component::ParentDir))
            && !candidate.is_absolute()
        {
            return Err(McpError::OutsideRoot);
        }

        let resolved = if candidate.exists() {
            std::fs::canonicalize(candidate).map_err(|_| McpError::OutsideRoot)?
        } else {
            let mut ancestor = candidate.to_path_buf();
            let mut tail = Vec::new();
            while !ancestor.exists() {
                if let Some(name) = ancestor.file_name() {
                    tail.push(name.to_os_string());
                } else {
                    return Err(McpError::OutsideRoot);
                }
                if !ancestor.pop() {
                    return Err(McpError::OutsideRoot);
                }
            }
            let mut base = std::fs::canonicalize(&ancestor).map_err(|_| McpError::OutsideRoot)?;
            for seg in tail.into_iter().rev() {
                base.push(seg);
            }
            base
        };

        if self.roots.iter().any(|root| resolved.starts_with(root)) {
            Ok(resolved)
        } else {
            Err(McpError::OutsideRoot)
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SecurityContext {
    pub tainted: bool,
}

impl SecurityContext {
    pub fn authorize_tool(&self, _tool_name: &str, risk: ToolRisk) -> Result<(), McpError> {
        if self.tainted && risk == ToolRisk::Mutating {
            return Err(McpError::TaintedBlocked);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ElicitationGate {
    items: Arc<RwLock<HashMap<Uuid, ElicitationRequest>>>,
}

impl ElicitationGate {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn submit(
        &self,
        session_id: Uuid,
        tool_name: impl Into<String>,
        args_preview: impl Into<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let req = ElicitationRequest {
            id,
            session_id,
            tool_name: tool_name.into(),
            args_preview: args_preview.into(),
            status: ElicitationStatus::Pending,
        };
        self.items.write().await.insert(id, req);
        id
    }

    pub async fn resolve(&self, id: Uuid, decision: ElicitationStatus) -> Result<(), McpError> {
        let mut guard = self.items.write().await;
        let item = guard.get_mut(&id).ok_or(McpError::ElicitationNotFound(id))?;
        item.status = decision;
        Ok(())
    }

    pub async fn status(&self, id: Uuid) -> Option<ElicitationStatus> {
        self.items.read().await.get(&id).map(|r| r.status)
    }
}
