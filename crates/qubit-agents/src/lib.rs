pub mod compressor;
pub mod provider;

pub use compressor::ContextCompressor;
pub use provider::{
    CliBridgeProvider, FallbackProviderChain, HttpApiProvider, ModelProvider, OfflineEchoProvider,
    ProviderError,
};

use qubit_mcp::{McpError, SecurityContext};
use qubit_protocol::{ChatMessage, ExecutionMode, MessageRole};
use qubit_storage::{StorageError, StorageHandle};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("security error: {0}")]
    Security(#[from] McpError),
}

pub struct ModeRouter;

impl ModeRouter {
    pub fn classify(prompt: &str, requested: Option<ExecutionMode>) -> ExecutionMode {
        if let Some(mode) = requested {
            return mode;
        }
        let lower = prompt.to_lowercase();
        if prompt.len() > 240
            || lower.contains("plan")
            || lower.contains("architekt")
            || lower.contains("refactor")
        {
            ExecutionMode::Deep
        } else {
            ExecutionMode::Reflex
        }
    }
}

pub struct AgentEngine {
    storage: StorageHandle,
    provider: Arc<FallbackProviderChain>,
    max_context_chars: usize,
}

impl AgentEngine {
    pub fn new(storage: StorageHandle, provider: Arc<FallbackProviderChain>) -> Self {
        Self {
            storage,
            provider,
            max_context_chars: 32_000,
        }
    }

    pub async fn execute(
        &self,
        session_id: Uuid,
        prompt: &str,
        requested_mode: Option<ExecutionMode>,
        tainted: bool,
    ) -> Result<(ExecutionMode, String), AgentError> {
        let _sec = SecurityContext { tainted };
        let mode = ModeRouter::classify(prompt, requested_mode);

        let user_msg = ChatMessage::new(session_id, MessageRole::User, prompt, tainted);
        self.storage.append_message(user_msg).await?;

        let history = self.storage.list_messages(session_id).await?;
        let compressed = ContextCompressor::compress(history, self.max_context_chars);

        let reply = self.provider.complete(&compressed, mode).await?;
        let assistant_msg = ChatMessage::new(session_id, MessageRole::Assistant, &reply, tainted);
        self.storage.append_message(assistant_msg).await?;

        Ok((mode, reply))
    }
}
