use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
};
use futures::{stream, Stream, StreamExt};
use async_stream::stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::{AppState};
use crate::providers::{GeminiService, OpenAIService, AnthropicService};

/// Chat message structure compatible with AI SDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Chat completion request - AI SDK compatible format
#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,
}

/// Chat completion delta for streaming responses
#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionDelta {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChoiceDelta>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceDelta {
    pub index: usize,
    pub delta: MessageDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ChatRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// AI SDK compatible UI Message Stream chunk
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum UIMessageChunk {
    #[serde(rename = "text-start")]
    TextStart,
    #[serde(rename = "text-delta")]
    TextDelta {
        textDelta: String,
    },
    #[serde(rename = "text-finish")]
    TextFinish,
    #[serde(rename = "tool-call")]
    ToolCall {
        toolCallId: String,
        toolName: String,
        args: serde_json::Value,
    },
    #[serde(rename = "tool-result")]
    ToolResult {
        toolCallId: String,
        result: serde_json::Value,
    },
    #[serde(rename = "step-finish")]
    StepFinish {
        isContinued: bool,
    },
    #[serde(rename = "finish")]
    Finish {
        #[serde(skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sources: Option<Vec<serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<Usage>,
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<serde_json::Value>,
    },
    #[serde(rename = "error")]
    Error {
        error: String,
    },
    #[serde(rename = "data")]
    Data {
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Determine the provider based on model name
fn get_provider_from_model(model: &str) -> Provider {
    if model.starts_with("gemini") || model.starts_with("models/gemini") {
        Provider::Gemini
    } else if model.starts_with("claude") {
        Provider::Anthropic
    } else {
        Provider::OpenAI
    }
}

/// AI provider enum
#[derive(Debug, Clone, PartialEq)]
enum Provider {
    OpenAI,
    Gemini,
    Anthropic,
}

/// Generate mock AI response
fn generate_mock_response() -> &'static str {
    let responses = [
        "That's interesting! Tell me more about that.",
        "I understand. How can I help you with that?",
        "Thanks for sharing! What else would you like to discuss?",
        "I see your point. Let me think about that for a moment.",
        "That's a great question! Here's what I think about it.",
        "I appreciate you sharing that with me.",
        "That makes sense. What are your thoughts on this?",
        "Interesting perspective! Have you considered other angles?",
        "I'd love to help you explore that idea further.",
        "That's a fascinating topic! Let me share what I know about it.",
    ];

    responses[fastrand::usize(0..responses.len())]
}

/// Create streaming response using AI SDK patterns
fn create_streaming_response(
    content: String,
) -> impl Stream<Item = Result<Event, anyhow::Error>> {
    let words: Vec<String> = content
        .split_whitespace()
        .map(|word| word.to_string() + " ")
        .collect();

    stream! {
        // Send text-start event first
        yield Ok::<Event, anyhow::Error>(
            Event::default().json_data(UIMessageChunk::TextStart)
                .map_err(|e| anyhow::anyhow!("SSE error: {}", e))?
        );

        // Send text-delta events for each word
        for (index, word) in words.into_iter().enumerate() {
            let chunk = UIMessageChunk::TextDelta {
                textDelta: word,
            };

            yield Ok::<Event, anyhow::Error>(
                Event::default().json_data(chunk)
                    .map_err(|e| anyhow::anyhow!("SSE error: {}", e))?
            );
        }

        // Send text-finish event at the end
        yield Ok::<Event, anyhow::Error>(
            Event::default().json_data(UIMessageChunk::TextFinish)
                .map_err(|e| anyhow::anyhow!("SSE error: {}", e))?
        );

        // Send final finish event with usage
        let usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };
        let finish_chunk = UIMessageChunk::Finish {
            reasoning: None,
            sources: None,
            usage: Some(usage),
            logprobs: None,
        };
        yield Ok::<Event, anyhow::Error>(
            Event::default().json_data(finish_chunk)
                .map_err(|e| anyhow::anyhow!("SSE error: {}", e))?
        );
    }
}

/// Create non-streaming response
fn create_chat_response(messages: &[ChatMessage]) -> ChatMessage {
    let user_message = messages.last().and_then(|msg| {
        if matches!(msg.role, ChatRole::User) {
            Some(msg.content.clone())
        } else {
            None
        }
    });

    let response_content = user_message
        .map(|_| generate_mock_response().to_string())
        .unwrap_or_else(|| "Hello! How can I help you today?".to_string());

    ChatMessage {
        id: format!("msg_{}", fastrand::u64(1000..9999)),
        role: ChatRole::Assistant,
        content: response_content,
        created_at: Some(chrono::Utc::now()),
        attachments: None,
        metadata: None,
    }
}


/// Chat completion endpoint - handles both OpenAI and Gemini with streaming and non-streaming
pub async fn chat_completion(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, StatusCode> {
    if request.messages.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Determine provider based on model (default to OpenAI)
    let model = request.model.clone().unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    let provider = get_provider_from_model(&model);

    // Route to appropriate provider
    match provider {
        Provider::OpenAI => {
            handle_openai_request(state, request, &model).await
        }
        Provider::Gemini => {
            handle_gemini_request(state, request, &model).await
        }
        Provider::Anthropic => {
            handle_anthropic_request(state, request, &model).await
        }
    }
}

/// Handle OpenAI requests
async fn handle_openai_request(
    state: AppState,
    request: ChatCompletionRequest,
    model: &str,
) -> Result<Response, StatusCode> {
    // Check if OpenAI service is available
    let openai_service = match &state.openai_service {
        Some(service) => service,
        None => {
            // Fallback to mock response if OpenAI service is not configured
            return handle_fallback_response(request).await;
        }
    };

    // Check if streaming is requested
    if request.stream.unwrap_or(false) {
        // Streaming response
        match openai_service
            .chat_completion_stream(request.messages, Some(model.to_string()), request.temperature, request.max_tokens)
            .await
        {
            Ok(openai_stream) => {
                // Convert OpenAI stream to SSE format
                let sse_stream = stream! {
                    for await result in openai_stream {
                        match result {
                            Ok(chunk) => {
                                yield Ok::<Event, Box<dyn std::error::Error + Send + Sync>>(
                                    Event::default().json_data(chunk)
                                        .unwrap_or_else(|_| Event::default().data("serialization error"))
                                );
                            }
                            Err(e) => {
                                yield Ok::<Event, Box<dyn std::error::Error + Send + Sync>>(
                                    Event::default().data(format!("error: {}", e))
                                );
                            }
                        }
                    }
                };

                let sse_response = Sse::new(sse_stream)
                    .keep_alive(
                        axum::response::sse::KeepAlive::new()
                            .interval(Duration::from_secs(15))
                            .text("keep-alive-text"),
                    )
                    .into_response();

                // Add AI SDK compatible headers
                let mut response = sse_response;
                let headers = response.headers_mut();
                headers.insert("content-type", "text/event-stream".parse().unwrap());
                headers.insert("cache-control", "no-cache".parse().unwrap());
                headers.insert("connection", "keep-alive".parse().unwrap());
                headers.insert("x-vercel-ai-ui-message-stream", "v1".parse().unwrap());
                headers.insert("x-accel-buffering", "no".parse().unwrap());

                Ok(response)
            }
            Err(e) => {
                // Return error response
                let error_response = Json(serde_json::json!({
                    "error": {
                        "message": format!("OpenAI API error: {}", e),
                        "type": "api_error",
                        "code": "openai_error"
                    }
                }));
                Ok((StatusCode::INTERNAL_SERVER_ERROR, error_response).into_response())
            }
        }
    } else {
        // Non-streaming response
        match openai_service
            .chat_completion(request.messages, Some(model.to_string()), request.temperature, request.max_tokens)
            .await
        {
            Ok(response) => Ok(Json(response).into_response()),
            Err(e) => {
                // Return error response
                let error_response = Json(serde_json::json!({
                    "error": {
                        "message": format!("OpenAI API error: {}", e),
                        "type": "api_error",
                        "code": "openai_error"
                    }
                }));
                Ok((StatusCode::INTERNAL_SERVER_ERROR, error_response).into_response())
            }
        }
    }
}

/// Handle Gemini requests
async fn handle_gemini_request(
    state: AppState,
    request: ChatCompletionRequest,
    model: &str,
) -> Result<Response, StatusCode> {
    // Check if Gemini service is available
    let gemini_service = match &state.gemini_service {
        Some(service) => service,
        None => {
            // Fallback to mock response if Gemini service is not configured
            return handle_fallback_response(request).await;
        }
    };

    // Check if streaming is requested
    if request.stream.unwrap_or(false) {
        // Streaming response
        let gemini_stream = gemini_service
            .chat_completion_stream(
                request.messages,
                Some(model.to_string()),
                request.temperature,
                request.max_tokens,
            )
            .await;

        let sse_stream = stream! {
            for await result in gemini_stream {
                match result {
                    Ok(chunk) => {
                        yield Ok::<Event, Box<dyn std::error::Error + Send + Sync>>(
                            Event::default().json_data(chunk)
                                .unwrap_or_else(|_| Event::default().data("serialization error"))
                        );
                    }
                    Err(e) => {
                        yield Ok::<Event, Box<dyn std::error::Error + Send + Sync>>(
                            Event::default().data(format!("error: {}", e))
                        );
                    }
                }
            }
        };

        let sse_response = Sse::new(sse_stream)
            .keep_alive(
                axum::response::sse::KeepAlive::new()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive-text"),
            )
            .into_response();

        // Add AI SDK compatible headers
        let mut response = sse_response;
        let headers = response.headers_mut();
        headers.insert("content-type", "text/event-stream".parse().unwrap());
        headers.insert("cache-control", "no-cache".parse().unwrap());
        headers.insert("connection", "keep-alive".parse().unwrap());
        headers.insert("x-vercel-ai-ui-message-stream", "v1".parse().unwrap());
        headers.insert("x-accel-buffering", "no".parse().unwrap());

        Ok(response)
    } else {
        // Non-streaming response
        match gemini_service
            .chat_completion(
                request.messages,
                Some(model.to_string()),
                request.temperature,
                request.max_tokens,
            )
            .await
        {
            Ok(response) => Ok(Json(response).into_response()),
            Err(e) => {
                // Return error response
                let error_response = Json(serde_json::json!({
                    "error": {
                        "message": format!("Gemini API error: {}", e),
                        "type": "api_error",
                        "code": "gemini_error"
                    }
                }));
                Ok((StatusCode::INTERNAL_SERVER_ERROR, error_response).into_response())
            }
        }
    }
}

/// Handle Anthropic requests
async fn handle_anthropic_request(
    state: AppState,
    request: ChatCompletionRequest,
    model: &str,
) -> Result<Response, StatusCode> {
    // Check if Anthropic service is available
    let anthropic_service = match &state.anthropic_service {
        Some(service) => service,
        None => {
            // Fallback to mock response if Anthropic service is not configured
            return handle_fallback_response(request).await;
        }
    };

    // Check if streaming is requested
    if request.stream.unwrap_or(false) {
        // Streaming response
        let anthropic_stream = anthropic_service
            .chat_completion_stream(
                request.messages,
                Some(model.to_string()),
                request.temperature,
                request.max_tokens,
            )
            .await;

        let sse_stream = stream! {
            for await result in anthropic_stream {
                match result {
                    Ok(chunk) => {
                        yield Ok::<Event, Box<dyn std::error::Error + Send + Sync>>(
                            Event::default().json_data(chunk)
                                .unwrap_or_else(|_| Event::default().data("serialization error"))
                        );
                    }
                    Err(e) => {
                        yield Ok::<Event, Box<dyn std::error::Error + Send + Sync>>(
                            Event::default().data(format!("error: {}", e))
                        );
                    }
                }
            }
        };

        let sse_response = Sse::new(sse_stream)
            .keep_alive(
                axum::response::sse::KeepAlive::new()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive-text"),
            )
            .into_response();

        // Add AI SDK compatible headers
        let mut response = sse_response;
        let headers = response.headers_mut();
        headers.insert("content-type", "text/event-stream".parse().unwrap());
        headers.insert("cache-control", "no-cache".parse().unwrap());
        headers.insert("connection", "keep-alive".parse().unwrap());
        headers.insert("x-vercel-ai-ui-message-stream", "v1".parse().unwrap());
        headers.insert("x-accel-buffering", "no".parse().unwrap());

        Ok(response)
    } else {
        // Non-streaming response
        match anthropic_service
            .chat_completion(
                request.messages,
                Some(model.to_string()),
                request.temperature,
                request.max_tokens,
            )
            .await
        {
            Ok(response) => Ok(Json(response).into_response()),
            Err(e) => {
                // Return error response
                let error_response = Json(serde_json::json!({
                    "error": {
                        "message": format!("Anthropic API error: {}", e),
                        "type": "api_error",
                        "code": "anthropic_error"
                    }
                }));
                Ok((StatusCode::INTERNAL_SERVER_ERROR, error_response).into_response())
            }
        }
    }
}

/// Fallback handler when OpenAI service is not available
async fn handle_fallback_response(request: ChatCompletionRequest) -> Result<Response, StatusCode> {
    // Use mock response when OpenAI is not configured
    let user_message = request.messages.last().and_then(|msg| {
        if matches!(msg.role, ChatRole::User) {
            Some(msg.content.clone())
        } else {
            None
        }
    });

    let response_content = user_message
        .map(|_| generate_mock_response().to_string())
        .unwrap_or_else(|| "Hello! How can I help you today? OpenAI API is not configured. Please set OPENAI_API_KEY environment variable.".to_string());

    if request.stream.unwrap_or(false) {
        let stream = create_streaming_response(response_content)
            .map(|result| result.map_err(|_| anyhow::anyhow!("Stream error")));

        let sse_response = Sse::new(stream)
            .keep_alive(
                axum::response::sse::KeepAlive::new()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive-text"),
            )
            .into_response();

        // Add AI SDK compatible headers
        let mut response = sse_response;
        let headers = response.headers_mut();
        headers.insert("content-type", "text/event-stream".parse().unwrap());
        headers.insert("cache-control", "no-cache".parse().unwrap());
        headers.insert("connection", "keep-alive".parse().unwrap());
        headers.insert("x-vercel-ai-ui-message-stream", "v1".parse().unwrap());
        headers.insert("x-accel-buffering", "no".parse().unwrap());

        Ok(response)
    } else {
        let response = ChatMessage {
            id: format!("msg_fallback_{}", fastrand::u64(1000..9999)),
            role: ChatRole::Assistant,
            content: response_content,
            created_at: Some(chrono::Utc::now()),
            attachments: None,
            metadata: Some(HashMap::from([
                ("warning".to_string(), serde_json::Value::String("OpenAI API not configured, using fallback response".to_string()))
            ])),
        };

        Ok(Json(response).into_response())
    }
}

/// Legacy API endpoint compatible with the existing frontend
pub async fn legacy_chat_handler(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Try to extract messages from the request
    let messages_array = request
        .get("messages")
        .and_then(|m| m.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if messages_array.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Convert to our ChatMessage format
    let chat_messages: Result<Vec<ChatMessage>, _> = messages_array
        .iter()
        .map(|msg| {
            let role = msg
                .get("role")
                .and_then(|r| r.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing role"))?;

            let content = msg
                .get("content")
                .and_then(|c| c.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

            let chat_role = match role {
                "user" => ChatRole::User,
                "assistant" => ChatRole::Assistant,
                "system" => ChatRole::System,
                _ => return Err(anyhow::anyhow!("Invalid role: {}", role)),
            };

            Ok(ChatMessage {
                id: msg.get("id")
                    .and_then(|id| id.as_str())
                    .unwrap_or(&format!("msg_{}", fastrand::u64(1000..9999)))
                    .to_string(),
                role: chat_role,
                content: content.to_string(),
                created_at: None,
                attachments: None,
                metadata: None,
            })
        })
        .collect();

    let chat_messages = chat_messages.map_err(|_| StatusCode::BAD_REQUEST)?;

    // Use OpenAI service if available
    if let Some(openai_service) = &state.openai_service {
        match openai_service.chat_completion(chat_messages, None, None, None).await {
            Ok(response) => {
                Ok(Json(serde_json::json!({
                    "role": "assistant",
                    "content": response.content
                })))
            }
            Err(e) => {
                // Fallback to mock on error
                let fallback_response = generate_mock_response();
                Ok(Json(serde_json::json!({
                    "role": "assistant",
                    "content": fallback_response,
                    "error": format!("OpenAI API error, using fallback: {}", e)
                })))
            }
        }
    } else {
        // Fallback to mock when OpenAI is not configured
        let response = generate_mock_response();
        Ok(Json(serde_json::json!({
            "role": "assistant",
            "content": response,
            "warning": "OpenAI API not configured, using fallback response"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use serde_json::json;
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        // Create test state without services to force fallback mode
        let state = crate::AppState {
            todos: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            openai_service: None, // Force fallback mode for tests
            gemini_service: None,  // Force fallback mode for tests
            anthropic_service: None, // Force fallback mode for tests
            database: None, // Force fallback mode for tests
        };
        Router::new()
            .route("/api/chat", post(super::legacy_chat_handler))
            .route("/api/v1/chat/completions", post(super::chat_completion))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_legacy_chat_endpoint() {
        let app = create_test_app();

        let request_body = json!({
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, how are you?"
                }
            ]
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/chat")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_chat_completion_endpoint() {
        let app = create_test_app();

        let request_body = json!({
            "messages": [
                {
                    "id": "msg1",
                    "role": "user",
                    "content": "Hello, how are you?"
                }
            ],
            "model": "gpt-3.5-turbo",
            "stream": false
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 200 even with fallback
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_empty_messages_error() {
        let app = create_test_app();

        let request_body = json!({
            "messages": []
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/chat")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_invalid_messages_format() {
        let app = create_test_app();

        let request_body = json!({
            "messages": "invalid"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/chat")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_gemini_model_routing() {
        let app = create_test_app();

        let request_body = json!({
            "messages": [
                {
                    "id": "msg1",
                    "role": "user",
                    "content": "Hello Gemini!"
                }
            ],
            "model": "gemini-1.5-flash",
            "stream": false
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 200 even with fallback (Gemini service not configured)
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_claude_model_routing() {
        let app = create_test_app();

        let request_body = json!({
            "messages": [
                {
                    "id": "msg1",
                    "role": "user",
                    "content": "Hello Claude!"
                }
            ],
            "model": "claude-3-5-sonnet-20241022",
            "stream": false
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 200 even with fallback (Anthropic service not configured)
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_provider_detection() {
        // Test OpenAI model detection
        assert_eq!(get_provider_from_model("gpt-4"), Provider::OpenAI);
        assert_eq!(get_provider_from_model("gpt-3.5-turbo"), Provider::OpenAI);

        // Test Gemini model detection
        assert_eq!(get_provider_from_model("gemini-1.5-flash"), Provider::Gemini);
        assert_eq!(get_provider_from_model("gemini-pro"), Provider::Gemini);
        assert_eq!(get_provider_from_model("models/gemini-1.5-pro"), Provider::Gemini);

        // Test Anthropic model detection
        assert_eq!(get_provider_from_model("claude-3-5-sonnet-20241022"), Provider::Anthropic);
        assert_eq!(get_provider_from_model("claude-3-opus-20240229"), Provider::Anthropic);
        assert_eq!(get_provider_from_model("claude-3-haiku-20240307"), Provider::Anthropic);

        // Test default to OpenAI
        assert_eq!(get_provider_from_model("unknown-model"), Provider::OpenAI);
    }
}