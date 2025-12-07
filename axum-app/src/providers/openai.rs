use anyhow::{anyhow, Result};
use async_stream::stream;
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::chat::{ChatMessage, ChatRole, UIMessageChunk};
use super::AIProvider;

#[derive(Debug, Clone)]
pub struct OpenAIService {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl OpenAIService {
    pub fn new(api_key: String, default_model: Option<String>) -> Self {
        let base_url = std::env::var("OPENAI_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        let default_model = default_model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        Self {
            client,
            api_key,
            base_url,
            default_model,
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;

        let default_model = std::env::var("OPENAI_DEFAULT_MODEL").ok();

        Ok(Self::new(api_key, default_model))
    }

    /// Convert our ChatMessage to OpenAI format
    fn convert_to_openai_message(message: ChatMessage) -> OpenAIChatMessage {
        let role = match message.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::System => "system",
        };

        OpenAIChatMessage {
            role: role.to_string(),
            content: message.content,
        }
    }

    /// Convert OpenAI response to our format
    fn convert_from_openai_response(response: &OpenAIChatResponse) -> ChatMessage {
        let choice = response.choices.first();
        let content = choice
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "No response generated".to_string());

        ChatMessage {
            id: response.id.clone(),
            role: ChatRole::Assistant,
            content,
            created_at: Some(chrono::Utc::now()),
            attachments: None,
            metadata: Some(HashMap::from([
                ("model".to_string(), serde_json::Value::String(response.model.clone())),
                ("usage".to_string(), serde_json::to_value(&response.usage).unwrap_or(serde_json::Value::Null)),
                ("finish_reason".to_string(), serde_json::Value::String(
                    choice.and_then(|c| c.finish_reason.clone()).unwrap_or("unknown".to_string())
                )),
            ])),
        }
    }

    /// Send chat completion request (non-streaming)
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<ChatMessage> {
        let openai_messages: Vec<OpenAIChatMessage> =
            messages.into_iter().map(Self::convert_to_openai_message).collect();

        let model_name = model.unwrap_or_else(|| self.default_model.clone());

        let request = OpenAIChatRequest {
            model: model_name,
            messages: openai_messages,
            temperature: temperature.unwrap_or(0.7),
            max_tokens: max_tokens.unwrap_or(1000),
            stream: false,
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI API error: {} - {}", status, error_text));
        }

        let openai_response: OpenAIChatResponse = response.json().await?;
        Ok(Self::convert_from_openai_response(&openai_response))
    }

    /// Send streaming chat completion request
    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<BoxStream<'static, Result<UIMessageChunk, anyhow::Error>>> {
        let openai_messages: Vec<OpenAIChatMessage> =
            messages.into_iter().map(Self::convert_to_openai_message).collect();

        let model_name = model.unwrap_or_else(|| self.default_model.clone());

        let request = OpenAIChatRequest {
            model: model_name,
            messages: openai_messages,
            temperature: temperature.unwrap_or(0.7),
            max_tokens: max_tokens.unwrap_or(1000),
            stream: true,
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI API error: {} - {}", status, error_text));
        }

        let stream = Box::pin(stream! {
            let mut bytes_stream = response.bytes_stream();

            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk);

                        // Process SSE data
                        for line in chunk_str.lines() {
                            let line = line.trim();
                            if line.starts_with("data: ") {
                                let data = &line[6..];

                                if data == "[DONE]" {
                                    // Send finish event
                                    yield Ok(UIMessageChunk::Finish {
                                        reasoning: None,
                                        sources: None,
                                        usage: None,
                                        logprobs: None,
                                    });
                                    break;
                                }

                                // Parse JSON chunk
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                                    if let Some(choices) = parsed.get("choices").and_then(|c| c.as_array()) {
                                        if let Some(choice) = choices.first() {
                                            if let Some(delta) = choice.get("delta") {
                                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                                    yield Ok(UIMessageChunk::TextDelta {
                                                        textDelta: content.to_string(),
                                                    });
                                                }
                                            }

                                            // Check for finish reason
                                            if let Some(finish_reason) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                                                if finish_reason != "null" {
                                                    yield Ok(UIMessageChunk::Finish {
                                                        reasoning: None,
                                                        sources: None,
                                                        usage: None,
                                                        logprobs: None,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(anyhow!("Stream error: {}", e));
                        break;
                    }
                }
            }
        });

        Ok(stream)
    }

    /// Get available model options
    pub fn get_model_options() -> Vec<&'static str> {
        vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-4",
            "gpt-3.5-turbo",
        ]
    }
}

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChatChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl AIProvider for OpenAIService {
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<ChatMessage> {
        Self::chat_completion(self, messages, model, temperature, max_tokens).await
    }

    async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> BoxStream<'static, Result<UIMessageChunk, anyhow::Error>> {
        Self::chat_completion_stream(self, messages, model, temperature, max_tokens).await.unwrap()
    }

    fn get_available_models(&self) -> Vec<&'static str> {
        Self::get_model_options()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires OPENAI_API_KEY environment variable
    async fn test_openai_service_creation() {
        dotenvy::dotenv().ok();

        let service = OpenAIService::from_env();
        assert!(service.is_ok());
    }
}