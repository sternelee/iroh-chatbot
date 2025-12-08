use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, warn};

use super::{MCPServerConfig, MCPTransportType};

/// MCP 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// MCP 服务器配置列表
    pub servers: HashMap<String, MCPServerConfigFile>,
    /// 全局配置
    #[serde(default)]
    pub global: MCPGlobalConfig,
}

/// 全局 MCP 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPGlobalConfig {
    /// 默认超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// 是否启用自动重连
    #[serde(default = "default_auto_reconnect")]
    pub auto_reconnect: bool,
    /// 最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

impl Default for MCPGlobalConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            auto_reconnect: default_auto_reconnect(),
            max_retries: default_max_retries(),
        }
    }
}

/// 文件中的 MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfigFile {
    /// 服务器名称
    pub name: String,
    /// 传输类型
    #[serde(rename = "transport")]
    pub transport_type: String,
    /// 端点 URL
    pub endpoint: String,
    /// 认证配置
    #[serde(default)]
    pub auth: Option<MCPAuthConfigFile>,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 文件中的认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MCPAuthConfigFile {
    /// 无认证
    #[serde(rename = "none")]
    None,
    /// API Key 认证
    #[serde(rename = "api_key")]
    ApiKey {
        /// API Key
        key: String,
        /// Header 名称
        #[serde(default = "default_auth_header")]
        header: String,
    },
    /// Bearer Token 认证
    #[serde(rename = "bearer")]
    Bearer {
        /// Token
        token: String,
    },
    /// 基本认证
    #[serde(rename = "basic")]
    Basic {
        /// 用户名
        username: String,
        /// 密码
        password: String,
    },
}

/// MCP 配置管理器
#[derive(Debug)]
pub struct MCPConfigManager {
    config: MCPConfig,
    config_path: String,
}

impl MCPConfigManager {
    /// 创建新的配置管理器
    pub async fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref().to_string_lossy().to_string();
        let config = Self::load_config(&config_path).await?;

        info!("MCP config manager created with config file: {}", config_path);
        Ok(Self { config, config_path })
    }

    /// 从文件加载配置
    async fn load_config(config_path: &str) -> Result<MCPConfig> {
        debug!("Loading MCP config from: {}", config_path);

        match fs::read_to_string(config_path).await {
            Ok(content) => {
                let config: MCPConfig = serde_json::from_str(&content)
                    .map_err(|e| anyhow!("Failed to parse MCP config: {}", e))?;

                info!("Successfully loaded MCP config with {} servers", config.servers.len());
                Ok(config)
            }
            Err(e) => {
                warn!("Failed to read MCP config file '{}': {}", config_path, e);
                warn!("Using empty MCP config");
                Ok(MCPConfig {
                    servers: HashMap::new(),
                    global: MCPGlobalConfig::default(),
                })
            }
        }
    }

    /// 重新加载配置
    pub async fn reload(&mut self) -> Result<()> {
        info!("Reloading MCP config from: {}", self.config_path);
        self.config = Self::load_config(&self.config_path).await?;
        info!("MCP config reloaded successfully");
        Ok(())
    }

    /// 获取所有启用的服务器配置
    pub fn get_enabled_servers(&self) -> Vec<MCPServerConfig> {
        self.config.servers
            .values()
            .filter(|server| server.enabled)
            .filter_map(|server| self.convert_server_config(server).ok())
            .collect()
    }

    /// 获取所有服务器配置
    pub fn get_all_servers(&self) -> Vec<MCPServerConfig> {
        self.config.servers
            .values()
            .filter_map(|server| self.convert_server_config(server).ok())
            .collect()
    }

    /// 根据标签筛选服务器
    pub fn get_servers_by_tag(&self, tag: &str) -> Vec<MCPServerConfig> {
        self.config.servers
            .values()
            .filter(|server| server.enabled && server.tags.contains(&tag.to_string()))
            .filter_map(|server| self.convert_server_config(server).ok())
            .collect()
    }

    /// 根据名称获取服务器配置
    pub fn get_server_by_name(&self, name: &str) -> Option<MCPServerConfig> {
        self.config.servers.get(name)
            .filter(|server| server.enabled)
            .and_then(|server| self.convert_server_config(server).ok())
    }

    /// 转换配置文件格式到内部格式
    fn convert_server_config(&self, server: &MCPServerConfigFile) -> Result<MCPServerConfig> {
        let transport_type = match server.transport_type.to_lowercase().as_str() {
            "sse" | "server-sent-events" => super::MCPTransportType::ServerSentEvents,
            "stdio" | "child-process" => super::MCPTransportType::ChildProcess,
            "http" | "streamable-http" => super::MCPTransportType::StreamableHttp,
            _ => return Err(anyhow!("Unsupported transport type: {}", server.transport_type)),
        };

        let auth = match server.auth.as_ref() {
            Some(MCPAuthConfigFile::None) => Some(super::MCPAuthConfig::None),
            Some(MCPAuthConfigFile::ApiKey { key, header }) => Some(
                super::MCPAuthConfig::ApiKey {
                    key: key.clone(),
                    header: header.clone(),
                }
            ),
            Some(MCPAuthConfigFile::Bearer { token }) => Some(
                super::MCPAuthConfig::Bearer {
                    token: token.clone(),
                }
            ),
            Some(MCPAuthConfigFile::Basic { username, password }) => Some(
                super::MCPAuthConfig::Basic {
                    username: username.clone(),
                    password: password.clone(),
                }
            ),
            None => None,
        };

        Ok(MCPServerConfig {
            name: server.name.clone(),
            transport_type,
            endpoint: server.endpoint.clone(),
            auth,
        })
    }

    /// 获取全局配置
    pub fn get_global_config(&self) -> &MCPGlobalConfig {
        &self.config.global
    }

    /// 保存当前配置到文件
    pub async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| anyhow!("Failed to serialize MCP config: {}", e))?;

        fs::write(&self.config_path, content)
            .await
            .map_err(|e| anyhow!("Failed to save MCP config: {}", e))?;

        info!("MCP config saved to: {}", self.config_path);
        Ok(())
    }

    /// 添加或更新服务器配置
    pub fn add_server(&mut self, server: MCPServerConfigFile) {
        self.config.servers.insert(server.name.clone(), server);
    }

    /// 删除服务器配置
    pub fn remove_server(&mut self, name: &str) -> bool {
        self.config.servers.remove(name).is_some()
    }

    /// 获取服务器统计信息
    pub fn get_stats(&self) -> MCPConfigStats {
        let total_servers = self.config.servers.len();
        let enabled_servers = self.config.servers.values().filter(|s| s.enabled).count();
        let servers_by_transport = {
            let mut counts = HashMap::new();
            for server in self.config.servers.values() {
                *counts.entry(server.transport_type.clone()).or_insert(0) += 1;
            }
            counts
        };

        MCPConfigStats {
            total_servers,
            enabled_servers,
            disabled_servers: total_servers - enabled_servers,
            servers_by_transport,
        }
    }
}

/// 配置统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfigStats {
    /// 总服务器数
    pub total_servers: usize,
    /// 启用的服务器数
    pub enabled_servers: usize,
    /// 禁用的服务器数
    pub disabled_servers: usize,
    /// 按传输类型统计的服务器数
    pub servers_by_transport: HashMap<String, usize>,
}

// 默认值函数
fn default_timeout() -> u64 { 30 }
fn default_auto_reconnect() -> bool { true }
fn default_max_retries() -> u32 { 3 }
fn default_enabled() -> bool { true }
fn default_auth_header() -> String { "X-API-Key".to_string() }

/// 创建示例配置文件
pub fn create_example_config() -> MCPConfig {
    let mut servers = HashMap::new();

    // 示例：本地开发服务器
    servers.insert("local-dev".to_string(), MCPServerConfigFile {
        name: "local-dev".to_string(),
        transport_type: "sse".to_string(),
        endpoint: "http://localhost:8080/sse".to_string(),
        auth: Some(MCPAuthConfigFile::ApiKey {
            key: "dev-api-key".to_string(),
            header: "X-API-Key".to_string(),
        }),
        enabled: true,
        description: Some("本地开发 MCP 服务器".to_string()),
        tags: vec!["development".to_string(), "local".to_string()],
    });

    // 示例：生产服务器
    servers.insert("production".to_string(), MCPServerConfigFile {
        name: "production".to_string(),
        transport_type: "sse".to_string(),
        endpoint: "https://api.example.com/mcp".to_string(),
        auth: Some(MCPAuthConfigFile::Bearer {
            token: "${PROD_BEARER_TOKEN}".to_string(),
        }),
        enabled: false,
        description: Some("生产环境 MCP 服务器".to_string()),
        tags: vec!["production".to_string(), "remote".to_string()],
    });

    // 示例：工具服务器
    servers.insert("tools".to_string(), MCPServerConfigFile {
        name: "tools".to_string(),
        transport_type: "stdio".to_string(),
        endpoint: "python -m mcp_server_tools".to_string(),
        auth: Some(MCPAuthConfigFile::None),
        enabled: true,
        description: Some("本地工具 MCP 服务器".to_string()),
        tags: vec!["tools".to_string(), "local".to_string()],
    });

    MCPConfig {
        servers,
        global: MCPGlobalConfig {
            timeout: 30,
            auto_reconnect: true,
            max_retries: 3,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::fs;

    #[tokio::test]
    async fn test_config_loading() {
        let temp_file = NamedTempFile::new().unwrap();
        let example_config = create_example_config();

        let config_content = serde_json::to_string_pretty(&example_config).unwrap();
        fs::write(temp_file.path(), config_content).await.unwrap();

        let manager = MCPConfigManager::new(temp_file.path()).await.unwrap();
        let servers = manager.get_enabled_servers();

        assert!(!servers.is_empty());
        assert_eq!(servers.len(), 2); // local-dev 和 tools 被启用
    }

    #[tokio::test]
    async fn test_config_by_tags() {
        let temp_file = NamedTempFile::new().unwrap();
        let example_config = create_example_config();

        let config_content = serde_json::to_string_pretty(&example_config).unwrap();
        fs::write(temp_file.path(), config_content).await.unwrap();

        let manager = MCPConfigManager::new(temp_file.path()).await.unwrap();
        let dev_servers = manager.get_servers_by_tag("development");
        let prod_servers = manager.get_servers_by_tag("production");

        assert_eq!(dev_servers.len(), 1);
        assert_eq!(prod_servers.len(), 0); // production 服务器被禁用
    }
}