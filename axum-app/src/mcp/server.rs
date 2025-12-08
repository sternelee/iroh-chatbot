use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::{MCPServerConfig, MCPTransportType};
use super::client::{create_mcp_client, MCPClient};
use super::config::MCPConfigManager;

/// MCP Server connection manager
#[derive(Debug)]
pub struct MCPServerManager {
    servers: Arc<RwLock<HashMap<String, Arc<MCPClient>>>>,
    config_manager: Arc<RwLock<MCPConfigManager>>,
}

impl MCPServerManager {
    /// Create a new MCP server manager
    pub async fn new() -> Result<Self> {
        let config_path = std::env::var("MCP_CONFIG_PATH")
            .unwrap_or_else(|_| "mcp.json".to_string());

        let config_manager = MCPConfigManager::new(&config_path).await?;

        Ok(Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            config_manager: Arc::new(RwLock::new(config_manager)),
        })
    }

    /// Create a new MCP server manager with custom config path
    pub async fn with_config_path<P: AsRef<std::path::Path>>(config_path: P) -> Result<Self> {
        let config_manager = MCPConfigManager::new(config_path).await?;

        Ok(Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            config_manager: Arc::new(RwLock::new(config_manager)),
        })
    }

    /// Initialize the server manager and connect to all configured servers
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing MCP server manager");

        let configs = {
            let config_manager = self.config_manager.read().await;
            config_manager.get_enabled_servers()
        };

        if configs.is_empty() {
            info!("No enabled MCP servers found in configuration");
            return Ok(());
        }

        info!("Found {} enabled MCP servers", configs.len());

        // Connect to all servers
        for server_config in &configs {
            info!("Attempting to connect to MCP server: {}", server_config.name);
            match create_mcp_client(server_config).await {
                Ok(client) => {
                    let mut servers = self.servers.write().await;
                    servers.insert(server_config.name.clone(), Arc::new(client));
                    info!("Successfully connected to MCP server: {}", server_config.name);
                }
                Err(e) => {
                    warn!("Failed to connect to MCP server {}: {}", server_config.name, e);
                }
            }
        }

        let servers = self.servers.read().await;
        info!("MCP server manager initialized with {} servers", servers.len());
        Ok(())
    }

    /// Reload configuration and reconnect servers
    pub async fn reload_config(&self) -> Result<()> {
        info!("Reloading MCP configuration");

        {
            let mut config_manager = self.config_manager.write().await;
            config_manager.reload().await?;
        }

        // Disconnect existing servers
        {
            let mut servers = self.servers.write().await;
            servers.clear();
        }

        // Reconnect with new configuration
        self.initialize().await?;

        info!("MCP configuration reloaded successfully");
        Ok(())
    }

    /// Get reference to the tool manager
    pub fn tool_manager(&self) -> &super::MCPToolManager {
        // This is a bit awkward - in a real implementation, we'd have a proper
        // tool manager that manages tools across all servers
        static EMPTY_TOOL_MANAGER: std::sync::OnceLock<super::MCPToolManager> = std::sync::OnceLock::new();
        EMPTY_TOOL_MANAGER.get_or_init(super::MCPToolManager::new)
    }

    /// Get all connected servers
    pub async fn get_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }

    /// Get a specific server client
    pub async fn get_server(&self, name: &str) -> Option<Arc<MCPClient>> {
        let servers = self.servers.read().await;
        servers.get(name).cloned()
    }

    /// Get configuration statistics
    pub async fn get_config_stats(&self) -> super::config::MCPConfigStats {
        let config_manager = self.config_manager.read().await;
        config_manager.get_stats()
    }

    /// Get servers by tag
    pub async fn get_servers_by_tag(&self, tag: &str) -> Vec<String> {
        let config_manager = self.config_manager.read().await;
        config_manager.get_servers_by_tag(tag)
            .into_iter()
            .map(|config| config.name)
            .collect()
    }

    /// Check if server is enabled
    pub async fn is_server_enabled(&self, name: &str) -> bool {
        let config_manager = self.config_manager.read().await;
        config_manager.get_server_by_name(name).is_some()
    }

    /// Get all available tags
    pub async fn get_all_tags(&self) -> Vec<String> {
        let config_manager = self.config_manager.read().await;
        let mut tags = std::collections::HashSet::new();
        for server in config_manager.get_all_servers() {
            // This would need to be extended to include tags from the config
            // For now, return empty vec
        }
        tags.into_iter().collect()
    }
}


/// Wrapper for MCP server manager with simplified API
#[derive(Debug)]
pub struct ServerManager {
    manager: MCPServerManager,
}

impl ServerManager {
    /// Create a new server manager
    pub async fn new() -> Result<Self> {
        Ok(Self {
            manager: MCPServerManager::new().await?,
        })
    }

    /// Create a new server manager with custom config path
    pub async fn with_config_path<P: AsRef<std::path::Path>>(config_path: P) -> Result<Self> {
        Ok(Self {
            manager: MCPServerManager::with_config_path(config_path).await?,
        })
    }

    /// Initialize the server manager and connect to all servers
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing MCP server manager");
        self.manager.initialize().await
    }

    /// Reload configuration and reconnect servers
    pub async fn reload_config(&self) -> Result<()> {
        self.manager.reload_config().await
    }

    /// Get reference to the tool manager
    pub fn tool_manager(&self) -> &super::MCPToolManager {
        self.manager.tool_manager()
    }

    /// Get reference to the underlying server manager
    pub fn manager(&self) -> &MCPServerManager {
        &self.manager
    }
}