use async_trait::async_trait;
use qubit_agents::{ContextCompressor, FallbackProviderChain, ModelProvider, ProviderError};
use qubit_protocol::{ChatMessage, ExecutionMode, MessageRole};
use std::sync::Arc;
use uuid::Uuid;

struct FailingProvider(u16);
#[async_trait]
impl ModelProvider for FailingProvider {
    async fn complete(
        &self,
        _: &[ChatMessage],
        _: ExecutionMode,
    ) -> Result<String, ProviderError> {
        Err(ProviderError::RetryableHttp(self.0))
    }
    fn name(&self) -> &str {
        "failing"
    }
}

struct OkProvider;
#[async_trait]
impl ModelProvider for OkProvider {
    async fn complete(
        &self,
        _: &[ChatMessage],
        mode: ExecutionMode,
    ) -> Result<String, ProviderError> {
        Ok(format!("ok:{mode:?}"))
    }
    fn name(&self) -> &str {
        "ok"
    }
}

#[tokio::test]
async fn fallback_chain_switches_on_529_and_compressor_protects_last_20() {
    let chain = FallbackProviderChain::new(vec![
        Arc::new(FailingProvider(529)),
        Arc::new(OkProvider),
    ]);
    let reply = chain.complete(&[], ExecutionMode::Reflex).await.unwrap();
    assert_eq!(reply, "ok:Reflex");

    let sid = Uuid::new_v4();
    let msgs: Vec<ChatMessage> = (0..35)
        .map(|i| {
            ChatMessage::new(
                sid,
                MessageRole::User,
                format!("message-{i}-{}", "x".repeat(50)),
                false,
            )
        })
        .collect();
    let compressed = ContextCompressor::compress(msgs, 2000);
    assert!(compressed.len() <= 21);
    assert!(compressed.last().unwrap().content.starts_with("message-34-"));
}
