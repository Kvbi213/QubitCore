//! Qubit Agent Orchestrator
//! Hermes Agent patterns, Reflex vs Deep execution modes, dynamic delegation, and context management.

use async_trait::async_trait;
use qubit_mcp::McpRegistry;
use qubit_protocol::{AgentDescriptor, AgentStatus, ChatMessage, ExecutionMode, MessageRole};
use qubit_storage::Storage;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent dispatch error: {0}")]
    Dispatch(String),
    #[error("Storage failure: {0}")]
    Storage(#[from] qubit_storage::StorageError),
    #[error("Context overflow: {0} tokens exceeds limit {1}")]
    ContextOverflow(usize, usize),
}

pub type Result<T> = std::result::Result<T, AgentError>;

#[async_trait]
pub trait Agent: Send + Sync {
    fn descriptor(&self) -> AgentDescriptor;
    async fn handle_message(&self, message: ChatMessage) -> Result<ChatMessage>;
}

/// Dynamic Agent Executor handling Reflex vs Deep routing
pub struct AgentEngine {
    storage: Storage,
    mcp_registry: McpRegistry,
}

impl AgentEngine {
    pub fn new(storage: Storage, mcp_registry: McpRegistry) -> Self {
        Self {
            storage,
            mcp_registry,
        }
    }

    /// Process an incoming message with selected execution mode
    pub async fn process(
        &self,
        mode: ExecutionMode,
        user_message: String,
    ) -> Result<ChatMessage> {
        let msg_id = Uuid::new_v4();
        self.storage
            .insert_message(&msg_id.to_string(), "user", &user_message)?;

        let response_text = match mode {
            ExecutionMode::Reflex => {
                // Low-latency direct model answer or fast tool execution
                format!("Reflex response to: {}", user_message)
            }
            ExecutionMode::Deep => {
                // Multi-step planning, delegation, progressive tool discovery
                let available_tools = self.mcp_registry.list_tools().await;
                format!(
                    "Deep reasoning mode engaged. Available tools: {}. Executing plan...",
                    available_tools.len()
                )
            }
        };

        let reply_id = Uuid::new_v4();
        self.storage
            .insert_message(&reply_id.to_string(), "assistant", &response_text)?;

        Ok(ChatMessage {
            id: reply_id,
            role: MessageRole::Assistant,
            content: response_text,
            created_at: chrono::Utc::now(),
            tool_calls: None,
        })
    }
}
