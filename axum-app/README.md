# Axum Chat Backend

Real OpenAI ChatGPT integration for the iroh-chat application with intelligent fallback.

## Quick Start

### 1. Configuration

Copy the example environment file:
```bash
cp .env.example .env
```

Edit `.env` with your OpenAI API key:
```bash
OPENAI_API_KEY=sk-your-openai-api-key
OPENAI_DEFAULT_MODEL=gpt-3.5-turbo
```

### 2. Run the Server

```bash
# Development server
cargo run

# Production build
cargo build --release
./target/release/axum-app
```

### 3. Test the API

```bash
# Legacy API (compatible with existing frontend)
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"messages": [{"role": "user", "content": "Hello!"}]}'

# OpenAI-compatible API
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"id": "msg1", "role": "user", "content": "Hello!"}
    ],
    "model": "gpt-3.5-turbo",
    "stream": false
  }'
```

## Features

- ✅ **Real OpenAI Integration** - Uses actual GPT models when configured
- ✅ **Intelligent Fallback** - Mock responses when OpenAI is not available
- ✅ **Streaming Support** - Server-Sent Events for real-time responses
- ✅ **AI SDK Compatible** - Follows AI SDK streaming format
- ✅ **Error Handling** - Graceful degradation and detailed error reporting
- ✅ **Multiple Models** - Support for GPT-4, GPT-3.5-turbo, etc.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENAI_API_KEY` | Yes | - | Your OpenAI API key |
| `OPENAI_DEFAULT_MODEL` | No | `gpt-3.5-turbo` | Default model to use |
| `OPENAI_API_BASE_URL` | No | `https://api.openai.com/v1` | OpenAI API endpoint |

## Supported Models

- `gpt-4o` - Latest and most capable GPT model
- `gpt-4o-mini` - Smaller, faster GPT-4 model
- `gpt-4-turbo` - Turbo variant of GPT-4
- `gpt-4` - Original GPT-4 model
- `gpt-3.5-turbo` - Fast, efficient model (default)

## API Endpoints

### Legacy Chat API
- **Endpoint**: `POST /api/chat`
- **Purpose**: Compatible with existing Vue frontend
- **Response**: Simple `{role, content}` format

### OpenAI-Compatible API
- **Endpoint**: `POST /api/v1/chat/completions`
- **Purpose**: Full OpenAI compatibility with metadata
- **Response**: Structured message with id, timestamps, usage info
- **Streaming**: Set `"stream": true` for real-time responses

## Error Handling

The API gracefully handles various scenarios:

- **No OpenAI API key**: Uses fallback responses with warning
- **Invalid API key**: Returns error with fallback response
- **API outages**: Degrades to fallback responses
- **Network issues**: Timeout handling with fallback

## Testing

```bash
# Run all tests
cargo test

# Run specific tests
cargo test chat::tests

# Run with output
cargo test -- --nocapture
```

## Development

### Project Structure
```
src/
├── lib.rs              # Application entry point
├── chat.rs             # Chat API handlers and types
├── openai_service.rs   # OpenAI integration service
├── routes.rs           # Legacy todo routes
├── todo.rs             # Todo data structures
└── main.rs             # Standalone server binary
```

### Adding New Features

1. **New API endpoints**: Add to `lib.rs` router
2. **OpenAI features**: Extend `openai_service.rs`
3. **Message types**: Update `chat.rs` data structures
4. **Tests**: Add test cases in relevant modules

### Dependencies

- `axum` - Web framework
- `tokio` - Async runtime
- `reqwest` - HTTP client for OpenAI API
- `serde` - Serialization/deserialization
- `async-stream` - Async stream utilities

## Deployment

The application is designed to run both standalone and integrated with Tauri:

### Standalone
```bash
cargo run --release
```

### Integrated with Tauri
The chat service is automatically imported by the Tauri application.

## Troubleshooting

### Common Issues

1. **"OPENAI_API_KEY not set"**
   - Set the environment variable or create `.env` file
   - Ensure the API key is valid

2. **API returns 401 Unauthorized**
   - Check your OpenAI API key validity
   - Verify you have sufficient credits

3. **Slow responses**
   - Check network connectivity
   - Try using a faster model (gpt-3.5-turbo)

4. **Compilation errors**
   - Ensure Rust version is up to date
   - Run `cargo clean && cargo build`

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo test` and `cargo fmt`
6. Submit a pull request

## License

This project is part of the iroh-chat application.