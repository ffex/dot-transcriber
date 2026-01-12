# Dot - Voice-to-Notes Telegram Bot

## Project Vision
Dot is an AI assistant Telegram bot that helps transform voice messages into structured notes and actionable tasks for a second-brain system (Obsidian-style). Perfect for recording thoughts while driving or walking.

## Key Features
- 🎤 **Voice Input**: Record and send audio messages via Telegram
- 📝 **Transcription**: Convert Italian audio to text
- 🧠 **Note Generation**: Transform transcripts into structured markdown notes
- ✅ **Task Extraction**: (Optional) Identify and extract actionable tasks from developer-focused content

## Technology Stack
- **Language**: Rust
- **Bot**: Telegram Bot API (teloxide)
- **Transcription**: whisper-rs (local whisper.cpp with Metal/CUDA acceleration)
- **Audio Processing**: symphonia + hound
- **AI Processing**: Ollama (local) or API-based models (OpenAI, Anthropic) - Coming in Phase 3
- **Output Format**: Markdown (.md)

---

## Development Phases

### Phase 1: Telegram Bot Foundation
**Status**: 🟢 Complete

**Goals**:
- Set up Telegram Bot with BotFather
- Implement audio message reception
- Handle file downloads from Telegram
- Basic bot commands and responses

**Tasks**:
- [x] Create Telegram bot with BotFather and get API token
- [x] Add teloxide or telegram-bot Rust crate to project
- [x] Implement webhook or polling mechanism
- [x] Handle audio message types (voice, audio file)
- [x] Download audio files to local storage (deferred to Phase 2 - detection only for now)
- [x] Implement basic /start and /help commands
- [x] Add /status command for bot monitoring
- [x] Create modular handler system
- [x] Implement configuration loading (TOML + env vars)

**Dependencies**: None

**Completed**: 2026-01-11

---

### Phase 2: Audio Transcription
**Status**: 🟢 Complete & Tested ✅

**Goals**:
- Research transcription options
- Implement Italian language transcription
- Handle various audio formats

**Tasks**:
- [x] Research transcription options (whisper.cpp, faster-whisper, OpenAI API)
- [x] Choose transcription solution (whisper-rs with local whisper.cpp)
- [x] Install CMake build dependency
- [x] Add whisper-rs with Metal/CUDA feature flags
- [x] Implement audio file download from Telegram
- [x] Implement audio format conversion (any format → 16kHz mono WAV)
- [x] Integrate whisper-rs transcription
- [x] Add Italian language support
- [x] Download Whisper base model (141MB)
- [x] Add error handling for transcription failures
- [x] Update handlers to use transcription
- [x] Build with Metal acceleration for M1 Mac
- [x] Debug and fix Opus codec support (Telegram voice format)
- [x] Add ffmpeg fallback for unsupported codecs
- [x] Test successfully with real Italian voice messages

**Dependencies**: Phase 1 complete

**Completed**: 2026-01-11
**Tested**: 2026-01-11 ✅

---

### Phase 3: Note Generation with AI
**Status**: 🔴 Not Started

**Goals**:
- Process transcribed text with AI model
- Generate structured markdown notes
- Support multiple note types

**Tasks**:
- [ ] Set up Ollama locally OR choose cloud API
- [ ] Design prompt for note generation
- [ ] Implement model integration
- [ ] Parse model output to valid markdown
- [ ] Define note structure and frontmatter
- [ ] Handle long transcripts (chunking if needed)
- [ ] Save notes to output directory

**Dependencies**: Phase 2 complete

---

### Phase 4: Task Extraction (Optional)
**Status**: 🔴 Not Started

**Goals**:
- Identify actionable tasks from transcripts
- Focus on developer/project-related tasks
- Format tasks in usable format

**Tasks**:
- [ ] Design prompt for task extraction
- [ ] Implement task identification logic
- [ ] Choose task output format (TODO.md, separate files, etc.)
- [ ] Add task priority/context detection
- [ ] Link tasks to parent notes

**Dependencies**: Phase 3 complete

---

## Current Status

**Active Phase**: Phase 2 Complete ✅ - Ready for Phase 3
**Last Updated**: 2026-01-11

### What We Have
- ✅ Rust project initialized (`dot_transcriber`)
- ✅ Project tracking documents (where-are-we.md, ClaudePrompts.md, AIPrompts.md)
- ✅ Configuration system (config.toml + .env support)
- ✅ Telegram bot fully functional with teloxide
- ✅ Commands: /start, /help, /status (all in Italian)
- ✅ Audio message detection (voice messages & audio files)
- ✅ Message routing with dptree
- ✅ Modular code structure (main.rs, config.rs, handlers.rs, transcription.rs)
- ✅ Logging system with pretty_env_logger
- ✅ **Audio transcription working!**
  - Downloads audio from Telegram
  - Converts any format to WAV
  - Transcribes with Whisper (Italian)
  - Returns transcribed text
- ✅ Whisper model downloaded (ggml-base.bin, 141MB)
- ✅ Metal acceleration for M1 Mac

### What We're Working On
- ✅ Transcription tested and working!
- Preparing for Phase 3: AI Note Generation

### Next Steps
1. ✅ ~~Test transcription with Italian voice messages~~ **DONE!**
2. Implement user authentication (security enhancement)
3. Begin Phase 3: AI-powered note generation with Ollama/API
4. Design note format and structure
5. Implement markdown note generation

---

## Configuration Needs

**Required Secrets**:
- `TELEGRAM_BOT_TOKEN`: From BotFather (required) ✅

**Phase 3 Secrets** (not yet needed):
- `OPENAI_API_KEY`: If using GPT models
- `ANTHROPIC_API_KEY`: If using Claude models

**System Dependencies**:
- CMake: For building whisper.cpp bindings ✅
- ffmpeg: For Telegram voice (Opus codec) support ✅

**Configuration Options**:
- Whisper model path: `./models/ggml-base.bin` ✅
- Output directory for notes: `./output/notes`
- Temporary directory: `./temp` ✅
- Language: Italian (`it`) ✅
- Task extraction: enabled/disabled
- Note templates (Phase 3)

---

## Notes & Decisions Log

### 2026-01-11: Phase 2 Tested Successfully! ✅🎉
- ✅ **Phase 2 fully tested with real Italian voice messages - IT WORKS!**
- **Issue discovered & solved**: Telegram uses Opus codec in OGG container
  - Symphonia doesn't support Opus decoder out of the box
  - **Solution**: Implemented ffmpeg fallback for unsupported codecs
  - Pipeline now: Symphonia (if supported) → ffmpeg fallback → Whisper
- **Final audio pipeline**:
  - Download from Telegram (OGG Opus format)
  - Detect codec and try Symphonia first
  - If codec unsupported (like Opus), fall back to ffmpeg
  - Convert to 16kHz mono WAV
  - Transcribe with Whisper (Italian)
  - Return transcribed text to user
- **Dependencies verified**:
  - CMake: Required for whisper.cpp compilation ✅
  - ffmpeg: Required for Opus/Telegram voice support ✅
- **Performance**: Fast transcription with Metal acceleration on M1
- **Next**: Ready for Phase 3 (AI-powered note generation)!

### 2026-01-11: Phase 2 Complete - Transcription Implementation
- ✅ **Phase 2 implementation completed**
- **Decision: Local Whisper** (whisper-rs) over OpenAI API
  - Reasoning: Free, private, works offline, good quality with GPU acceleration
  - Cost analysis: API would cost ~$11/month for 1hr/day usage
- **Cross-platform support**:
  - M1 Mac: Built with Metal acceleration (`cargo build --features metal`)
  - Windows NVIDIA: Use CUDA (`cargo build --features cuda`)
  - CPU fallback available for other systems
- **Whisper model**: ggml-base.bin (141MB, good balance for Italian)
- **Build requirements**: CMake needed for compiling whisper.cpp bindings

### 2026-01-11: Phase 1 Complete - Bot Foundation Ready
- ✅ **Phase 1 completed and tested successfully**
- Chose **teloxide** as Telegram bot framework (v0.13, most popular Rust option)
- Implemented configuration system with TOML files and environment variable support
- Created modular handler system for easy extension
- All bot responses in Italian for native user experience
- Bot successfully receives and detects audio messages
- Audio file download deferred to Phase 2 when transcription is implemented
- **Architecture decisions**:
  - Used async/await with tokio runtime
  - dptree for declarative message routing
  - Separate modules: config, handlers, main
  - Config-driven design (no hardcoded values)

### 2026-01-11: Project Inception
- Chose Rust as implementation language
- Identified 4 main development phases
- Created project tracking system
- Key consideration: Balance between local processing (privacy, cost) vs cloud APIs (ease, quality)

---

## Questions & Open Items

1. ~~**Transcription Choice**: Local Whisper vs API?~~ ✅ **RESOLVED**: Local whisper-rs
2. **Model Choice**: Ollama local vs cloud API for note generation? (Phase 3)
3. **Note Organization**: Single file per recording or aggregate by date/topic?
4. **Note Format**: What frontmatter? Tags, dates, categories?
5. **Deployment**: Run on personal server, cloud VPS, or local machine?
6. **Security**: How to restrict bot access to specific users?

---

## Resources & References

- [Telegram Bot API Docs](https://core.telegram.org/bots/api)
- [teloxide - Rust Telegram Bot Framework](https://github.com/teloxide/teloxide)
- [dptree - Declarative handler trees](https://github.com/teloxide/dptree)
- [whisper-rs - Rust bindings for whisper.cpp](https://github.com/tazz4843/whisper-rs)
- [whisper.cpp - C++ port of Whisper](https://github.com/ggml-org/whisper.cpp)
- [Whisper models on HuggingFace](https://huggingface.co/ggerganov/whisper.cpp)
- [Ollama](https://ollama.ai/)
- [Obsidian Markdown Format](https://help.obsidian.md/Editing+and+formatting/Basic+formatting+syntax)

---

## Federico's Tasks

These are tasks for Federico to explore and implement:

### 🔍 Learning & Research
- [ ] **Read about dptree**: Understand the declarative routing pattern we're using
  - Resource: [dptree documentation](https://docs.rs/dptree/latest/dptree/)
  - Key concepts: handler trees, filters, endpoints
  - See how we use it in `src/main.rs` lines 46-66

- [ ] **Study Cargo features**: Understand `cargo build --features metal`
  - What are Cargo features and how do they work?
  - Why we use features for Metal/CUDA (conditional compilation)
  - How it enables different hardware acceleration paths
  - Resource: [Cargo Features Documentation](https://doc.rust-lang.org/cargo/reference/features.html)
  - See our implementation in `Cargo.toml` lines 37-44

- [ ] **Study whisper-rs**: Understand the transcription library
  - What is whisper-rs and how does it work?
  - How does it bind to whisper.cpp (C++ library)?
  - Why we chose local whisper over API
  - How Metal/CUDA acceleration works
  - Resource: [whisper-rs GitHub](https://github.com/tazz4843/whisper-rs)
  - See our implementation in `src/transcription.rs`

### 🛠️ Code Improvements
- [ ] **Remove camel case**: Review code for camelCase variables and convert to snake_case
  - Rust convention is `snake_case` for variables and functions
  - Check: `config_voice`, `config_audio` in main.rs
  - Run `cargo clippy` to find naming issues

### 🔒 Security Enhancement
- [ ] **Implement security check**: Ensure only authorized users can use the bot
  - Add `allowed_users` list to config.toml (Telegram user IDs)
  - Create middleware/filter to check user ID on every message
  - Reject unauthorized users with polite message
  - Consider: How to get your Telegram user ID? (Bot can log it, or use @userinfobot)
  - Implementation location: Create new filter in dptree chain or add check in handlers

### Write a good README TO install every thing
- Or maybe we can implement homebrew recipe? nice!
