# MCP 配置文件使用说明

本项目支持使用本地 `mcp.json` 配置文件来管理 MCP (Model Context Protocol) 服务器。

## 配置文件位置

默认配置文件位置：
- 当前目录：`./mcp.json`
- 可通过环境变量 `MCP_CONFIG_PATH` 自定义路径

## 配置文件结构

```json
{
  "global": {
    "timeout": 30,
    "auto_reconnect": true,
    "max_retries": 3
  },
  "servers": {
    "server-name": {
      "name": "server-name",
      "transport": "sse|stdio|http",
      "endpoint": "服务器端点URL或命令",
      "enabled": true|false,
      "description": "服务器描述",
      "tags": ["tag1", "tag2"],
      "auth": {
        "type": "none|api_key|bearer|basic",
        "key": "API密钥（仅api_key类型）",
        "header": "HTTP头名称（仅api_key类型）",
        "token": "Bearer令牌（仅bearer类型）",
        "username": "用户名（仅basic类型）",
        "password": "密码（仅basic类型）"
      }
    }
  }
}
```

## 全局配置 (global)

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `timeout` | 数字 | 30 | 服务器连接超时时间（秒） |
| `auto_reconnect` | 布尔值 | true | 是否自动重连 |
| `max_retries` | 数字 | 3 | 最大重试次数 |

## 服务器配置 (servers)

每个服务器配置包含以下字段：

### 基本字段

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `name` | 字符串 | 是 | 服务器唯一名称 |
| `transport` | 字符串 | 是 | 传输类型 |
| `endpoint` | 字符串 | 是 | 服务器端点 |
| `enabled` | 布尔值 | 否 | 是否启用（默认：true） |
| `description` | 字符串 | 否 | 服务器描述 |
| `tags` | 数组 | 否 | 标签列表 |

### 传输类型 (transport)

| 值 | 说明 | 示例 |
|----|-----|------|
| `sse` | Server-Sent Events | `http://localhost:8080/sse` |
| `stdio` | 标准输入输出 | `python -m mcp_server_tools` |
| `http` | HTTP 请求 | `https://api.example.com/mcp` |

### 认证配置 (auth)

#### 无认证 (none)
```json
{
  "type": "none"
}
```

#### API Key 认证 (api_key)
```json
{
  "type": "api_key",
  "key": "your-api-key",
  "header": "X-API-Key"
}
```

#### Bearer Token 认证 (bearer)
```json
{
  "type": "bearer",
  "token": "your-bearer-token"
}
```

#### 基本认证 (basic)
```json
{
  "type": "basic",
  "username": "your-username",
  "password": "your-password"
}
```

## 环境变量支持

配置文件中的值可以使用环境变量替换，格式：`${ENV_VAR_NAME}`

```json
{
  "servers": {
    "github": {
      "name": "github",
      "transport": "sse",
      "endpoint": "https://api.github.com/mcp",
      "auth": {
        "type": "bearer",
        "token": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

## 示例配置

### 基本示例
```json
{
  "global": {
    "timeout": 30,
    "auto_reconnect": true,
    "max_retries": 3
  },
  "servers": {
    "memory": {
      "name": "memory",
      "transport": "stdio",
      "endpoint": "npx @modelcontextprotocol/server-memory",
      "enabled": true,
      "description": "内存服务器",
      "tags": ["memory", "tools"],
      "auth": {
        "type": "none"
      }
    }
  }
}
```

### 多服务器示例
```json
{
  "global": {
    "timeout": 30,
    "auto_reconnect": true,
    "max_retries": 3
  },
  "servers": {
    "local-dev": {
      "name": "local-dev",
      "transport": "sse",
      "endpoint": "http://localhost:8080/sse",
      "enabled": true,
      "description": "本地开发服务器",
      "tags": ["development", "local"],
      "auth": {
        "type": "api_key",
        "key": "dev-api-key",
        "header": "X-API-Key"
      }
    },
    "filesystem": {
      "name": "filesystem",
      "transport": "stdio",
      "endpoint": "npx @modelcontextprotocol/server-filesystem",
      "enabled": false,
      "description": "文件系统服务器",
      "tags": ["filesystem", "tools"],
      "auth": {
        "type": "none"
      }
    },
    "github": {
      "name": "github",
      "transport": "sse",
      "endpoint": "https://api.github.com/mcp",
      "enabled": false,
      "description": "GitHub 集成服务器",
      "tags": ["github", "development"],
      "auth": {
        "type": "bearer",
        "token": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

## 常用 MCP 服务器

以下是一些常用的 MCP 服务器配置：

### 内存服务器
```json
{
  "name": "memory",
  "transport": "stdio",
  "endpoint": "npx @modelcontextprotocol/server-memory",
  "enabled": true,
  "description": "提供临时存储和检索功能",
  "tags": ["memory", "tools"],
  "auth": { "type": "none" }
}
```

### 文件系统服务器
```json
{
  "name": "filesystem",
  "transport": "stdio",
  "endpoint": "npx @modelcontextprotocol/server-filesystem",
  "enabled": false,
  "description": "文件系统操作工具",
  "tags": ["filesystem", "tools"],
  "auth": { "type": "none" }
}
```

### SQLite 数据库服务器
```json
{
  "name": "sqlite",
  "transport": "stdio",
  "endpoint": "npx @modelcontextprotocol/server-sqlite",
  "enabled": false,
  "description": "SQLite 数据库操作工具",
  "tags": ["database", "sqlite"],
  "auth": { "type": "none" }
}
```

### Puppeteer 网页自动化
```json
{
  "name": "puppeteer",
  "transport": "stdio",
  "endpoint": "npx @modelcontextprotocol/server-puppeteer",
  "enabled": false,
  "description": "网页自动化工具",
  "tags": ["automation", "web"],
  "auth": { "type": "none" }
}
```

### Brave 搜索服务器
```json
{
  "name": "brave-search",
  "transport": "sse",
  "endpoint": "https://api.search.brave.com/mcp",
  "enabled": false,
  "description": "Brave 搜索集成",
  "tags": ["search", "web"],
  "auth": {
    "type": "api_key",
    "key": "${BRAVE_API_KEY}",
    "header": "X-Subscription-Token"
  }
}
```

## 使用说明

1. **创建配置文件**：复制 `mcp.json.example` 为 `mcp.json`
2. **编辑配置**：根据需要修改服务器配置
3. **设置环境变量**：如果使用了环境变量替换，确保设置相应的环境变量
4. **启动应用**：应用会自动加载配置并连接到启用的服务器

## 故障排除

### 配置文件未找到
如果配置文件未找到，应用会使用空配置，不会连接任何 MCP 服务器。

### 服务器连接失败
检查以下项目：
- 服务器端点 URL 是否正确
- 认证信息是否正确
- 网络连接是否正常
- MCP 服务器是否正在运行

### 权限问题
确保应用有权限：
- 读取配置文件
- 执行 stdio 类型的命令
- 访问网络端点

## 日志

应用启动时会输出 MCP 相关日志：
- 配置文件加载状态
- 服务器连接状态
- 错误信息（如果有）

使用适当的日志级别可以查看详细的调试信息。