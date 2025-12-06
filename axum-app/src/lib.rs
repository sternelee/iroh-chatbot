mod routes;
mod todo;
mod chat;
mod openai_service;
mod gemini_service;
mod anthropic_service;
mod database;

use axum::{
    routing::{get, post},
    Router,
};
use routes::{create_todo, delete_todo, list_todos, toggle_todo};
use chat::{chat_completion, legacy_chat_handler};
use openai_service::OpenAIService;
use gemini_service::GeminiService;
use anthropic_service::AnthropicService;
use database::ChatDatabase;
use std::sync::{Arc, Mutex};
use todo::Todo;
use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    openai_service: Option<Arc<OpenAIService>>,
    gemini_service: Option<Arc<GeminiService>>,
    anthropic_service: Option<Arc<AnthropicService>>,
    database: Option<Arc<ChatDatabase>>,
}

impl AppState {
    pub async fn new() -> Self {
        // Load environment variables
        let _ = dotenv();

        // Try to initialize OpenAI service from environment
        let openai_service = OpenAIService::from_env()
            .ok()
            .map(Arc::new);

        // Try to initialize Gemini service from environment
        let gemini_service = GeminiService::from_env()
            .ok()
            .map(Arc::new);

        // Try to initialize Anthropic service from environment
        let anthropic_service = AnthropicService::from_env()
            .ok()
            .map(Arc::new);

        // Try to initialize database from environment
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:./chat.db".to_string());

        let database = ChatDatabase::new(&database_url)
            .await
            .ok()
            .map(Arc::new);

        Self {
            todos: Arc::new(Mutex::new(Vec::new())),
            openai_service,
            gemini_service,
            anthropic_service,
            database,
        }
    }
}

pub async fn create_axum_app() -> Router {
    let state = AppState::new().await;

    Router::new()
        // Legacy Todo routes (keeping for backward compatibility)
        .route("/", get(list_todos))
        .route("/todo", post(create_todo))
        .route("/todo/{:id}/delete", post(delete_todo))
        .route("/todo/{:id}/toggle", post(toggle_todo))
        // Chat API routes
        .route("/api/chat", post(legacy_chat_handler))
        .route("/api/v1/chat/completions", post(chat_completion))
        .with_state(state)
}
