use crate::AppState;
use crate::agent::{AgentConfig, AgentExecution, ExecutionStatus};
use axum::{
    extract::{Path, State},
    http::{StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Create agent API routes
pub fn agent_routes() -> Router<AppState> {
    Router::new()
        // Agent CRUD operations
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/{agent_id}", get(get_agent).delete(delete_agent))
        .route("/agents/{agent_id}/execute", post(execute_agent))
        // Tool management
        .route("/agents/{agent_id}/tools", get(list_agent_tools))
        .route("/tools", get(list_tools))
        // useCompletion compatibility endpoint
        .route("/agent/{agent_id}", post(use_completion))
}

/// Create a new agent
pub async fn create_agent(
    State(state): State<AppState>,
    Json(request): Json<CreateAgentRequest>,
) -> Result<Json<CreateAgentResponse>, StatusCode> {
    let config = AgentConfig {
        id: request.id.unwrap_or_else(|| format!("agent_{}", fastrand::u64(1000..9999))),
        name: request.name,
        description: request.description.unwrap_or_default(),
        provider: request.provider.unwrap_or_else(|| "openai".to_string()),
        model: request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
        system: request.system.unwrap_or_else(|| "You are a helpful AI assistant.".to_string()),
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        tools: request.tools.unwrap_or_default(),
        max_tool_rounds: request.max_tool_rounds,
        created_at: chrono::Utc::now(),
        last_used: None,
        metadata: request.metadata,
    };

    match state.agent_manager.create_agent(config).await {
        Ok(agent_id) => Ok(Json(CreateAgentResponse {
            success: true,
            agent_id,
            message: "Agent created successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// List all agents
pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<ListAgentsResponse>, StatusCode> {
    match state.agent_manager.list_agents().await {
        Ok(agents) => {
            let count = agents.len();
            Ok(Json(ListAgentsResponse {
                success: true,
                agents,
                count,
            }))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get a specific agent
pub async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<GetAgentResponse>, StatusCode> {
    match state.agent_manager.get_agent(&agent_id).await {
        Ok(Some(agent)) => Ok(Json(GetAgentResponse {
            success: true,
            agent: agent.get_config().clone(),
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Delete an agent
pub async fn delete_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<DeleteAgentResponse>, StatusCode> {
    match state.agent_manager.delete_agent(&agent_id).await {
        Ok(deleted) => {
            if deleted {
                Ok(Json(DeleteAgentResponse {
                    success: true,
                    message: "Agent deleted successfully".to_string(),
                }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Execute an agent with a prompt
pub async fn execute_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(request): Json<ExecuteAgentRequest>,
) -> Result<Json<ExecuteAgentResponse>, StatusCode> {
    match state.agent_manager.execute_agent(&agent_id, &request.prompt).await {
        Ok(execution) => Ok(Json(ExecuteAgentResponse {
            success: true,
            execution,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// useCompletion compatible endpoint
/// This mimics the useCompletion hook API from the AI SDK
pub async fn use_completion(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(request): Json<UseCompletionRequest>,
) -> Result<Response, StatusCode> {
    // Non-streaming response
    match state.agent_manager.execute_agent(&agent_id, &request.prompt).await {
            Ok(execution) => {
                let response = UseCompletionResponse {
                    completion: execution.response,
                    usage: execution.usage.map(|u| UseCompletionUsage {
                        prompt_tokens: u.prompt_tokens,
                        completion_tokens: u.completion_tokens,
                        total_tokens: u.total_tokens,
                    }),
                    tool_calls: execution.tool_calls.into_iter().map(|tc| UseCompletionToolCall {
                        id: tc.id,
                        type_: tc.call_type,
                        function: UseCompletionFunction {
                            name: tc.function.name,
                            arguments: tc.function.arguments,
                        },
                    }).collect(),
                    finish_reason: match execution.status {
                        ExecutionStatus::Completed => "stop".to_string(),
                        ExecutionStatus::MaxRoundsReached => "length".to_string(),
                        ExecutionStatus::Failed => "error".to_string(),
                        _ => "unknown".to_string(),
                    },
                    agent_id: Some(execution.agent_id),
                    execution_id: Some(execution.execution_id),
                };

                Ok(Json(response).into_response())
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// List tools for a specific agent
pub async fn list_agent_tools(
    State(_state): State<AppState>,
    Path(_agent_id): Path<String>,
) -> Result<Json<ListToolsResponse>, StatusCode> {
    // For now, return default tools
    let default_tools = crate::agent::create_default_tools();
    let count = default_tools.len();
    let tools = default_tools.into_iter().map(|tool| ToolInfo {
        name: tool.name,
        description: tool.description,
        parameters: tool.parameters,
    }).collect();

    Ok(Json(ListToolsResponse {
        success: true,
        tools,
        count,
    }))
}

/// List all available tools
pub async fn list_tools(
    State(_state): State<AppState>,
) -> Result<Json<ListToolsResponse>, StatusCode> {
    let default_tools = crate::agent::create_default_tools();
    let count = default_tools.len();
    let tools = default_tools.into_iter().map(|tool| ToolInfo {
        name: tool.name,
        description: tool.description,
        parameters: tool.parameters,
    }).collect();

    Ok(Json(ListToolsResponse {
        success: true,
        tools,
        count,
    }))
}

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub system: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<String>>,
    pub max_tool_rounds: Option<u32>,
    pub metadata: Option<std::collections::HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct CreateAgentResponse {
    pub success: bool,
    pub agent_id: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ListAgentsResponse {
    pub success: bool,
    pub agents: Vec<crate::agent::AgentInfo>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct GetAgentResponse {
    pub success: bool,
    pub agent: AgentConfig,
}

#[derive(Debug, Serialize)]
pub struct DeleteAgentResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteAgentRequest {
    pub prompt: String,
    pub parameters: Option<std::collections::HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteAgentResponse {
    pub success: bool,
    pub execution: AgentExecution,
}

#[derive(Debug, Deserialize)]
pub struct UseCompletionRequest {
    pub prompt: String,
    pub stream: Option<bool>,
    pub parameters: Option<std::collections::HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct UseCompletionResponse {
    pub completion: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UseCompletionUsage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<UseCompletionToolCall>,
    pub finish_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UseCompletionUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct UseCompletionToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: UseCompletionFunction,
}

#[derive(Debug, Serialize)]
pub struct UseCompletionFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize)]
pub struct ListToolsResponse {
    pub success: bool,
    pub tools: Vec<ToolInfo>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

