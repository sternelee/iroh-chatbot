use anyhow::Result;
use async_stream::stream;
use futures::{stream::BoxStream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;

use crate::chat::{ChatMessage, ChatRole, UIMessageChunk};

/// Anthropic Claude API Service
/// Provides integration with Anthropic's Claude AI models
#[derive(Debug)]
pub struct AnthropicService {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl AnthropicService {
    /// Create a new Anthropic service with the given API key
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        "anthropic-version",
                        reqwest::header::HeaderValue::from_static("2023-06-01"),
                    );
                    headers.insert(
                        "content-type",
                        reqwest::header::HeaderValue::from_static("application/json"),
                    );
                    headers
                })
                .build()
                .unwrap(),
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            default_model: model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
        }
    }

    /// Create Anthropic service from environment variables
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY environment variable not set"))?;

        let model = std::env::var("ANTHROPIC_MODEL").ok();
        Ok(Self::new(api_key, model))
    }

    /// Get available Anthropic models
    pub fn get_model_options() -> Vec<&'static str> {
        vec![
            "claude-3-5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
            "claude-3-opus-20240229",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
        ]
    }

    /// Convert chat messages to Anthropic format
    fn convert_to_anthropic_messages(messages: &[ChatMessage]) -> Vec<AnthropicMessage> {
        messages
            .iter()
            .filter_map(|msg| {
                match msg.role {
                    ChatRole::User => Some(AnthropicMessage {
                        role: "user".to_string(),
                        content: vec![AnthropicContent {
                            type_: "text".to_string(),
                            text: msg.content.clone(),
                        }],
                    }),
                    ChatRole::Assistant => Some(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: vec![AnthropicContent {
                            type_: "text".to_string(),
                            text: msg.content.clone(),
                        }],
                    }),
                    ChatRole::System => {
                        // Anthropic supports system messages in a different way
                        // We'll handle this in the request building
                        None
                    }
                }
            })
            .collect()
    }

    /// Extract system message and handle it separately
    fn extract_system_message(messages: &[ChatMessage]) -> (Option<String>, Vec<ChatMessage>) {
        let system_msg = messages
            .iter()
            .find(|msg| matches!(msg.role, ChatRole::System))
            .map(|msg| msg.content.clone());

        let filtered_messages: Vec<ChatMessage> = messages
            .iter()
            .filter(|msg| !matches!(msg.role, ChatRole::System))
            .cloned()
            .collect();

        (system_msg, filtered_messages)
    }

    /// Generate chat completion (non-streaming)
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<ChatMessage> {
        let model = model.unwrap_or_else(|| self.default_model.clone());
        let (system_msg, filtered_messages) = Self::extract_system_message(&messages);

        if filtered_messages.is_empty() {
            return Err(anyhow::anyhow!("No valid messages to process"));
        }

        let anthropic_messages = Self::convert_to_anthropic_messages(&filtered_messages);

        let mut request = AnthropicRequest {
            model: model.clone(),
            messages: anthropic_messages,
            max_tokens: max_tokens.unwrap_or(4096),
            temperature: temperature.unwrap_or(0.7),
            stream: false,
            system: system_msg,
        };

        let url = format!("{}/v1/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        if let Some(content) = anthropic_response.content.first() {
            return Ok(ChatMessage {
                id: format!("claude_{}", fastrand::u64(1000..9999)),
                role: ChatRole::Assistant,
                content: content.text.clone(),
                created_at: Some(chrono::Utc::now()),
                attachments: None,
                metadata: Some(HashMap::from([
                    ("model".to_string(), serde_json::Value::String(model)),
                    ("provider".to_string(), serde_json::Value::String("anthropic".to_string())),
                ])),
            });
        }

        Err(anyhow::anyhow!("No valid response from Anthropic API"))
    }

    /// Generate chat completion (streaming)
    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> BoxStream<'static, Result<UIMessageChunk, anyhow::Error>> {
        let model = model.unwrap_or_else(|| self.default_model.clone());
        let (system_msg, filtered_messages) = Self::extract_system_message(&messages);

        if filtered_messages.is_empty() {
            return Box::pin(stream! {
                yield Err(anyhow::anyhow!("No valid messages to process"));
            });
        }

        let anthropic_messages = Self::convert_to_anthropic_messages(&filtered_messages);
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();

        Box::pin(stream! {
            // Send text-start event
            yield Ok(UIMessageChunk::TextStart);

            let mut request = AnthropicRequest {
                model: model.clone(),
                messages: anthropic_messages,
                max_tokens: max_tokens.unwrap_or(4096),
                temperature: temperature.unwrap_or(0.7),
                stream: true,
                system: system_msg,
            };

            let url = format!("{}/v1/messages", base_url);

            let client = Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        "anthropic-version",
                        reqwest::header::HeaderValue::from_static("2023-06-01"),
                    );
                    headers.insert(
                        "content-type",
                        reqwest::header::HeaderValue::from_static("application/json"),
                    );
                    headers
                })
                .build()
                .unwrap();

            match client.post(&url).header("x-api-key", &api_key).json(&request).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        let error_text = response.text().await.unwrap_or_default();
                        yield Ok(UIMessageChunk::Error {
                            error: format!("Anthropic API error: {}", error_text),
                        });
                        return;
                    }

                    let mut stream = response.bytes_stream();

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                                    // Process Anthropic's SSE format
                                    let lines: Vec<&str> = text.split('\n').collect();
                                    for line in lines {
                                        if line.trim().is_empty() {
                                            continue;
                                        }
                                        if line.trim() == "event: message_stop" {
                                            yield Ok(UIMessageChunk::TextFinish);

                                            // Send finish event
                                            let usage = crate::chat::Usage {
                                                prompt_tokens: 0,
                                                completion_tokens: 0,
                                                total_tokens: 0,
                                            };
                                            yield Ok(UIMessageChunk::Finish {
                                                reasoning: None,
                                                sources: None,
                                                usage: Some(usage),
                                                logprobs: None,
                                            });
                                            return;
                                        }
                                        if line.trim().starts_with("data: ") {
                                            let json_str = &line.trim()[6..];
                                            if let Ok(anthropic_chunk) = serde_json::from_str::<AnthropicStreamChunk>(json_str) {
                                                if let Some(delta) = anthropic_chunk.delta {
                                                    if let Some(text) = delta.text {
                                                        yield Ok(UIMessageChunk::TextDelta {
                                                            textDelta: text,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                yield Ok(UIMessageChunk::Error {
                                    error: format!("Stream error: {}", e),
                                });
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    yield Ok(UIMessageChunk::Error {
                        error: format!("Request error: {}", e),
                    });
                }
            }
        })
    }
}

// Anthropic API structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(rename = "stream")]
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String, // "user" or "assistant"
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    type_: String, // "text"
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    role: String,
    content: Vec<AnthropicContent>,
    #[serde(rename = "model")]
    model: String,
    #[serde(rename = "stop_reason")]
    stop_reason: Option<String>,
    #[serde(rename = "stop_sequence")]
    stop_sequence: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicStreamChunk {
    #[serde(rename = "type")]
    type_: String,
    delta: Option<AnthropicDelta>,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    type_: String,
    text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")]
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_options() {
        let models = AnthropicService::get_model_options();
        assert!(!models.is_empty());
        assert!(models.contains(&"claude-3-5-sonnet-20241022"));
        assert!(models.contains(&"claude-3-haiku-20240307"));
    }

    #[test]
    fn test_convert_messages() {
        let messages = vec![
            ChatMessage {
                id: "1".to_string(),
                role: ChatRole::User,
                content: "Hello".to_string(),
                created_at: None,
                attachments: None,
                metadata: None,
            },
            ChatMessage {
                id: "2".to_string(),
                role: ChatRole::Assistant,
                content: "Hi there!".to_string(),
                created_at: None,
                attachments: None,
                metadata: None,
            },
        ];

        let anthropic_messages = AnthropicService::convert_to_anthropic_messages(&messages);
        assert_eq!(anthropic_messages.len(), 2);
        assert_eq!(anthropic_messages[0].role, "user");
        assert_eq!(anthropic_messages[1].role, "assistant");
    }

    #[test]
    fn test_extract_system_message() {
        let messages = vec![
            ChatMessage {
                id: "1".to_string(),
                role: ChatRole::System,
                content: "You are a helpful assistant.".to_string(),
                created_at: None,
                attachments: None,
                metadata: None,
            },
            ChatMessage {
                id: "2".to_string(),
                role: ChatRole::User,
                content: "Hello".to_string(),
                created_at: None,
                attachments: None,
                metadata: None,
            },
        ];

        let (system_msg, filtered) = AnthropicService::extract_system_message(&messages);
        assert!(system_msg.is_some());
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].role, ChatRole::User);
    }

    #[test]
    fn test_claude_model_names() {
        let models = AnthropicService::get_model_options();
        assert!(models.iter().any(|&m| m.contains("claude")));
        assert!(models.iter().any(|&m| m.contains("sonnet")));
        assert!(models.iter().any(|&m| m.contains("haiku")));
        assert!(models.iter().any(|&m| m.contains("opus")));
    }
}