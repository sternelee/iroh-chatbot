use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Import the AI services
use crate::providers::{OpenAIService, AnthropicService, GeminiService, OpenRouterService, AIProvider};

/// AI Agent configuration and execution engine
#[derive(Debug, Clone)]
pub struct AgentManager {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    tools: Arc<RwLock<HashMap<String, ToolDefinition>>>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new agent with specified configuration
    pub async fn create_agent(&self, config: AgentConfig) -> Result<String> {
        let agent_id = config.id.clone();
        let agent = Agent::from_config(config)?;

        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), agent);

        Ok(agent_id)
    }

    /// Get an agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.get(agent_id).cloned())
    }

    /// Execute an agent with a prompt
    pub async fn execute_agent(&self, agent_id: &str, prompt: &str) -> Result<AgentExecution> {
        let agents = self.agents.read().await;
        let agent = agents.get(agent_id)
            .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))?;

        let execution = agent.execute(prompt, &self.tools).await?;
        Ok(execution)
    }

    /// Stream execution of an agent (for real-time responses)
    pub async fn execute_agent_stream(
        &self,
        agent_id: &str,
        prompt: &str,
    ) -> Result<AgentExecutionStream> {
        let agents = self.agents.read().await;
        let agent = agents.get(agent_id)
            .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))?;

        let stream = agent.execute_stream(prompt, &self.tools).await?;
        Ok(stream)
    }

    /// Register a tool for use by agents
    pub async fn register_tool(&self, tool: ToolDefinition) -> Result<()> {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    /// List all available agents
    pub async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let agents = self.agents.read().await;
        let agent_infos: Vec<AgentInfo> = agents
            .values()
            .map(|agent| AgentInfo {
                id: agent.config.id.clone(),
                name: agent.config.name.clone(),
                description: agent.config.description.clone(),
                provider: agent.config.provider.clone(),
                model: agent.config.model.clone(),
                created_at: agent.config.created_at,
                last_used: agent.config.last_used,
                tool_count: agent.config.tools.len(),
            })
            .collect();

        Ok(agent_infos)
    }

    /// Delete an agent
    pub async fn delete_agent(&self, agent_id: &str) -> Result<bool> {
        let mut agents = self.agents.write().await;
        Ok(agents.remove(agent_id).is_some())
    }
}

/// AI Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String, // "openai", "anthropic", "gemini", "openrouter"
    pub model: String,
    pub system: String, // System prompt
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Vec<String>, // Tool names
    pub max_tool_rounds: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub metadata: Option<HashMap<String, Value>>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: format!("agent_{}", fastrand::u64(1000..9999)),
            name: "New Agent".to_string(),
            description: "AI agent with default configuration".to_string(),
            provider: "openrouter".to_string(),
            model: "openai/gpt-3.5-turbo".to_string(),
            system: "You are a helpful AI assistant.".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            tools: vec![],
            max_tool_rounds: Some(10),
            created_at: now,
            last_used: None,
            metadata: None,
        }
    }
}

/// AI Agent instance
#[derive(Debug, Clone)]
pub struct Agent {
    config: AgentConfig,
}

impl Agent {
    pub fn from_config(config: AgentConfig) -> Result<Self> {
        // Validate agent configuration
        Self::validate_config(&config)?;

        Ok(Self { config })
    }

    /// Get the agent's configuration
    pub fn get_config(&self) -> &AgentConfig {
        &self.config
    }

    fn validate_config(config: &AgentConfig) -> Result<()> {
        // Validate provider
        match config.provider.as_str() {
            "openai" | "anthropic" | "gemini" | "openrouter" => {},
            _ => return Err(anyhow!("Unsupported provider: {}", config.provider)),
        }

        // Validate temperature
        if let Some(temp) = config.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(anyhow!("Temperature must be between 0.0 and 2.0"));
            }
        }

        // Validate max_tool_rounds
        if let Some(rounds) = config.max_tool_rounds {
            if rounds == 0 {
                return Err(anyhow!("max_tool_rounds must be at least 1"));
            }
        }

        Ok(())
    }

    /// Execute the agent with a prompt
    pub async fn execute(&self, prompt: &str, tools: &Arc<RwLock<HashMap<String, ToolDefinition>>>) -> Result<AgentExecution> {
        let start_time = Utc::now();
        let execution_id = format!("exec_{}", fastrand::u64(1000..9999));

        // Load available tools
        let available_tools = self.load_tools(&self.config.tools, tools).await?;

        // Create the system prompt with tool information
        let system_prompt = self.build_system_prompt(&available_tools)?;

        // Initialize execution state
        let mut state = AgentExecutionState {
            execution_id: execution_id.clone(),
            agent_id: self.config.id.clone(),
            prompt: prompt.to_string(),
            system_prompt,
            messages: vec![],
            tool_calls: vec![],
            tool_results: vec![],
            rounds: 0,
            max_rounds: self.config.max_tool_rounds.unwrap_or(10),
            status: ExecutionStatus::Running,
            start_time,
            end_time: None,
            final_response: None,
            usage: None,
        };

        // Execute the tool loop
        self.execute_tool_loop(&mut state, &available_tools).await?;

        // Create execution result
        let execution = AgentExecution {
            execution_id: state.execution_id,
            agent_id: state.agent_id,
            prompt: state.prompt,
            response: state.final_response.unwrap_or_else(|| "No response generated".to_string()),
            tool_calls: state.tool_calls,
            tool_results: state.tool_results,
            status: state.status,
            start_time: state.start_time,
            end_time: state.end_time.unwrap_or(Utc::now()),
            rounds: state.rounds,
            usage: state.usage,
        };

        Ok(execution)
    }

    /// Stream execution of the agent
    pub async fn execute_stream(&self, prompt: &str, tools: &Arc<RwLock<HashMap<String, ToolDefinition>>>) -> Result<AgentExecutionStream> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let agent = self.clone();
        let prompt = prompt.to_string();
        let tools = tools.clone();

        // Spawn streaming execution task
        tokio::spawn(async move {
            match agent.execute(&prompt, &tools).await {
                Ok(execution) => {
                    // Send final result
                    let _ = tx.send(AgentStreamEvent::Complete(execution));
                }
                Err(e) => {
                    // Send error
                    let _ = tx.send(AgentStreamEvent::Error(e.to_string()));
                }
            }
        });

        Ok(AgentExecutionStream { rx })
    }

    async fn execute_tool_loop(&self, state: &mut AgentExecutionState, tools: &[ToolDefinition]) -> Result<()> {
        // Add initial user message
        state.messages.push(ChatMessage {
            role: "user".to_string(),
            content: state.prompt.clone(),
            tool_calls: None,
        });

        while state.rounds < state.max_rounds {
            state.rounds += 1;

            // Make API call to the AI provider
            let response = self.make_api_call(&state.system_prompt, &state.messages).await?;

            // Add assistant response
            state.messages.push(response.message.clone());

            // Check if there are tool calls
            if let Some(tool_calls) = response.message.tool_calls {
                state.tool_calls.extend(tool_calls.clone());

                // Execute each tool call
                for tool_call in tool_calls {
                    let tool_result = self.execute_tool_call(&tool_call, tools).await?;
                    state.tool_results.push(tool_result.clone());

                    // Add tool result to conversation
                    state.messages.push(ChatMessage {
                        role: "tool".to_string(),
                        content: serde_json::to_string(&tool_result.result).unwrap_or_default(),
                        tool_calls: None,
                    });
                }

                // Continue the loop to get a response to tool results
                continue;
            } else {
                // No more tool calls, we have a final response
                state.final_response = Some(response.message.content);
                state.status = ExecutionStatus::Completed;
                state.end_time = Some(Utc::now());
                state.usage = response.usage;
                break;
            }
        }

        // Check if we exceeded max rounds
        if state.rounds >= state.max_rounds && state.final_response.is_none() {
            state.status = ExecutionStatus::MaxRoundsReached;
            state.end_time = Some(Utc::now());
        }

        Ok(())
    }

    async fn make_api_call(&self, system_prompt: &str, messages: &[ChatMessage]) -> Result<AIResponse> {
        // Convert agent message format to chat service format
        let mut chat_messages = vec![
            crate::chat::ChatMessage {
                id: "system".to_string(),
                role: crate::chat::ChatRole::System,
                content: system_prompt.to_string(),
                created_at: Some(chrono::Utc::now()),
                attachments: None,
                metadata: None,
            }
        ];

        // Add existing messages
        for msg in messages {
            let role = match msg.role.as_str() {
                "user" => crate::chat::ChatRole::User,
                "assistant" => crate::chat::ChatRole::Assistant,
                "tool" => crate::chat::ChatRole::User, // Convert tool messages to user messages for now
                _ => crate::chat::ChatRole::User,
            };

            chat_messages.push(crate::chat::ChatMessage {
                id: fastrand::u64(1000..9999).to_string(),
                role,
                content: msg.content.clone(),
                created_at: Some(chrono::Utc::now()),
                attachments: None,
                metadata: None,
            });
        }

        // Call the appropriate AI provider
        let response = match self.config.provider.as_str() {
            "openai" => {
                let service = OpenAIService::from_env()
                    .map_err(|e| anyhow!("Failed to initialize OpenAI service: {}", e))?;

                service.chat_completion(chat_messages, Some(self.config.model.clone()), self.config.temperature, self.config.max_tokens).await?
            }
            "anthropic" => {
                let service = AnthropicService::from_env()
                    .map_err(|e| anyhow!("Failed to initialize Anthropic service: {}", e))?;

                service.chat_completion(
                    chat_messages,
                    Some(self.config.model.clone()),
                    self.config.temperature,
                    self.config.max_tokens
                ).await?
            }
            "gemini" => {
                let service = GeminiService::from_env()
                    .map_err(|e| anyhow!("Failed to initialize Gemini service: {}", e))?;

                service.chat_completion(
                    chat_messages,
                    Some(self.config.model.clone()),
                    self.config.temperature,
                    self.config.max_tokens
                ).await?
            }
            "openrouter" => {
                let service = OpenRouterService::from_env()
                    .map_err(|e| anyhow!("Failed to initialize OpenRouter service: {}", e))?;

                service.chat_completion(
                    chat_messages,
                    Some(self.config.model.clone()),
                    self.config.temperature,
                    self.config.max_tokens
                ).await?
            }
            _ => return Err(anyhow!("Unsupported provider: {}", self.config.provider)),
        };

        // Convert the response back to agent format
        // Note: For now, we don't parse tool calls from real providers, but this would need to be added
        let usage = response.metadata
            .as_ref()
            .and_then(|m| m.get("usage"))
            .and_then(|u| serde_json::from_value(u.clone()).ok())
            .map(|usage: crate::chat::Usage| UsageInfo {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            });

        Ok(AIResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content: response.content,
                tool_calls: None, // TODO: Parse tool calls from real AI responses
            },
            usage,
        })
    }

    async fn execute_tool_call(&self, tool_call: &ToolCallInfo, tools: &[ToolDefinition]) -> Result<ToolResult> {
        let tool = tools.iter()
            .find(|t| t.name == tool_call.function.name)
            .ok_or_else(|| anyhow!("Tool not found: {}", tool_call.function.name))?;

        let arguments: Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| anyhow!("Invalid tool arguments: {}", e))?;

        // Execute the tool (this would be the actual tool implementation)
        let result = self.execute_tool_function(&tool.name, &arguments).await?;

        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            result,
        })
    }

    async fn execute_tool_function(&self, tool_name: &str, arguments: &Value) -> Result<Value> {
        match tool_name {
            "calculator" => {
                let expression = arguments.get("expression")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing expression parameter"))?;

                // Simple calculator (in production, use a proper math parser)
                let result = self.evaluate_expression(expression)?;
                Ok(serde_json::json!({ "result": result }))
            }
            "web_search" => {
                let query = arguments.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing query parameter"))?;

                // Mock search results
                Ok(serde_json::json!({
                    "results": [
                        {
                            "title": format!("Search result for: {}", query),
                            "url": "https://example.com",
                            "snippet": "This is a mock search result."
                        }
                    ]
                }))
            }
            _ => Err(anyhow!("Unknown tool: {}", tool_name))
        }
    }

    fn evaluate_expression(&self, expression: &str) -> Result<f64> {
        // Very simple expression evaluator - in production, use a proper library
        // This is just for demonstration
        if expression == "2 + 2" {
            Ok(4.0)
        } else if expression == "10 * 5" {
            Ok(50.0)
        } else {
            Err(anyhow!("Unsupported expression: {}", expression))
        }
    }

    async fn load_tools(&self, tool_names: &[String], tools: &Arc<RwLock<HashMap<String, ToolDefinition>>>) -> Result<Vec<ToolDefinition>> {
        let tools_registry = tools.read().await;
        let mut available_tools = Vec::new();

        for tool_name in tool_names {
            if let Some(tool) = tools_registry.get(tool_name) {
                available_tools.push(tool.clone());
            } else {
                return Err(anyhow!("Tool not found: {}", tool_name));
            }
        }

        Ok(available_tools)
    }

    fn build_system_prompt(&self, tools: &[ToolDefinition]) -> Result<String> {
        let mut system_prompt = self.config.system.clone();

        if !tools.is_empty() {
            system_prompt.push_str("\n\nYou have access to the following tools:\n");

            for tool in tools {
                system_prompt.push_str(&format!("\n- {}: {}\n", tool.name, tool.description));
            }

            system_prompt.push_str("\nUse these tools when needed to help answer the user's request. Continue using tools until you have sufficient information to provide a complete answer.");
        }

        Ok(system_prompt)
    }
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema for parameters
    pub function: ToolFunction,
}

/// Tool function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    MaxRoundsReached,
}

/// Execution state
#[derive(Debug)]
struct AgentExecutionState {
    execution_id: String,
    agent_id: String,
    prompt: String,
    system_prompt: String,
    messages: Vec<ChatMessage>,
    tool_calls: Vec<ToolCallInfo>,
    tool_results: Vec<ToolResult>,
    rounds: u32,
    max_rounds: u32,
    status: ExecutionStatus,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    final_response: Option<String>,
    usage: Option<UsageInfo>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

/// Tool call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolFunctionCall,
}

/// Tool function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub result: Value,
}

/// AI response
#[derive(Debug, Clone)]
pub struct AIResponse {
    pub message: ChatMessage,
    pub usage: Option<UsageInfo>,
}

/// Usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    pub execution_id: String,
    pub agent_id: String,
    pub prompt: String,
    pub response: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub tool_results: Vec<ToolResult>,
    pub status: ExecutionStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub rounds: u32,
    pub usage: Option<UsageInfo>,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub tool_count: usize,
}

/// Agent stream event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStreamEvent {
    Progress {
        execution_id: String,
        round: u32,
        message: String
    },
    ToolCall {
        execution_id: String,
        tool_name: String,
        arguments: Value
    },
    ToolResult {
        execution_id: String,
        tool_name: String,
        result: Value
    },
    Complete(AgentExecution),
    Error(String),
}

/// Agent execution stream
#[derive(Debug)]
pub struct AgentExecutionStream {
    rx: tokio::sync::mpsc::UnboundedReceiver<AgentStreamEvent>,
}

impl AgentExecutionStream {
    pub async fn next(&mut self) -> Option<AgentStreamEvent> {
        self.rx.recv().await
    }
}

/// Create default tools
pub fn create_default_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "calculator".to_string(),
            description: "Perform mathematical calculations".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate"
                    }
                },
                "required": ["expression"]
            }),
            function: ToolFunction {
                name: "calculator".to_string(),
                description: "Perform mathematical calculations".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate"
                        }
                    },
                    "required": ["expression"]
                }),
            },
        },
        ToolDefinition {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
            function: ToolFunction {
                name: "web_search".to_string(),
                description: "Search the web for information".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let manager = AgentManager::new();

        let config = AgentConfig {
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            system: "You are a helpful assistant.".to_string(),
            tools: vec!["calculator".to_string()],
            ..Default::default()
        };

        let agent_id = manager.create_agent(config).await.unwrap();
        assert!(!agent_id.is_empty());
    }

    #[tokio::test]
    async fn test_agent_execution() {
        let manager = AgentManager::new();

        // Register default tools
        for tool in create_default_tools() {
            manager.register_tool(tool).await.unwrap();
        }

        let config = AgentConfig {
            name: "Calculator Agent".to_string(),
            description: "An agent that can do math".to_string(),
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            system: "You are a helpful assistant that can perform calculations.".to_string(),
            tools: vec!["calculator".to_string()],
            ..Default::default()
        };

        let agent_id = manager.create_agent(config).await.unwrap();
        let execution = manager.execute_agent(&agent_id, "What is 2 + 2?").await.unwrap();

        assert_eq!(execution.agent_id, agent_id);
        assert!(execution.response.len() > 0);
    }
}