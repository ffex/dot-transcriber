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
- **Bot**: Telegram Bot API
- **Transcription**: TBD (OpenAI Whisper API, local Whisper, or alternatives)
- **AI Processing**: Ollama (local) or API-based models (OpenAI, Anthropic)
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
**Status**: 🔴 Not Started

**Goals**:
- Research transcription options
- Implement Italian language transcription
- Handle various audio formats

**Tasks**:
- [ ] Research transcription options:
  - OpenAI Whisper API (cloud)
  - Local Whisper (whisper.cpp, faster-whisper)
  - Other APIs (AssemblyAI, Deepgram)
- [ ] Choose transcription solution
- [ ] Implement transcription integration
- [ ] Add Italian language support
- [ ] Handle audio format conversions if needed
- [ ] Add error handling for transcription failures

**Dependencies**: Phase 1 complete

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

**Active Phase**: Phase 1 Complete ✅ - Ready for Phase 2
**Last Updated**: 2026-01-11

### What We Have
- ✅ Rust project initialized (`dot_transcriber`)
- ✅ Project tracking documents (WhereAreWe.md, ClaudePrompts.md, AIPrompts.md)
- ✅ Configuration system (config.toml + .env support)
- ✅ Telegram bot fully functional with teloxide
- ✅ Commands: /start, /help, /status (all in Italian)
- ✅ Audio message detection (voice messages & audio files)
- ✅ Message routing with dptree
- ✅ Modular code structure (main.rs, config.rs, handlers.rs)
- ✅ Logging system with pretty_env_logger
- ✅ Bot tested and working!

### What We're Working On
- Preparing for Phase 2: Audio Transcription
- Code improvements and security enhancements

### Next Steps
1. Implement user authentication (only authorized users can use bot)
2. Research and choose transcription service (Whisper API vs local)
3. Implement audio file download from Telegram
4. Integrate transcription service
5. Begin Phase 2: Audio Transcription

---

## Configuration Needs

**Required Secrets**:
- `TELEGRAM_BOT_TOKEN`: From BotFather
- `OPENAI_API_KEY`: (if using Whisper API or GPT)
- `ANTHROPIC_API_KEY`: (if using Claude API)

**Configuration Options**:
- Output directory for notes
- Model selection (local/cloud)
- Language settings
- Note templates

---

## Notes & Decisions Log

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

1. **Transcription Choice**: Local Whisper vs API? (Consider: cost, speed, quality, privacy)
2. **Model Choice**: Ollama local vs cloud API? (Consider: similar tradeoffs)
3. **Note Organization**: Single file per recording or aggregate by date/topic?
4. **Note Format**: What frontmatter? Tags, dates, categories?
5. **Deployment**: Run on personal server, cloud VPS, or local machine?

---

## Resources & References

- [Telegram Bot API Docs](https://core.telegram.org/bots/api)
- [teloxide - Rust Telegram Bot Framework](https://github.com/teloxide/teloxide)
- [dptree - Declarative handler trees](https://github.com/teloxide/dptree)
- [Whisper by OpenAI](https://openai.com/research/whisper)
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
