use anyhow::{anyhow, Result};
use rmcp::{
    model::{
        CallToolRequestParam, ClientCapabilities, Implementation,
        InitializeRequestParam,
    },
    service::{serve_client, RoleClient},
    transport::{SseClientTransport},
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::{MCPTool, MCPToolResult, MCPTransportType};

/// MCP client wrapper for connecting to MCP servers
pub struct MCPClient {
    service: rmcp::service::RunningService<RoleClient, ()>,
    server_name: String,
    tools: Arc<RwLock<HashMap<String, MCPTool>>>,
}

impl std::fmt::Debug for MCPClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MCPClient")
            .field("server_name", &self.server_name)
            .finish()
    }
}

impl MCPClient {
    /// Create a new MCP client
    pub async fn new(
        server_name: &str,
        transport_type: MCPTransportType,
        endpoint: &str,
    ) -> Result<Self> {
        info!("Creating MCP client for server: {}", server_name);

        let transport = match transport_type {
            MCPTransportType::ServerSentEvents => {
                SseClientTransport::start(endpoint).await?
            }
            _ => {
                return Err(anyhow!("Transport type {:?} not yet implemented", transport_type));
            }
        };

        // Create client service using serve_client
        let service = serve_client((), transport).await?;
        debug!("MCP client created");

        let mcp_client = Self {
            service,
            server_name: server_name.to_string(),
            tools: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load available tools
        mcp_client.load_tools().await?;

        Ok(mcp_client)
    }

    /// Load available tools from the MCP server
    async fn load_tools(&self) -> Result<()> {
        info!("Loading tools from MCP server: {}", self.server_name);

        let tools_result = self
            .service
            .list_tools(Default::default())
            .await
            .map_err(|e| anyhow!("Failed to list tools: {:?}", e))?;

        let tools = tools_result.tools;
        let mut tools_map = self.tools.write().await;

        for tool in tools {
            let mcp_tool = MCPTool {
                name: tool.name.to_string(),
                description: tool.description.unwrap_or_default().to_string(),
                parameters: super::MCPToolParameters {
                    type_: "object".to_string(),
                    properties: HashMap::new(),
                    required: Vec::new(),
                },
                server_name: self.server_name.clone(),
            };

            tools_map.insert(tool.name.to_string(), mcp_tool);
            debug!("Loaded tool: {}", tool.name);
        }

        info!("Loaded {} tools from MCP server: {}", tools_map.len(), self.server_name);

        Ok(())
    }

    /// Execute a tool call
    pub async fn call_tool(&self, tool_name: &str, arguments: HashMap<String, Value>) -> Result<MCPToolResult> {
        debug!("Calling tool {} on MCP server: {}", tool_name, self.server_name);

        // Convert HashMap to serde_json::Map for the API
        let args_map: serde_json::Map<String, Value> = arguments.into_iter()
            .map(|(k, v)| (k, v))
            .collect();

        let tool_call_params = CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments: Some(args_map),
        };

        let result = self
            .service
            .call_tool(tool_call_params)
            .await
            .map_err(|e| anyhow!("Failed to call tool {}: {:?}", tool_name, e))?;

        // Extract the result from the content
        let content_result = result.content
            .first()
            .map(|c| {
                // For now, just serialize the content as JSON
                serde_json::to_value(c).unwrap_or(serde_json::Value::Null)
            });

        Ok(MCPToolResult {
            success: true,
            result: content_result,
            error: None,
        })
    }

    /// Get available tools
    pub async fn get_tools(&self) -> Vec<MCPTool> {
        self.tools.read().await.values().cloned().collect()
    }

    
    /// Get the server name
    pub fn server_name(&self) -> &str {
        &self.server_name
    }
}


/// Factory function to create MCP clients from configuration
pub async fn create_mcp_client(
    server_config: &super::MCPServerConfig,
) -> Result<MCPClient> {
    MCPClient::new(
        &server_config.name,
        server_config.transport_type.clone(),
        &server_config.endpoint,
    )
    .await
}