//! Qubit Model Context Protocol (MCP) Host & Client
//! Provides tool discovery, transport adapters (stdio, sse, ws), and zero-overhead registry.

use async_trait::async_trait;
use qubit_protocol::{ToolCall, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
}

pub type Result<T> = std::result::Result<T, McpError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value>;
}

#[derive(Default, Clone)]
pub struct McpRegistry {
    tools: Arc<RwLock<HashMap<String, (ToolDefinition, Arc<dyn ToolHandler>)>>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
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

    /// Execute a tool call against the registry
    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
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
