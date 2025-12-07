//! AI Provider Services Module
//!
//! This module contains implementations for various AI providers including OpenAI, Anthropic, Google Gemini, and OpenRouter.
//! Each provider implements a common interface for chat completion and streaming.

use async_trait::async_trait;
use anyhow::Result;
use futures::stream::BoxStream;
use crate::chat::{ChatMessage, UIMessageChunk};

pub mod openai;
pub mod anthropic;
pub mod gemini;
pub mod openrouter;

// Re-export the main service structs
pub use openai::OpenAIService;
pub use anthropic::AnthropicService;
pub use gemini::GeminiService;
pub use openrouter::OpenRouterService;

/// Common trait for AI providers
#[async_trait]
pub trait AIProvider {
    /// Non-streaming chat completion
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<ChatMessage>;

    /// Streaming chat completion
    async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> BoxStream<'static, Result<UIMessageChunk, anyhow::Error>>;

    /// Get available models for this provider
    fn get_available_models(&self) -> Vec<&'static str>;
}

