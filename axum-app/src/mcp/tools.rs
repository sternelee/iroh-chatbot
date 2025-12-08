use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::{client::MCPClient, MCPTool, MCPToolResult};

/// Manages multiple MCP clients and provides a unified interface for tool execution
#[derive(Debug)]
pub struct MCPToolManager {
    clients: Arc<RwLock<HashMap<String, Arc<MCPClient>>>>,
    tools_index: Arc<RwLock<HashMap<String, String>>>, // tool_name -> server_name mapping
}

impl MCPToolManager {
    /// Create a new MCP tool manager
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            tools_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an MCP client to the manager
    pub async fn add_client(&self, client: MCPClient) -> Result<()> {
        let server_name = client.server_name().to_string();
        let tools = client.get_tools().await;

        {
            let mut clients = self.clients.write().await;
            clients.insert(server_name.clone(), Arc::new(client));
        }

        {
            let mut tools_index = self.tools_index.write().await;
            for tool in tools {
                tools_index.insert(tool.name.clone(), server_name.clone());
                debug!("Registered tool {} from server {}", tool.name, server_name);
            }
        }

        info!("Added MCP client for server: {}", server_name);
        Ok(())
    }

    /// Remove an MCP client from the manager
    pub async fn remove_client(&self, server_name: &str) -> Result<()> {
        let tools_to_remove = {
            let mut tools_index = self.tools_index.write().await;
            tools_index
                .iter()
                .filter(|(_, server)| *server == server_name)
                .map(|(tool_name, _)| tool_name.clone())
                .collect::<Vec<_>>()
        };

        {
            let mut tools_index = self.tools_index.write().await;
            for tool_name in &tools_to_remove {
                tools_index.remove(tool_name);
            }
        }

        {
            let mut clients = self.clients.write().await;
            clients.remove(server_name);
        }

        info!("Removed MCP client for server: {} ({} tools)", server_name, tools_to_remove.len());
        Ok(())
    }

    /// Get all available tools from all connected MCP servers
    pub async fn get_all_tools(&self) -> Vec<MCPTool> {
        let mut all_tools = Vec::new();
        let clients = self.clients.read().await;

        for client in clients.values() {
            let tools = client.get_tools().await;
            all_tools.extend(tools);
        }

        all_tools
    }

    /// Execute a tool call by name (automatically finds the correct server)
    pub async fn call_tool(&self, tool_name: &str, arguments: HashMap<String, Value>) -> Result<MCPToolResult> {
        let tools_index = self.tools_index.read().await;
        let clients = self.clients.read().await;

        match tools_index.get(tool_name) {
            Some(server_name) => {
                match clients.get(server_name) {
                    Some(client) => {
                        debug!("Calling tool {} on server {}", tool_name, server_name);
                        client.call_tool(tool_name, arguments).await
                    }
                    None => {
                        error!("Server {} not found for tool {}", server_name, tool_name);
                        Err(anyhow!("Server {} not found for tool {}", server_name, tool_name))
                    }
                }
            }
            None => {
                warn!("Tool {} not found in any MCP server", tool_name);
                Err(anyhow!("Tool {} not found in any MCP server", tool_name))
            }
        }
    }

    /// Check if a tool is available
    pub async fn has_tool(&self, tool_name: &str) -> bool {
        let tools_index = self.tools_index.read().await;
        tools_index.contains_key(tool_name)
    }

    /// Get the server name for a specific tool
    pub async fn get_tool_server(&self, tool_name: &str) -> Option<String> {
        let tools_index = self.tools_index.read().await;
        tools_index.get(tool_name).cloned()
    }

    /// Get list of all connected servers
    pub async fn get_servers(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    /// Get tools from a specific server
    pub async fn get_server_tools(&self, server_name: &str) -> Vec<MCPTool> {
        let clients = self.clients.read().await;
        match clients.get(server_name) {
            Some(client) => client.get_tools().await,
            None => Vec::new(),
        }
    }

    /// Refresh tools from all connected servers
    pub async fn refresh_tools(&self) -> Result<()> {
        info!("Refreshing tools from all MCP servers");

        let servers: Vec<String> = {
            let clients = self.clients.read().await;
            clients.keys().cloned().collect()
        };

        // Clear the tools index
        {
            let mut tools_index = self.tools_index.write().await;
            tools_index.clear();
        }

        // Reload tools from each server
        for server_name in &servers {
            if let Some(client) = {
                let clients = self.clients.read().await;
                clients.get(server_name.as_str()).cloned()
            } {
                let tools = client.get_tools().await;
                {
                    let mut tools_index = self.tools_index.write().await;
                    for tool in tools {
                        tools_index.insert(tool.name.clone(), server_name.clone());
                        debug!("Re-registered tool {} from server {}", tool.name, server_name);
                    }
                }
            }
        }

        info!("Refresh completed MCP tools from {} servers", servers.len());
        Ok(())
    }

    /// Get statistics about the MCP tool manager
    pub async fn get_stats(&self) -> MCPToolManagerStats {
        let clients = self.clients.read().await;
        let tools_index = self.tools_index.read().await;

        let mut server_tools = std::collections::HashMap::new();
        for (name, client) in clients.iter() {
            let tool_count = client.get_tools().await.len();
            server_tools.insert((*name).to_string(), tool_count);
        }

        MCPToolManagerStats {
            total_servers: clients.len(),
            total_tools: tools_index.len(),
            server_tools,
        }
    }
}

/// Statistics for the MCP tool manager
#[derive(Debug, Clone)]
pub struct MCPToolManagerStats {
    pub total_servers: usize,
    pub total_tools: usize,
    pub server_tools: HashMap<String, usize>,
}

impl Default for MCPToolManager {
    fn default() -> Self {
        Self::new()
    }
}