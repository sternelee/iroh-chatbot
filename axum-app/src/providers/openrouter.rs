use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use futures::stream::BoxStream;
use futures::StreamExt;
use async_trait::async_trait;

use crate::chat::{ChatMessage, ChatRole, UIMessageChunk};
use crate::providers::AIProvider;

/// OpenRouter service for AI model access
#[derive(Debug, Clone)]
pub struct OpenRouterService {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| anyhow!("OPENROUTER_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key))
    }

    async fn make_request(&self, messages: Vec<OpenRouterMessage>, model: Option<String>,
                         temperature: Option<f32>, max_tokens: Option<u32>,
                         stream: bool) -> Result<OpenRouterResponse> {
        let request = OpenRouterRequest {
            model: model.unwrap_or_else(|| "openai/gpt-3.5-turbo".to_string()),
            messages,
            temperature,
            max_tokens,
            stream,
            ..Default::default()
        };

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/iroh-chatbot")
            .header("X-Title", "Iroh Chatbot")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenRouter API error: {}", error_text));
        }

        let response_text = response.text().await?;
        let openrouter_response: OpenRouterResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse OpenRouter response: {}", e))?;

        Ok(openrouter_response)
    }

    fn convert_to_openrouter_messages(messages: &[ChatMessage]) -> Vec<OpenRouterMessage> {
        messages
            .iter()
            .map(|msg| OpenRouterMessage {
                role: match msg.role {
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                    ChatRole::System => "system".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect()
    }

    fn convert_from_openrouter_message(openrouter_msg: &OpenRouterChoice) -> ChatMessage {
        ChatMessage {
            id: openrouter_msg.message.id.clone(),
            role: match openrouter_msg.message.role.as_str() {
                "user" => ChatRole::User,
                "assistant" => ChatRole::Assistant,
                "system" => ChatRole::System,
                _ => ChatRole::Assistant,
            },
            content: openrouter_msg.message.content.clone(),
            created_at: Some(chrono::Utc::now()),
            attachments: None,
            metadata: None,
        }
    }

    pub fn get_model_options() -> Vec<&'static str> {
        vec![
            // OpenAI models
            "openai/gpt-4o",
            "openai/gpt-4o-mini",
            "openai/gpt-4-turbo",
            "openai/gpt-3.5-turbo",

            // Anthropic models
            "anthropic/claude-3.5-sonnet",
            "anthropic/claude-3.5-haiku",
            "anthropic/claude-3-opus",

            // Google models
            "google/gemini-pro-1.5",
            "google/gemini-flash-1.5",
            "google/gemini-pro",

            // Meta models
            "meta-llama/llama-3.1-405b-instruct",
            "meta-llama/llama-3.1-70b-instruct",
            "meta-llama/llama-3.1-8b-instruct",

            // Other popular models
            "mistralai/mixtral-8x7b-instruct",
            "cohere/command-r-plus",
            "perplexity/llama-3.1-sonar-small-128k-online",
        ]
    }
}

#[async_trait::async_trait]
impl AIProvider for OpenRouterService {
    async fn chat_completion(&self, messages: Vec<ChatMessage>, model: Option<String>,
                            temperature: Option<f32>, max_tokens: Option<u32>) -> Result<ChatMessage> {
        let openrouter_messages = Self::convert_to_openrouter_messages(&messages);
        let response = self.make_request(openrouter_messages, model, temperature, max_tokens, false).await?;

        if let Some(choice) = response.choices.first() {
            Ok(Self::convert_from_openrouter_message(choice))
        } else {
            Err(anyhow!("No response choices returned from OpenRouter"))
        }
    }

    async fn chat_completion_stream(&self, messages: Vec<ChatMessage>, model: Option<String>,
                                  temperature: Option<f32>, max_tokens: Option<u32>) -> BoxStream<'static, Result<UIMessageChunk, anyhow::Error>> {
        let openrouter_messages = Self::convert_to_openrouter_messages(&messages);
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();
        let selected_model = model.unwrap_or_else(|| "openai/gpt-3.5-turbo".to_string());

        let stream = async_stream::stream! {
            let request = OpenRouterRequest {
                model: selected_model,
                messages: openrouter_messages,
                temperature,
                max_tokens,
                stream: true,
                ..Default::default()
            };

            let client = Client::new();
            match client
                .post(&format!("{}/chat/completions", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://github.com/iroh-chatbot")
                .header("X-Title", "Iroh Chatbot")
                .json(&request)
                .send()
                .await {
                Ok(response) => {
                    if !response.status().is_success() {
                        let error_text = response.text().await.unwrap_or_default();
                        yield Err(anyhow!("OpenRouter API error: {}", error_text));
                        return;
                    }

                    let mut byte_stream = response.bytes_stream();
                    let mut buffer = String::new();

                    while let Some(chunk_result) = byte_stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if let Ok(chunk_str) = std::str::from_utf8(&chunk) {
                                    buffer.push_str(chunk_str);

                                    // Process complete lines
                                    while let Some(newline_pos) = buffer.find('\n') {
                                        let line = buffer[..newline_pos].to_string();
                                        buffer = buffer[newline_pos + 1..].to_string();

                                        if line.trim().is_empty() || !line.starts_with("data: ") {
                                            continue;
                                        }

                                        let data = &line[6..]; // Remove "data: " prefix

                                        if data.trim() == "[DONE]" {
                                            return;
                                        }

                                        match serde_json::from_str::<OpenRouterStreamChunk>(data) {
                                            Ok(chunk) => {
                                                if let Some(content) = chunk.choices
                                                    .first()
                                                    .and_then(|choice| choice.delta.content.as_ref()) {

                                                    yield Ok(UIMessageChunk::TextDelta {
                                                        textDelta: content.clone(),
                                                    });
                                                }
                                            }
                                            Err(e) => {
                                                // Skip malformed JSON but continue processing
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                yield Err(anyhow!("Stream error: {}", e));
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    yield Err(anyhow!("Failed to start stream: {}", e));
                    return;
                }
            }
        };

        Box::pin(stream)
    }

    fn get_available_models(&self) -> Vec<&'static str> {
        Self::get_model_options()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

impl Default for OpenRouterRequest {
    fn default() -> Self {
        Self {
            model: "openai/gpt-3.5-turbo".to_string(),
            messages: Vec::new(),
            temperature: None,
            max_tokens: None,
            stream: false,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenRouterChoice>,
    usage: Option<OpenRouterUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterChoice {
    index: u32,
    message: OpenRouterResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterResponseMessage {
    role: String,
    content: String,
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterStreamChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenRouterStreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterStreamChoice {
    index: u32,
    delta: OpenRouterDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenRouterDelta {
    role: Option<String>,
    content: Option<String>,
}