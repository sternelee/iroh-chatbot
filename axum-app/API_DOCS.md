# Chat API Documentation

This document describes the chat API endpoints available in the Axum backend, designed with AI SDK compatibility in mind.

## Overview

The chat API provides multi-provider AI integration with intelligent fallback:

1. **Legacy Chat API** (`/api/chat`) - Compatible with the existing frontend
2. **Multi-Provider Chat API** (`/api/v1/chat/completions`) - Non-streaming (OpenAI & Gemini)
3. **Multi-Provider Streaming Chat API** (`/api/v1/chat/completions`) - Server-Sent Events (OpenAI & Gemini)

## AI Provider Integration

The backend integrates with **multiple AI providers** based on the model name in requests:

- **OpenAI**: Models starting with `gpt-` (e.g., `gpt-4`, `gpt-3.5-turbo`)
- **Google Gemini**: Models starting with `gemini-` (e.g., `gemini-1.5-flash`, `gemini-pro`)

### Environment Variables

Set the following environment variables to enable AI provider integration:

```bash
# OpenAI API Configuration (Optional)
OPENAI_API_KEY=sk-your-openai-api-key
OPENAI_DEFAULT_MODEL=gpt-3.5-turbo

# Google Gemini API Configuration (Optional)
GOOGLE_AI_API_KEY=your_google_ai_api_key_here
GEMINI_MODEL=gemini-1.5-flash
```

### Configuration File

Create a `.env` file in the `axum-app` directory:

```bash
# Copy the example template
cp .env.example .env

# Edit with your API keys
OPENAI_API_KEY=sk-your-openai-api-key
GOOGLE_AI_API_KEY=your_google_ai_api_key_here
GEMINI_MODEL=gemini-1.5-flash
```

### Fallback Behavior

- **With API Keys**: Real AI responses with streaming support
- **Without API Keys**: Intelligent fallback responses with warning messages
- **API Errors**: Graceful degradation to fallback responses with error details

### Supported Models

**OpenAI Models:**
- `gpt-4o` - Latest GPT-4 model
- `gpt-4o-mini` - Smaller, faster GPT-4 model
- `gpt-4-turbo` - Turbo variant of GPT-4
- `gpt-4` - Original GPT-4 model
- `gpt-3.5-turbo` - Default fallback model

**Google Gemini Models:**
- `gemini-1.5-flash` - Fast, cost-effective
- `gemini-1.5-pro` - More capable, slower
- `gemini-1.5-flash-8b` - Smaller, faster
- `gemini-pro` - Previous generation
- `gemini-pro-vision` - Multimodal

### Automatic Provider Detection

The system automatically routes requests based on the model name:

```javascript
// Routes to OpenAI
{ model: "gpt-4", messages: [...] }

// Routes to Gemini
{ model: "gemini-1.5-flash", messages: [...] }

// Default to OpenAI for unknown models
{ model: "unknown-model", messages: [...] }
```

### Real vs Mock Response Examples

**Without OpenAI API (Fallback):**
```json
{
  "content": "That's a fascinating topic! Let me share what I know about it.",
  "role": "assistant",
  "warning": "OpenAI API not configured, using fallback response"
}
```

**With OpenAI API (Real GPT):**
```json
{
  "id": "chatcmpl-abc123",
  "role": "assistant",
  "content": "Hello! I'm a real AI assistant powered by GPT.",
  "created_at": "2024-12-06T01:30:00Z",
  "metadata": {
    "model": "gpt-3.5-turbo",
    "usage": {"prompt_tokens": 10, "completion_tokens": 15, "total_tokens": 25}
  }
}
```

## Endpoints

### 1. Legacy Chat API

**Endpoint:** `POST /api/chat`

**Description:** Legacy endpoint compatible with the existing Vue frontend

**Request Body:**
```json
{
  "messages": [
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ]
}
```

**Response:**
```json
{
  "role": "assistant",
  "content": "That's interesting! Tell me more about that."
}
```

**Example:**
```bash
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ]
  }'
```

### 2. OpenAI-Compatible Chat API (Non-streaming)

**Endpoint:** `POST /api/v1/chat/completions`

**Description:** OpenAI-compatible chat completion endpoint with structured responses

**Request Body:**
```json
{
  "messages": [
    {
      "id": "msg1",
      "role": "user",
      "content": "Hello, how are you?",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "model": "gpt-3.5-turbo",
  "stream": false,
  "max_tokens": 1000,
  "temperature": 0.7
}
```

**Response:**
```json
{
  "id": "msg_1234",
  "role": "assistant",
  "content": "That's interesting! Tell me more about that.",
  "created_at": "2024-01-01T00:00:00Z",
  "attachments": null,
  "metadata": null
}
```

**Example:**
```bash
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {
        "id": "msg1",
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "model": "gpt-3.5-turbo",
    "stream": false
  }'
```

### 3. OpenAI-Compatible Chat API (Streaming)

**Endpoint:** `POST /api/v1/chat/completions` (with `"stream": true`)

**Description:** Server-Sent Events streaming response using AI SDK compatible format

**Note:** The same endpoint handles both streaming and non-streaming requests. Set `"stream": true` in the request body to enable streaming.

**Request Body:**
```json
{
  "messages": [
    {
      "id": "msg1",
      "role": "user",
      "content": "Hello, how are you?"
    }
  ],
  "model": "gpt-3.5-turbo",
  "stream": true
}
```

**Response (Server-Sent Events):**
```
data: {"type":"text-delta","textDelta":"That's "}
data: {"type":"text-delta","textDelta":"interesting! "}
data: {"type":"text-delta","textDelta":"Tell me "}
data: {"type":"text-delta","textDelta":"more about "}
data: {"type":"text-delta","textDelta":"that."}
data: [DONE]
```

**Example:**
```bash
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "messages": [
      {
        "id": "msg1",
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "model": "gpt-3.5-turbo",
    "stream": true
  }'
```

## Data Models

### ChatMessage

```rust
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attachments: Option<Vec<Attachment>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
```

### ChatRole

```rust
pub enum ChatRole {
    User,
    Assistant,
    System,
}
```

### Attachment

```rust
pub struct Attachment {
    pub attachment_type: String,
    pub url: String,
    pub media_type: Option<String>,
    pub filename: Option<String>,
}
```

### ChatCompletionRequest

```rust
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}
```

## Error Handling

The API returns appropriate HTTP status codes:

- **200 OK**: Successful request
- **400 Bad Request**: Invalid request format or missing required fields
- **500 Internal Server Error**: Server error during processing

### Error Response Format

```json
{
  "error": "Invalid request format"
}
```

## AI SDK Compatibility

The streaming response format is compatible with the AI SDK's UI message stream format:

### UI Message Chunk Types

- **text-delta**: Partial text content
- **tool-call**: Tool invocation request
- **tool-result**: Tool execution result
- **step-finish**: Completion of a reasoning step
- **finish**: Completion with optional metadata
- **error**: Error information

## Usage Examples

### JavaScript/Fetch API (Non-streaming)

```javascript
const response = await fetch('http://localhost:3000/api/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    messages: [
      { id: 'msg1', role: 'user', content: 'Hello, how are you?' }
    ],
    model: 'gpt-3.5-turbo',
    stream: false
  })
});

const data = await response.json();
console.log(data.content);
```

### JavaScript/Fetch API (Streaming)

```javascript
const response = await fetch('http://localhost:3000/api/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'text/event-stream',
  },
  body: JSON.stringify({
    messages: [
      { id: 'msg1', role: 'user', content: 'Hello, how are you?' }
    ],
    model: 'gpt-3.5-turbo',
    stream: true
  })
});

const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;

  const chunk = decoder.decode(value);
  const lines = chunk.split('\n');

  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') return;

      try {
        const parsed = JSON.parse(data);
        if (parsed.type === 'text-delta') {
          console.log(parsed.textDelta);
        }
      } catch (e) {
        // Ignore parsing errors
      }
    }
  }
}
```

## Testing

Run the test suite to verify API functionality:

```bash
cargo test
```

The tests cover:
- Basic endpoint functionality
- Error handling for invalid requests
- Request validation
- Response format verification