//! Qubit Model Context Protocol (MCP) Host & Client
//! Provides tool discovery, transport adapters (stdio, sse, ws), capability scoping,
//! and Zero-Trust MCP with `TaintedContext` security boundaries and Elicitation gating.

use async_trait::async_trait;
use qubit_protocol::{ToolCall, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Security policy violation: Tool '{0}' blocked by Zero-Trust policy (Context is Tainted)")]
    TaintedPolicyViolation(String),
    #[error("Elicitation required: Execution of '{0}' requires human authorization: {1}")]
    ElicitationRequired(String, String),
}

pub type Result<T> = std::result::Result<T, McpError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolRiskLevel {
    /// Safe, read-only tools that do not mutate host state (e.g. read_file, list_roots)
    SafeReadOnly,
    /// Tools that mutate state within an isolated sandbox (e.g. sandbox_execute, pyghidra_analyze)
    SandboxedMutation,
    /// Dangerous tools that affect the host machine (e.g. terminal execution, raw git diff/commit, network egress)
    HostDangerous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    #[serde(default = "default_safe_risk_level")]
    pub risk_level: ToolRiskLevel,
}

fn default_safe_risk_level() -> ToolRiskLevel {
    ToolRiskLevel::SafeReadOnly
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value>;
}

/// Execution security context for MCP tools.
/// Tracks taint status (e.g. when processing untrusted web content or external binaries)
/// and authorized elicitation tokens.
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// If true, the session has ingested untrusted external data (e.g. malicious binary, prompt-injection hazard)
    tainted: Arc<AtomicBool>,
    /// Authorized tool tokens via explicit MCP user elicitation
    authorized_tools: Arc<RwLock<HashSet<String>>>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new(false)
    }
}

impl SecurityContext {
    pub fn new(initially_tainted: bool) -> Self {
        Self {
            tainted: Arc::new(AtomicBool::new(initially_tainted)),
            authorized_tools: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check if context is marked as tainted
    pub fn is_tainted(&self) -> bool {
        self.tainted.load(Ordering::SeqCst)
    }

    /// Mark the context as tainted (e.g. after receiving untrusted inputs)
    pub fn set_tainted(&self, tainted: bool) {
        self.tainted.store(tainted, Ordering::SeqCst);
    }

    /// Grant authorization for a specific tool call via MCP Elicitation
    pub async fn authorize_tool(&self, tool_name: &str) {
        let mut set = self.authorized_tools.write().await;
        set.insert(tool_name.to_string());
    }

    /// Revoke authorization for a tool
    pub async fn revoke_tool(&self, tool_name: &str) {
        let mut set = self.authorized_tools.write().await;
        set.remove(tool_name);
    }

    /// Check if a tool has an explicit human elicitation grant
    pub async fn is_tool_authorized(&self, tool_name: &str) -> bool {
        let set = self.authorized_tools.read().await;
        set.contains(tool_name)
    }
}

#[derive(Default, Clone)]
pub struct McpRegistry {
    tools: Arc<RwLock<HashMap<String, (ToolDefinition, Arc<dyn ToolHandler>)>>>,
    security: SecurityContext,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            security: SecurityContext::default(),
        }
    }

    pub fn with_security_context(security: SecurityContext) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            security,
        }
    }

    pub fn security(&self) -> &SecurityContext {
        &self.security
    }

    /// Register a local or bridged MCP tool
    pub async fn register<H: ToolHandler + 'static>(
        &self,
        definition: ToolDefinition,
        handler: H,
    ) {
        let mut tools = self.tools.write().await;
        tools.insert(definition.name.clone(), (definition, Arc::new(handler)));
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().map(|(def, _)| def.clone()).collect()
    }

    /// Check if execution of a tool is allowed under the current Zero-Trust security rules
    pub async fn check_permission(&self, tool_name: &str) -> Result<()> {
        let tools = self.tools.read().await;
        let (def, _) = tools
            .get(tool_name)
            .ok_or_else(|| McpError::NotFound(tool_name.to_string()))?;

        // Zero-Trust Capability Scoping rule:
        // When Context is Tainted, HostDangerous tools (terminal execution, diff, write)
        // are blocked unless explicitly authorized via MCP Elicitation.
        if self.security.is_tainted() && def.risk_level == ToolRiskLevel::HostDangerous {
            if !self.security.is_tool_authorized(tool_name).await {
                return Err(McpError::TaintedPolicyViolation(format!(
                    "Tool '{}' is classified as HostDangerous and blocked in TaintedContext. \
                     Must be approved via MCP Elicitation before proceeding.",
                    tool_name
                )));
            }
        }

        Ok(())
    }

    /// Execute a tool call against the registry under Zero-Trust scoping
    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        // 1. Permission and Capability Check
        if let Err(policy_err) = self.check_permission(&call.name).await {
            return ToolResult {
                call_id: call.id.clone(),
                success: false,
                output: None,
                error: Some(policy_err.to_string()),
            };
        }

        let tools = self.tools.read().await;
        match tools.get(&call.name) {
            Some((_, handler)) => match handler.execute(call.arguments.clone()).await {
                Ok(output) => ToolResult {
                    call_id: call.id.clone(),
                    success: true,
                    output: Some(output),
                    error: None,
                },
                Err(e) => ToolResult {
                    call_id: call.id.clone(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                },
            },
            None => ToolResult {
                call_id: call.id.clone(),
                success: false,
                output: None,
                error: Some(format!("Tool '{}' not registered", call.name)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoHandler;
    #[async_trait]
    impl ToolHandler for EchoHandler {
        async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value> {
            Ok(args)
        }
    }

    #[tokio::test]
    async fn test_safe_tool_in_tainted_context() {
        let registry = McpRegistry::new();
        registry.security().set_tainted(true);

        registry
            .register(
                ToolDefinition {
                    name: "read_file".into(),
                    description: "Read file safely".into(),
                    input_schema: serde_json::json!({}),
                    risk_level: ToolRiskLevel::SafeReadOnly,
                },
                EchoHandler,
            )
            .await;

        let call = ToolCall {
            id: "call-1".into(),
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "qubit.env"}),
        };

        let result = registry.execute(&call).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_host_dangerous_blocked_in_tainted_context_without_elicitation() {
        let registry = McpRegistry::new();
        registry.security().set_tainted(true);

        registry
            .register(
                ToolDefinition {
                    name: "run_command".into(),
                    description: "Execute host terminal command".into(),
                    input_schema: serde_json::json!({}),
                    risk_level: ToolRiskLevel::HostDangerous,
                },
                EchoHandler,
            )
            .await;

        let call = ToolCall {
            id: "call-2".into(),
            name: "run_command".into(),
            arguments: serde_json::json!({"cmd": "rm -rf /"}),
        };

        // Execution must fail with TaintedPolicyViolation
        let result = registry.execute(&call).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("blocked in TaintedContext"));

        // Grant explicit elicitation authorization
        registry.security().authorize_tool("run_command").await;
        let approved_result = registry.execute(&call).await;
        assert!(approved_result.success);
    }
}
