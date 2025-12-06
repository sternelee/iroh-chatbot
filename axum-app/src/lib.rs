mod routes;
mod todo;
mod chat;
mod openai_service;

use axum::{
    routing::{get, post},
    Router,
};
use routes::{create_todo, delete_todo, list_todos, toggle_todo};
use chat::{chat_completion, legacy_chat_handler};
use openai_service::OpenAIService;
use std::sync::{Arc, Mutex};
use todo::Todo;

#[derive(Debug, Clone)]
pub struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    openai_service: Option<Arc<OpenAIService>>,
}

impl Default for AppState {
    fn default() -> Self {
        // Try to initialize OpenAI service from environment
        let openai_service = OpenAIService::from_env()
            .ok()
            .map(Arc::new);

        Self {
            todos: Arc::new(Mutex::new(Vec::new())),
            openai_service,
        }
    }
}

pub fn create_axum_app() -> Router {
    let state = AppState::default();

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
