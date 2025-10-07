# AI Gateway - LLM Hub Data Plane

High-performance unified LLM API gateway for the LLM Hub platform.

## Features

- 🚀 **Unified LLM API**: Single interface for multiple providers (OpenAI, Anthropic, Google, etc.)
- 🎯 **Smart Routing**: Intelligent provider selection based on cost and performance
- 💰 **Cost Optimization**: Caching, prompt optimization, and usage tracking
- 🔐 **Security**: Project API key authentication, encryption, rate limiting
- 📊 **Analytics**: Real-time usage monitoring and cost tracking
- 🎵 **Multimodal**: Support for text, audio transcription, and real-time voice

## Quick Start

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- MongoDB 7.0+
- Docker (optional)

### Development Setup

1. **Clone the repository**:
```bash
git clone https://github.com/ai-llm-hub/llm-hub-ai-gateway.git
cd ai-gateway
```

2. **Configure environment**:
```bash
cp .env.example .env.development
# Edit .env.development with your settings
```

3. **Run the server**:
```bash
./run.sh run
```

4. **Check health**:
```bash
curl http://localhost:3001/health
```

5. **View API docs**:
Open http://localhost:3001/swagger-ui/ in your browser

## Architecture

The AI Gateway follows Clean Architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                     API Layer (HTTP)                     │
│  - Handlers, DTOs, Middleware, Routers                   │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                  Domain Layer (Business)                 │
│  - Entities, Services, Repositories, Providers           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│              Infrastructure Layer (External)             │
│  - Database (MongoDB/PostgreSQL), Cache, LLM Clients     │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│              Shared Layer (Cross-cutting)                │
│  - Config, Error, Utils                                  │
└─────────────────────────────────────────────────────────┘
```

## API Endpoints

### Audio Transcription

```bash
POST /v1/audio/transcribe

curl -X POST http://localhost:3001/v1/audio/transcribe \
  -H "Authorization: Bearer pk_your_api_key" \
  -F "file=@audio.mp3" \
  -F "model=whisper-1"
```

### Health Check

```bash
GET /health

curl http://localhost:3001/health
```

## Commands

```bash
# Development
./run.sh build          # Build debug
./run.sh run            # Run server
./run.sh test           # Run tests
./run.sh watch          # Auto-reload

# Code Quality
./run.sh check          # Check code
./run.sh clippy         # Run linter
./run.sh fmt            # Format code

# Production
./run.sh build --release
./run.sh run --release
```

## Configuration

### Environment Variables

- `AI_GATEWAY_SERVER_PORT`: Server port (default: 3001)
- `AI_GATEWAY_DATABASE_MONGODB_URL`: MongoDB connection URL
- `AI_GATEWAY_SECURITY_ENCRYPTION_KEY`: Base64-encoded 32-byte key
- `RUST_LOG`: Log level (default: ai_gateway=debug)

### Database Support

The AI Gateway uses the Repository Pattern for database abstraction:
- **Current**: MongoDB
- **Planned**: PostgreSQL, MySQL

## Security

- **Authentication**: Project API keys (format: `pk_xxxxx`)
- **Encryption**: AES-256-GCM for LLM API keys
- **Transport**: TLS 1.3 required
- **Rate Limiting**: Per-project quotas

## License

MIT