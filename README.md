# Dot - Voice-to-Notes Telegram Bot

🎤 Transform voice messages into structured markdown notes and actionable tasks.

## What is Dot?

Dot is your AI companion that listens to your voice messages on Telegram and transforms them into organized notes for your second brain. Perfect for capturing ideas while driving, walking, or anytime you prefer speaking over typing.

## Features

- 🤖 **Telegram Bot Integration**: Send voice messages directly to your bot
- 🇮🇹 **Italian Transcription**: Native support for Italian language
- 📝 **Smart Note Generation**: AI-powered transformation into structured markdown
- ✅ **Task Extraction**: Automatically identify actionable items from your recordings
- 🗂️ **Obsidian Compatible**: Generate notes ready for your second brain

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Telegram account
- (Optional) Ollama installed for local AI processing

### Setup

1. Clone and build:
```bash
cargo build --release
```

2. Configure your bot:
```bash
cp config.example.toml config.toml
# Edit config.toml with your settings
```

3. Set up environment variables:
```bash
cp .env.example .env
# Add your API keys to .env
```

4. Run:
```bash
cargo run
```

## Configuration

See `config.example.toml` for all available options.

Key settings:
- Telegram bot token
- Transcription service (Whisper API, local, etc.)
- AI model (Ollama local or cloud API)
- Output directories

## Project Status

🔴 **Phase 0**: Project Setup (In Progress)

See [WhereAreWe.md](./WhereAreWe.md) for detailed development status and roadmap.

## Development

See [ClaudePrompts.md](./ClaudePrompts.md) for development guidelines.

## License

MIT
