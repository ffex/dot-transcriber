# Dot - Voice-to-Notes Telegram Bot

🎤 Transform voice messages into structured markdown notes and actionable tasks.
Develop as experiment with Claude Code!

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
- **CMake** (required for building whisper.cpp)
- **ffmpeg** (required for Telegram voice message support)
- (Optional) Ollama installed for local AI processing (Phase 3)

#### Installing Dependencies

**macOS**:
```bash
brew install cmake ffmpeg
```

**Windows** (with Chocolatey):
```bash
choco install cmake ffmpeg
```

**Linux** (Ubuntu/Debian):
```bash
sudo apt install cmake ffmpeg
```

### Setup

1. Download Whisper model:
```bash
cd models
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -o ggml-base.bin
cd ..
```

2. Configure your bot:
```bash
cp config.example.toml config.toml
# Edit config.toml with your Telegram bot token
```

3. Build with hardware acceleration:

**M1/M2/M3 Mac**:
```bash
cargo build --release --features metal
```

**Windows/Linux with NVIDIA GPU**:
```bash
cargo build --release --features cuda
```

**CPU-only (any system)**:
```bash
cargo build --release --features cpu
```

4. Run:
```bash
# Mac
cargo run --features metal

# Windows NVIDIA
cargo run --features cuda

# CPU
cargo run --features cpu
```

## Configuration

See `config.example.toml` for all available options.

Key settings:
- Telegram bot token
- Transcription service (Whisper API, local, etc.)
- AI model (Ollama local or cloud API)
- Output directories

## Project Status

- ✅ **Phase 1**: Telegram Bot Foundation (Complete & Tested)
- ✅ **Phase 2**: Audio Transcription (Complete & Tested)
- ✅ **Phase 3**: AI Note Generation (Complete & Tested) 🎉
- 🔴 **Phase 4**: Task Extraction (Optional - Not Started)

**System Fully Functional!** 🚀

**Current Features**:
- 🤖 Telegram bot with Italian responses
- 🎤 Voice message transcription (Italian)
- ✨ **LLM-based transcription cleanup** (fixes errors)
- 🧠 **AI-powered note generation** (Ollama)
- 📝 Structured markdown notes with frontmatter
- 🏷️ Automatic tag suggestions
- 💾 Save notes to files (Obsidian-compatible)
- 🚀 Metal/CUDA acceleration support
- 🌐 Local + Remote Ollama support (LAN)
- 📱 Commands: /start, /help, /status

See [where-are-we.md](./where-are-we.md) for detailed development status and roadmap.

## Development

See [ClaudePrompts.md](./ClaudePrompts.md) for development guidelines.

## License

MIT
