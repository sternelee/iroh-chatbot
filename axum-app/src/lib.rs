mod routes;
mod todo;
mod chat;
mod providers;
mod database;
mod agent;
mod agent_api;
mod mcp;

use axum::{
    routing::{get, post},
    Router,
};
use routes::{create_todo, delete_todo, list_todos, toggle_todo};
use chat::{chat_completion, legacy_chat_handler};
use providers::{OpenAIService, GeminiService, AnthropicService};
use database::ChatDatabase;
use agent::AgentManager;
use agent_api::agent_routes;
use mcp::{MCPToolManager, MCPServerManager};
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
    agent_manager: Arc<AgentManager>,
    mcp_tool_manager: Arc<MCPToolManager>,
    mcp_server_manager: Arc<MCPServerManager>,
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

        // Initialize agent manager and register default tools
        let agent_manager = Arc::new(AgentManager::new());
        for tool in agent::create_default_tools() {
            let _ = agent_manager.register_tool(tool).await;
        }

        // Initialize MCP tool manager and server manager
        let mcp_tool_manager = Arc::new(MCPToolManager::new());
        let mcp_server_manager = match MCPServerManager::new().await {
            Ok(manager) => {
                let arc_manager = Arc::new(manager);
                // Try to initialize MCP servers from configuration
                // This will connect to configured MCP servers and load their tools
                if let Err(e) = arc_manager.initialize().await {
                    tracing::warn!("Failed to initialize MCP servers: {}", e);
                }
                arc_manager
            }
            Err(e) => {
                tracing::error!("Failed to create MCP server manager: {}", e);
                // Create a dummy server manager that won't connect to any servers
                // This prevents the app from crashing if MCP config is invalid
                Arc::new(MCPServerManager::with_config_path("nonexistent.json").await.unwrap())
            }
        };

        Self {
            todos: Arc::new(Mutex::new(Vec::new())),
            openai_service,
            gemini_service,
            anthropic_service,
            database,
            agent_manager,
            mcp_tool_manager,
            mcp_server_manager,
        }
    }
}

pub async fn create_axum_app() -> Router {
    let state = AppState::new().await;

    Router::new()
        // Legacy Todo routes (keeping for backward compatibility)
        .route("/", get(list_todos))
        .route("/todo", post(create_todo))
        .route("/todo/{id}/delete", post(delete_todo))
        .route("/todo/{id}/toggle", post(toggle_todo))
        // Chat API routes
        .route("/api/chat", post(legacy_chat_handler))
        .route("/api/v1/chat/completions", post(chat_completion))
        // Agent API routes
        .nest("/api", agent_routes())
        .with_state(state)
}
