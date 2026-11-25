use async_trait::async_trait;
use qubit_protocol::{ChatMessage, ExecutionMode, MessageRole};
use serde_json::json;
use std::process::Stdio;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("retryable provider HTTP status: {0}")]
    RetryableHttp(u16),
    #[error("fatal provider error: {0}")]
    Fatal(String),
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn complete(
        &self,
        messages: &[ChatMessage],
        mode: ExecutionMode,
    ) -> Result<String, ProviderError>;
    fn name(&self) -> &str;
}

pub struct FallbackProviderChain {
    providers: Vec<Arc<dyn ModelProvider>>,
}

impl FallbackProviderChain {
    pub fn new(providers: Vec<Arc<dyn ModelProvider>>) -> Self {
        Self { providers }
    }

    pub async fn complete(
        &self,
        messages: &[ChatMessage],
        mode: ExecutionMode,
    ) -> Result<String, ProviderError> {
        let mut last_err = ProviderError::Fatal("no providers configured".to_string());
        for provider in &self.providers {
            match provider.complete(messages, mode).await {
                Ok(text) => return Ok(text),
                Err(ProviderError::RetryableHttp(code))
                    if matches!(code, 408 | 429 | 504 | 529) =>
                {
                    last_err = ProviderError::RetryableHttp(code);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_err)
    }
}

pub struct HttpApiProvider {
    client: reqwest::Client,
    endpoint: String,
    api_key: String,
    model: String,
}

impl HttpApiProvider {
    pub fn new(endpoint: String, api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
            api_key,
            model,
        }
    }
}

#[async_trait]
impl ModelProvider for HttpApiProvider {
    async fn complete(
        &self,
        messages: &[ChatMessage],
        _mode: ExecutionMode,
    ) -> Result<String, ProviderError> {
        let payload_msgs: Vec<_> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::System => "system",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => "tool",
                    MessageRole::User => "user",
                };
                json!({ "role": role, "content": m.content })
            })
            .collect();

        let res = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.model,
                "messages": payload_msgs
            }))
            .send()
            .await
            .map_err(|e| ProviderError::Fatal(e.to_string()))?;

        let status = res.status().as_u16();
        if matches!(status, 408 | 429 | 504 | 529) {
            return Err(ProviderError::RetryableHttp(status));
        }
        if !res.status().is_success() {
            return Err(ProviderError::Fatal(format!("HTTP {status}")));
        }

        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ProviderError::Fatal(e.to_string()))?;
        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProviderError::Fatal("missing content in response".to_string()))
    }

    fn name(&self) -> &str {
        "http-api"
    }
}

pub struct CliBridgeProvider {
    pub binary: String,
    pub args: Vec<String>,
}

#[async_trait]
impl ModelProvider for CliBridgeProvider {
    async fn complete(
        &self,
        messages: &[ChatMessage],
        _mode: ExecutionMode,
    ) -> Result<String, ProviderError> {
        let prompt = messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or_default();

        let mut child = Command::new(&self.binary)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ProviderError::Fatal(e.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(prompt.as_bytes()).await;
        }
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| ProviderError::Fatal(e.to_string()))?;
        if !output.status.success() {
            return Err(ProviderError::Fatal(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn name(&self) -> &str {
        "cli-bridge"
    }
}

pub struct OfflineEchoProvider;

#[async_trait]
impl ModelProvider for OfflineEchoProvider {
    async fn complete(
        &self,
        messages: &[ChatMessage],
        mode: ExecutionMode,
    ) -> Result<String, ProviderError> {
        let last = messages.last().map(|m| m.content.as_str()).unwrap_or("");
        Ok(format!("[qubit-core:{mode:?}] {last}"))
    }

    fn name(&self) -> &str {
        "offline-echo"
    }
}
