//! Ollama `/api/chat` client (feature `ollama`).

use async_trait::async_trait;
use machi_types::{ErrorCode, MachiError, Message, Role, Usage};
use serde_json::{Value, json};
use tracing::{Instrument, info_span};

use crate::openai_compat::http_status_error;
use crate::sample::{SampleRequest, SampleResponse, ToolChoice};
use crate::sampler::LlmSampler;

/// Ollama HTTP configuration.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Base URL without trailing slash (default `http://127.0.0.1:11434`).
    pub base_url: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:11434".into(),
        }
    }
}

impl OllamaConfig {
    /// Create from base URL.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut base_url = base_url.into();
        while base_url.ends_with('/') {
            base_url.pop();
        }
        Self { base_url }
    }

    /// Chat endpoint URL.
    #[must_use]
    pub fn chat_url(&self) -> String {
        format!("{}/api/chat", self.base_url)
    }
}

/// Build Ollama `/api/chat` JSON body.
#[must_use]
pub fn build_ollama_chat_body(req: &SampleRequest) -> Value {
    let messages: Vec<Value> = req
        .messages
        .iter()
        .map(|m| {
            json!({
                "role": m.role.as_str(),
                "content": m.text(),
            })
        })
        .collect();
    let tools = if req.tools.is_empty() || matches!(req.tool_choice, ToolChoice::None) {
        None
    } else {
        Some(Value::Array(
            req.tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect(),
        ))
    };
    let mut body = json!({
        "model": req.model,
        "messages": messages,
        "stream": false,
    });
    if let Some(tools) = tools
        && let Some(obj) = body.as_object_mut()
    {
        obj.insert("tools".into(), tools);
    }
    body
}

/// Parse Ollama `/api/chat` non-streaming response.
///
/// # Errors
///
/// Invalid JSON shape yields [`ErrorCode::LlmInvalidResponse`].
pub fn parse_ollama_chat_response(body: &Value) -> Result<SampleResponse, MachiError> {
    let message_v = body.get("message").ok_or_else(|| {
        MachiError::new(
            ErrorCode::LlmInvalidResponse,
            "ollama response missing message",
        )
    })?;
    let content = message_v
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let mut tool_calls = Vec::new();
    if let Some(arr) = message_v.get("tool_calls").and_then(Value::as_array) {
        for (i, item) in arr.iter().enumerate() {
            let name = item
                .pointer("/function/name")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let args = item
                .pointer("/function/arguments")
                .cloned()
                .unwrap_or(json!({}));
            let id = format!("ollama_call_{i}");
            tool_calls.push(machi_types::ToolCall {
                id: machi_types::ToolCallId::new(id)?,
                name: name.to_owned(),
                arguments: args,
            });
        }
    }
    let message = if tool_calls.is_empty() {
        Message::assistant(content)
    } else {
        let mut m = Message::assistant_tools(tool_calls);
        if !content.is_empty() {
            m.content = Some(content);
        }
        m
    };
    // Ollama may not always report token usage the same way.
    let usage = Usage::zero();
    let stop_reason = body
        .get("done_reason")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let _ = Role::Assistant;
    Ok(SampleResponse {
        message,
        usage,
        stop_reason,
    })
}

/// HTTP client for Ollama chat.
#[derive(Debug, Clone)]
pub struct OllamaSampler {
    config: OllamaConfig,
    client: reqwest::Client,
}

impl OllamaSampler {
    /// Default client.
    ///
    /// # Errors
    ///
    /// Client build failures.
    pub fn new(config: OllamaConfig) -> Result<Self, MachiError> {
        let client = reqwest::Client::builder().build().map_err(|e| {
            MachiError::new(ErrorCode::LlmProvider, format!("http client build: {e}"))
        })?;
        Ok(Self { config, client })
    }

    /// Inject client (tests).
    #[must_use]
    pub fn with_client(config: OllamaConfig, client: reqwest::Client) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl LlmSampler for OllamaSampler {
    async fn sample(&self, request: SampleRequest) -> Result<SampleResponse, MachiError> {
        if request.cancel.is_cancelled() {
            return Err(MachiError::new(ErrorCode::LlmCancelled, "sample cancelled"));
        }
        let body = build_ollama_chat_body(&request);
        let url = self.config.chat_url();
        let span = info_span!(
            "machi.sample.http",
            machi.model = %request.model,
            machi.provider = "ollama",
        );
        async move {
            let response = self
                .client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    MachiError::new(ErrorCode::LlmProvider, format!("http request failed: {e}"))
                        .with_retry(machi_types::RetryClass::Backoff)
                })?;
            let status = response.status().as_u16();
            let text = response.text().await.map_err(|e| {
                MachiError::new(
                    ErrorCode::LlmProvider,
                    format!("http body read failed: {e}"),
                )
            })?;
            if !(200..300).contains(&status) {
                return Err(http_status_error(status, &text));
            }
            let value: Value = serde_json::from_str(&text).map_err(|e| {
                MachiError::new(
                    ErrorCode::LlmInvalidResponse,
                    format!("invalid JSON body: {e}"),
                )
            })?;
            parse_ollama_chat_response(&value)
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text() {
        let body = json!({
            "message": { "role": "assistant", "content": "hi from ollama" },
            "done": true,
            "done_reason": "stop"
        });
        let resp = parse_ollama_chat_response(&body).expect("parse");
        assert_eq!(resp.message.text(), "hi from ollama");
    }
}
