use anyhow::Result;
use async_stream::stream;
use futures::{stream::BoxStream, Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::chat::{ChatMessage, ChatRole, UIMessageChunk};

/// Google Gemini API Service
/// Provides integration with Google's Gemini AI models
#[derive(Debug)]
pub struct GeminiService {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl GeminiService {
    /// Create a new Gemini service with the given API key
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            default_model: model.unwrap_or_else(|| "gemini-1.5-flash".to_string()),
        }
    }

    /// Create Gemini service from environment variables
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GOOGLE_AI_API_KEY")
            .map_err(|_| anyhow::anyhow!("GOOGLE_AI_API_KEY environment variable not set"))?;

        let model = std::env::var("GEMINI_MODEL").ok();
        Ok(Self::new(api_key, model))
    }

    /// Get available Gemini models
    pub fn get_model_options() -> Vec<&'static str> {
        vec![
            "gemini-1.5-flash",
            "gemini-1.5-pro",
            "gemini-1.5-flash-8b",
            "gemini-pro",
            "gemini-pro-vision",
        ]
    }

    /// Convert chat messages to Gemini format
    fn convert_to_gemini_messages(messages: &[ChatMessage]) -> Vec<GeminiContent> {
        messages
            .iter()
            .filter_map(|msg| {
                match msg.role {
                    ChatRole::User => Some(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart {
                            text: msg.content.clone(),
                        }],
                    }),
                    ChatRole::Assistant => Some(GeminiContent {
                        role: "model".to_string(),
                        parts: vec![GeminiPart {
                            text: msg.content.clone(),
                        }],
                    }),
                    ChatRole::System => {
                        // Gemini doesn't have system messages, so we prepend to the first user message
                        None
                    }
                }
            })
            .collect()
        }

    /// Extract system message and prepend to first user message
    fn extract_system_message(messages: &[ChatMessage]) -> (Option<String>, Vec<ChatMessage>) {
        let system_msg = messages
            .iter()
            .find(|msg| matches!(msg.role, ChatRole::System))
            .map(|msg| msg.content.clone());

        let mut filtered_messages: Vec<ChatMessage> = messages
            .iter()
            .filter(|msg| !matches!(msg.role, ChatRole::System))
            .cloned()
            .collect();

        // If we have a system message and user messages, prepend it
        if let (Some(sys_msg), Some(first_user)) = (system_msg.as_ref(), filtered_messages.first_mut()) {
            first_user.content = format!("{}\n\n{}", sys_msg, first_user.content);
        }

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

        let gemini_contents = Self::convert_to_gemini_messages(&filtered_messages);

        let request = GeminiRequest {
            contents: gemini_contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature,
                max_output_tokens: max_tokens,
                top_p: None,
                top_k: None,
                candidate_count: Some(1),
                stop_sequences: None,
            }),
            safety_settings: Some(Self::default_safety_settings()),
        };

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url, model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        if let Some(candidates) = gemini_response.candidates {
            if let Some(candidate) = candidates.first() {
                if let Some(content) = &candidate.content {
                    if let Some(part) = content.parts.first() {
                        return Ok(ChatMessage {
                            id: format!("gemini_{}", fastrand::u64(1000..9999)),
                            role: ChatRole::Assistant,
                            content: part.text.clone(),
                            created_at: Some(chrono::Utc::now()),
                            attachments: None,
                            metadata: Some(HashMap::from([
                                ("model".to_string(), serde_json::Value::String(model)),
                                ("provider".to_string(), serde_json::Value::String("gemini".to_string())),
                            ])),
                        });
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No valid response from Gemini API"))
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

        let gemini_contents = Self::convert_to_gemini_messages(&filtered_messages);
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();

        Box::pin(stream! {
            // Send text-start event
            yield Ok(UIMessageChunk::TextStart);

            let request = GeminiRequest {
                contents: gemini_contents,
                generation_config: Some(GeminiGenerationConfig {
                    temperature,
                    max_output_tokens: max_tokens,
                    top_p: None,
                    top_k: None,
                    candidate_count: Some(1),
                    stop_sequences: None,
                }),
                safety_settings: Some(Self::default_safety_settings()),
            };

            let url = format!(
                "{}/v1beta/models/{}:streamGenerateContent?key={}",
                base_url, model, api_key
            );

            let client = Client::new();

            match client.post(&url).json(&request).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        let error_text = response.text().await.unwrap_or_default();
                        yield Ok(UIMessageChunk::Error {
                            error: format!("Gemini API error: {}", error_text),
                        });
                        return;
                    }

                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                                    buffer.push_str(&text);

                                    // Process complete JSON objects
                                    while let Some(newline_pos) = buffer.find('\n') {
                                        let line = buffer[..newline_pos].to_string();
                                        buffer = buffer[newline_pos + 1..].to_string();

                                        if line.trim().starts_with("data: ") {
                                            let json_str = &line[6..];
                                            if let Ok(gemini_chunk) = serde_json::from_str::<GeminiStreamResponse>(json_str) {
                                                let has_candidates = gemini_chunk.candidates.is_some();
                                                if let Some(ref candidates) = gemini_chunk.candidates {
                                                    if let Some(candidate) = candidates.first() {
                                                        if let Some(content) = &candidate.content {
                                                            if let Some(part) = content.parts.first() {
                                                                yield Ok(UIMessageChunk::TextDelta {
                                                                    textDelta: part.text.clone(),
                                                                });
                                                            }
                                                        }
                                                    }
                                                }

                                                // Check if this is a completion chunk
                                                if !has_candidates && gemini_chunk.prompt_feedback.is_none() {
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

    /// Default safety settings for Gemini
    fn default_safety_settings() -> Vec<GeminiSafetySetting> {
        vec![
            GeminiSafetySetting {
                category: "HARM_CATEGORY_HARASSMENT".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
            GeminiSafetySetting {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
            GeminiSafetySetting {
                category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
            GeminiSafetySetting {
                category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
        ]
    }
}

// Gemini API structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_settings: Option<Vec<GeminiSafetySetting>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    role: String, // "user" or "model"
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiSafetySetting {
    category: String,
    threshold: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    prompt_feedback: Option<GeminiPromptFeedback>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiStreamResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    prompt_feedback: Option<GeminiPromptFeedback>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
    finish_reason: Option<String>,
    index: Option<i32>,
    safety_ratings: Option<Vec<GeminiSafetyRating>>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiPromptFeedback {
    block_reason: Option<String>,
    safety_ratings: Option<Vec<GeminiSafetyRating>>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiSafetyRating {
    category: String,
    probability: String,
    blocked: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_options() {
        let models = GeminiService::get_model_options();
        assert!(!models.is_empty());
        assert!(models.contains(&"gemini-1.5-flash"));
        assert!(models.contains(&"gemini-1.5-pro"));
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

        let gemini_messages = GeminiService::convert_to_gemini_messages(&messages);
        assert_eq!(gemini_messages.len(), 2);
        assert_eq!(gemini_messages[0].role, "user");
        assert_eq!(gemini_messages[1].role, "model");
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

        let (system_msg, filtered) = GeminiService::extract_system_message(&messages);
        assert!(system_msg.is_some());
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].content.contains("You are a helpful assistant."));
        assert!(filtered[0].content.contains("Hello"));
    }
}