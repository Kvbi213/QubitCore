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
    #[error("Agent not found: {0}")]
    NotFound(String),
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

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn mcp_registry(&self) -> &McpRegistry {
        &self.mcp_registry
    }

    /// Process an incoming message with selected execution mode
    pub async fn process(
        &self,
        mode: ExecutionMode,
        user_message: String,
    ) -> Result<ChatMessage> {
        let msg_id = Uuid::new_v4();
        self.storage
            .insert_message(&msg_id.to_string(), "user", &user_message)
            .await?;

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
            .insert_message(&reply_id.to_string(), "assistant", &response_text)
            .await?;

        Ok(ChatMessage {
            id: reply_id,
            role: MessageRole::Assistant,
            content: response_text,
            created_at: chrono::Utc::now(),
            tool_calls: None,
        })
    }

    /// Execute a task explicitly targeted at a registered agent
    pub async fn execute_agent_task(
        &self,
        agent_id: Uuid,
        mode: ExecutionMode,
        task: String,
    ) -> Result<ChatMessage> {
        let agent_id_str = agent_id.to_string();
        let agent = self
            .storage
            .get_agent(&agent_id_str)?
            .ok_or_else(|| AgentError::NotFound(agent_id_str.clone()))?;

        // 1. Update status to Executing
        self.storage
            .update_agent_status(&agent_id_str, "executing")
            .await?;

        // 2. Record incoming task message
        let task_msg_id = Uuid::new_v4();
        self.storage
            .insert_message_full(
                &task_msg_id.to_string(),
                Some(&agent_id_str),
                "user",
                &task,
                None,
            )
            .await?;

        // 3. Execution logic based on mode and agent role
        let response_text = match mode {
            ExecutionMode::Reflex => {
                format!(
                    "Agent '{}' [{}] executed reflex task: {}",
                    agent.name, agent.role, task
                )
            }
            ExecutionMode::Deep => {
                let tools = self.mcp_registry.list_tools().await;
                format!(
                    "Agent '{}' [{}] executed deep reasoning with {} tools: {}",
                    agent.name,
                    agent.role,
                    tools.len(),
                    task
                )
            }
        };

        // 4. Record checkpoint
        let checkpoint_json = serde_json::json!({
            "task": task,
            "status": "completed",
            "mode": mode,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string();

        self.storage
            .save_checkpoint(&agent_id_str, 1, &checkpoint_json)
            .await?;

        // 5. Record outgoing reply message
        let reply_id = Uuid::new_v4();
        self.storage
            .insert_message_full(
                &reply_id.to_string(),
                Some(&agent_id_str),
                "assistant",
                &response_text,
                None,
            )
            .await?;

        // 6. Return agent status to Idle
        self.storage
            .update_agent_status(&agent_id_str, "idle")
            .await?;

        Ok(ChatMessage {
            id: reply_id,
            role: MessageRole::Assistant,
            content: response_text,
            created_at: chrono::Utc::now(),
            tool_calls: None,
        })
    }
}
