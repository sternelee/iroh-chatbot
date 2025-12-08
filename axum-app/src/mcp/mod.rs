//! Model Context Protocol (MCP) Integration
//!
//! This module provides integration with MCP servers to enable AI agents
//! to use external tools and services through the standardized MCP protocol.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod client;
pub mod tools;
pub mod server;
pub mod config;

// Re-export main MCP components
pub use client::MCPClient;
pub use tools::MCPToolManager;
pub use server::MCPServerManager;
pub use config::{MCPConfigManager, MCPServerConfigFile, MCPGlobalConfig, MCPAuthConfigFile};

/// MCP tool definition compatible with agent tool system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub parameters: MCPToolParameters,
    pub server_name: String,
}

/// MCP tool parameters schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolParameters {
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub required: Vec<String>,
}

/// MCP tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolResult {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// MCP configuration for connecting to servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub servers: Vec<MCPServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub name: String,
    pub transport_type: MCPTransportType,
    pub endpoint: String,
    pub auth: Option<MCPAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MCPTransportType {
    #[serde(rename = "sse")]
    ServerSentEvents,
    #[serde(rename = "http")]
    StreamableHttp,
    #[serde(rename = "stdio")]
    ChildProcess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MCPAuthConfig {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "api_key")]
    ApiKey {
        key: String,
        #[serde(default = "default_auth_header")]
        header: String,
    },
    #[serde(rename = "bearer")]
    Bearer {
        token: String,
    },
    #[serde(rename = "basic")]
    Basic {
        username: String,
        password: String,
    },
}

fn default_auth_header() -> String {
    "X-API-Key".to_string()
}

impl Default for MCPToolParameters {
    fn default() -> Self {
        Self {
            type_: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }
}

/// Convert MCP tool to agent tool format
impl From<MCPTool> for crate::agent::ToolDefinition {
    fn from(mcp_tool: MCPTool) -> Self {
        let properties_clone = mcp_tool.parameters.properties.clone();
        crate::agent::ToolDefinition {
            name: mcp_tool.name.clone(),
            description: mcp_tool.description.clone(),
            parameters: serde_json::Value::Object(properties_clone.clone().into_iter().collect()),
            function: crate::agent::ToolFunction {
                name: mcp_tool.name.clone(),
                description: mcp_tool.description.clone(),
                parameters: serde_json::json!({
                    "type": mcp_tool.parameters.type_,
                    "properties": properties_clone,
                    "required": mcp_tool.parameters.required
                }),
            },
        }
    }
}