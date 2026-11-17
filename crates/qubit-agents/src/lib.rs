//! Qubit Agent Orchestrator & Context Management Engine
//! Implements Hermes Agent patterns:
//! - 4-Phase Context Compression Algorithm:
//!     * Phase 1: Context Window 50% Threshold Monitor (`CONTEXT_THRESHOLD = 0.50`)
//!     * Phase 2: Tail Protection (`PROTECT_LAST_N = 20`)
//!     * Phase 3: Slice Summarization with 20% Token Budget (`SUMMARY_RATIO = 0.20`, min 2,000, max 12,000)
//!     * Phase 4: Assembly (Head + Summary + Tail)
//! - Deterministic Fallback Ladder (`DETERMINISTIC_SUMMARY_ROUTE`):
//!     * Fallback on HTTP 529 / Timeouts without stalling execution loop.

use async_trait::async_trait;
use qubit_mcp::McpRegistry;
use qubit_protocol::{AgentDescriptor, AgentStatus, ChatMessage, ExecutionMode, MessageRole};
use qubit_storage::Storage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, warn};
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
    #[error("Summarization failure: {0}")]
    Summarization(String),
}

pub type Result<T> = std::result::Result<T, AgentError>;

// =============================================================================
// 4-PHASE CONTEXT COMPRESSION CONSTANTS & STRUCTURES
// =============================================================================

/// Phase 1: Context window monitoring threshold (50% of model window)
pub const CONTEXT_THRESHOLD: f32 = 0.50;

/// Phase 2: Protection of the most recent messages (tail preservation)
pub const PROTECT_LAST_N: usize = 20;

/// Phase 3: Token budget allocated to the generated summary (20% of context window)
pub const SUMMARY_RATIO: f32 = 0.20;

/// Minimum and maximum token boundaries for summary budget
pub const MIN_SUMMARY_TOKENS: usize = 2_000;
pub const MAX_SUMMARY_TOKENS: usize = 12_000;

/// Default baseline context window in tokens (e.g. 128k context)
pub const DEFAULT_MAX_CONTEXT_TOKENS: usize = 128_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummarizerStatus {
    Success,
    FallbackLocalTruncation,
}

#[derive(Debug, Clone)]
pub struct ContextCompressionResult {
    pub compressed: bool,
    pub original_message_count: usize,
    pub resulting_message_count: usize,
    pub estimated_tokens_before: usize,
    pub estimated_tokens_after: usize,
    pub summarizer_status: SummarizerStatus,
}

/// Estimates tokens for a given string (standard heuristic: ~4 characters per token)
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

/// Estimates total tokens for a slice of messages
pub fn estimate_messages_tokens(messages: &[ChatMessage]) -> usize {
    messages
        .iter()
        .map(|m| estimate_tokens(&m.content) + 4) // +4 token overhead per message structure
        .sum()
}

// =============================================================================
// DETERMINISTIC FALLBACK ROUTE FOR SUMMARIZATION
// =============================================================================

/// Summarizer trait enabling swappable LLM routes and deterministic local fallbacks
#[async_trait]
pub trait ContextSummarizer: Send + Sync {
    async fn summarize(
        &self,
        slice_to_compress: &[ChatMessage],
        max_tokens: usize,
    ) -> std::result::Result<String, String>;
}

/// Deterministic summary route:
/// Tries primary LLM summary; upon timeout or HTTP 529/rate-limit error,
/// falls back to deterministic local extractive condensation without blocking the agent loop.
pub struct DeterministicSummaryRoute {
    primary_endpoint: Option<String>,
}

impl DeterministicSummaryRoute {
    pub fn new(primary_endpoint: Option<String>) -> Self {
        Self { primary_endpoint }
    }

    /// Local deterministic extractive condensation (fallback)
    pub fn deterministic_local_fallback(slice: &[ChatMessage], max_tokens: usize) -> String {
        let mut summary_lines = Vec::new();
        summary_lines.push("[SYSTEM HERMES DETERMINISTIC CONDENSED HISTORY]".to_string());

        let max_chars = max_tokens * 4;
        let mut accumulated_chars = summary_lines[0].len();

        for (idx, msg) in slice.iter().enumerate() {
            let role_prefix = match msg.role {
                MessageRole::System => "System",
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
                MessageRole::Tool => "Tool",
            };

            // Extract first 150 chars or first paragraph
            let snippet = msg
                .content
                .lines()
                .next()
                .unwrap_or("")
                .trim();
            let truncated = if snippet.len() > 150 {
                &snippet[..150]
            } else {
                snippet
            };

            let entry = format!("- Step {}: {} -> {}", idx + 1, role_prefix, truncated);
            if accumulated_chars + entry.len() > max_chars {
                summary_lines.push(format!("- [... {} earlier events condensed]", slice.len() - idx));
                break;
            }

            accumulated_chars += entry.len() + 1;
            summary_lines.push(entry);
        }

        summary_lines.join("\n")
    }
}

#[async_trait]
impl ContextSummarizer for DeterministicSummaryRoute {
    async fn summarize(
        &self,
        slice_to_compress: &[ChatMessage],
        max_tokens: usize,
    ) -> std::result::Result<String, String> {
        // If no remote endpoint is configured, or if simulated remote call fails with 529/timeout:
        if let Some(endpoint) = &self.primary_endpoint {
            // Attempt remote LLM summarization with strict 5-second timeout
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|e| e.to_string())?;

            let body = serde_json::json!({
                "model": "groq/openai/gpt-oss-120b",
                "messages": [{
                    "role": "system",
                    "content": "You are Hermes Context Compressor. Produce a concise, dense factual summary."
                }, {
                    "role": "user",
                    "content": format!("Summarize the following {} interactions within {} tokens:\n{:?}",
                        slice_to_compress.len(), max_tokens, slice_to_compress)
                }]
            });

            match client.post(endpoint).json(&body).send().await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(json_resp) = resp.json::<serde_json::Value>().await {
                        if let Some(text) = json_resp["choices"][0]["message"]["content"].as_str() {
                            return Ok(text.to_string());
                        }
                    }
                }
                Ok(resp) if resp.status().as_u16() == 529 || resp.status().as_u16() == 429 => {
                    warn!(
                        "Summarization route returned HTTP {} (overloaded). Engaging DETERMINISTIC_SUMMARY_ROUTE fallback.",
                        resp.status()
                    );
                }
                Err(e) => {
                    warn!(
                        "Summarization endpoint connection failed/timed out: {}. Engaging DETERMINISTIC_SUMMARY_ROUTE fallback.",
                        e
                    );
                }
                _ => {}
            }
        }

        // Deterministic Fallback: local extractive distillation guaranteed to never fail or hang
        Ok(Self::deterministic_local_fallback(slice_to_compress, max_tokens))
    }
}

// =============================================================================
// HERMES 4-PHASE COMPRESSION ENGINE
// =============================================================================

pub struct HermesContextEngine {
    max_context_tokens: usize,
    summarizer: Arc<dyn ContextSummarizer>,
}

impl HermesContextEngine {
    pub fn new(max_context_tokens: usize, summarizer: Arc<dyn ContextSummarizer>) -> Self {
        Self {
            max_context_tokens,
            summarizer,
        }
    }

    pub fn with_default_summarizer(max_context_tokens: usize) -> Self {
        Self::new(
            max_context_tokens,
            Arc::new(DeterministicSummaryRoute::new(None)),
        )
    }

    /// Executes the 4-Phase Context Compression Algorithm on a conversation history:
    /// - Phase 1: Monitor threshold (50% of max_context_tokens)
    /// - Phase 2: Protection of Tail (last 20 messages) and Head (initial system/task message)
    /// - Phase 3: Slice Summarization with 20% Token Budget (min 2,000, max 12,000)
    /// - Phase 4: Assembly (Head + Summary + Tail)
    pub async fn compress_context(
        &self,
        messages: &mut Vec<ChatMessage>,
    ) -> Result<ContextCompressionResult> {
        let initial_count = messages.len();
        let initial_tokens = estimate_messages_tokens(messages);

        // Phase 1: Monitoring Threshold (50% of context limit)
        let threshold_tokens = (self.max_context_tokens as f32 * CONTEXT_THRESHOLD) as usize;
        if initial_tokens <= threshold_tokens || initial_count <= (PROTECT_LAST_N + 2) {
            return Ok(ContextCompressionResult {
                compressed: false,
                original_message_count: initial_count,
                resulting_message_count: initial_count,
                estimated_tokens_before: initial_tokens,
                estimated_tokens_after: initial_tokens,
                summarizer_status: SummarizerStatus::Success,
            });
        }

        info!(
            "Context tokens ({}) crossed 50% threshold ({}). Triggering Hermes 4-Phase Compression.",
            initial_tokens, threshold_tokens
        );

        // Phase 2: Tail Protection
        // Protect the last N messages (PROTECT_LAST_N) + preserve the Head (index 0)
        let head_count = 1;
        let split_idx = initial_count - PROTECT_LAST_N;
        if split_idx <= head_count {
            return Ok(ContextCompressionResult {
                compressed: false,
                original_message_count: initial_count,
                resulting_message_count: initial_count,
                estimated_tokens_before: initial_tokens,
                estimated_tokens_after: initial_tokens,
                summarizer_status: SummarizerStatus::Success,
            });
        }

        let head = messages[0].clone();
        let slice_to_compress = &messages[head_count..split_idx];
        let tail = messages[split_idx..].to_vec();

        // Phase 3: Calculate 20% Summary Budget (constrained between 2,000 and 12,000)
        let raw_budget = (self.max_context_tokens as f32 * SUMMARY_RATIO) as usize;
        let summary_token_budget = raw_budget.clamp(MIN_SUMMARY_TOKENS, MAX_SUMMARY_TOKENS);

        // Execute summarization with deterministic fallback
        let (summary_text, status) = match self
            .summarizer
            .summarize(slice_to_compress, summary_token_budget)
            .await
        {
            Ok(s) => (s, SummarizerStatus::Success),
            Err(err) => {
                warn!("Primary summarizer error: {}. Using deterministic local condensation.", err);
                (
                    DeterministicSummaryRoute::deterministic_local_fallback(
                        slice_to_compress,
                        summary_token_budget,
                    ),
                    SummarizerStatus::FallbackLocalTruncation,
                )
            }
        };

        // Phase 4: Assembly (Head + Summary Message + Tail)
        let summary_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: format!(
                "[HERMES CONTEXT SUMMARY - Condensed {} intermediate messages]:\n{}",
                slice_to_compress.len(),
                summary_text
            ),
            created_at: chrono::Utc::now(),
            tool_calls: None,
        };

        let mut assembled = Vec::with_capacity(1 + 1 + tail.len());
        assembled.push(head);
        assembled.push(summary_message);
        assembled.extend(tail);

        let final_tokens = estimate_messages_tokens(&assembled);
        let final_count = assembled.len();

        info!(
            "Hermes Compression complete: {} -> {} messages, ~{} -> ~{} tokens (Status: {:?})",
            initial_count, final_count, initial_tokens, final_tokens, status
        );

        *messages = assembled;

        Ok(ContextCompressionResult {
            compressed: true,
            original_message_count: initial_count,
            resulting_message_count: final_count,
            estimated_tokens_before: initial_tokens,
            estimated_tokens_after: final_tokens,
            summarizer_status: status,
        })
    }
}

// =============================================================================
// AGENT ENGINE
// =============================================================================

#[async_trait]
pub trait Agent: Send + Sync {
    fn descriptor(&self) -> AgentDescriptor;
    async fn handle_message(&self, message: ChatMessage) -> Result<ChatMessage>;
}

/// Dynamic Agent Executor handling Reflex vs Deep routing & Hermes Context Compression
pub struct AgentEngine {
    storage: Storage,
    mcp_registry: McpRegistry,
    context_engine: HermesContextEngine,
}

impl AgentEngine {
    pub fn new(storage: Storage, mcp_registry: McpRegistry) -> Self {
        Self {
            storage,
            mcp_registry,
            context_engine: HermesContextEngine::with_default_summarizer(DEFAULT_MAX_CONTEXT_TOKENS),
        }
    }

    pub fn with_context_engine(
        storage: Storage,
        mcp_registry: McpRegistry,
        context_engine: HermesContextEngine,
    ) -> Self {
        Self {
            storage,
            mcp_registry,
            context_engine,
        }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn mcp_registry(&self) -> &McpRegistry {
        &self.mcp_registry
    }

    pub fn context_engine(&self) -> &HermesContextEngine {
        &self.context_engine
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

    /// Execute a task explicitly targeted at a registered agent with context compression
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

        // 3. Retrieve conversation history for context maintenance
        let stored_messages = self.storage.get_messages(Some(&agent_id_str), 100)?;
        let mut chat_history: Vec<ChatMessage> = stored_messages
            .into_iter()
            .map(|m| {
                let role = match m.role.as_str() {
                    "system" => MessageRole::System,
                    "assistant" => MessageRole::Assistant,
                    "tool" => MessageRole::Tool,
                    _ => MessageRole::User,
                };
                ChatMessage {
                    id: Uuid::parse_str(&m.id).unwrap_or_else(|_| Uuid::new_v4()),
                    role,
                    content: m.content,
                    created_at: chrono::Utc::now(),
                    tool_calls: None,
                }
            })
            .collect();

        // 4. Run Hermes 4-Phase Context Compression
        let compression_res = self.context_engine.compress_context(&mut chat_history).await?;
        if compression_res.compressed {
            debug!(
                "Conversation history for agent {} compressed from {} to {} messages",
                agent.name, compression_res.original_message_count, compression_res.resulting_message_count
            );
        }

        // 5. Execution logic based on mode and agent role
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
                    "Agent '{}' [{}] executed deep reasoning with {} tools (Context messages: {}): {}",
                    agent.name,
                    agent.role,
                    tools.len(),
                    chat_history.len(),
                    task
                )
            }
        };

        // 6. Record checkpoint
        let checkpoint_json = serde_json::json!({
            "task": task,
            "status": "completed",
            "mode": mode,
            "messages_count": chat_history.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string();

        self.storage
            .save_checkpoint(&agent_id_str, 1, &checkpoint_json)
            .await?;

        // 7. Record outgoing reply message
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

        // 8. Return agent status to Idle
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_messages(count: usize, chars_per_msg: usize) -> Vec<ChatMessage> {
        let payload = "A".repeat(chars_per_msg);
        (0..count)
            .map(|i| ChatMessage {
                id: Uuid::new_v4(),
                role: if i == 0 {
                    MessageRole::System
                } else if i % 2 == 1 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                content: format!("Message {}: {}", i, payload),
                created_at: chrono::Utc::now(),
                tool_calls: None,
            })
            .collect()
    }

    #[tokio::test]
    async fn test_hermes_context_compression_triggers_on_threshold() {
        // Setup: small context window of 2,000 tokens (50% threshold is 1,000 tokens)
        let engine = HermesContextEngine::with_default_summarizer(2_000);

        // 30 messages of 200 chars each = ~1,500 tokens (exceeds 1,000 threshold)
        let mut messages = create_dummy_messages(30, 200);
        let orig_count = messages.len();

        let res = engine.compress_context(&mut messages).await.expect("compression");
        assert!(res.compressed);
        assert_eq!(res.original_message_count, orig_count);

        // Phase 2 & 4 rule: Head (1) + Summary (1) + Tail (20) = 22 messages
        assert_eq!(messages.len(), 22);
        assert_eq!(messages[0].role, MessageRole::System); // Head preserved
        assert!(messages[1].content.contains("[HERMES CONTEXT SUMMARY")); // Summary inserted
        assert_eq!(res.resulting_message_count, 22);
    }

    #[tokio::test]
    async fn test_hermes_context_compression_skips_when_under_threshold() {
        let engine = HermesContextEngine::with_default_summarizer(100_000);
        let mut messages = create_dummy_messages(10, 50);

        let res = engine.compress_context(&mut messages).await.expect("compression");
        assert!(!res.compressed);
        assert_eq!(messages.len(), 10);
    }

    #[test]
    fn test_deterministic_local_fallback() {
        let messages = create_dummy_messages(10, 100);
        let summary = DeterministicSummaryRoute::deterministic_local_fallback(&messages, 1000);
        assert!(summary.contains("[SYSTEM HERMES DETERMINISTIC CONDENSED HISTORY]"));
        assert!(summary.contains("Step 1"));
    }
}
