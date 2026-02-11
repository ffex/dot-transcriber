use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub telegram: TelegramConfig,
    pub transcription: TranscriptionConfig,
    pub correction: CorrectionConfig,
    pub notes_generation: NotesGenerationConfig,
    pub ai_model: AiModelConfig,
    pub output: OutputConfig,
    pub features: FeaturesConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub poll_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TranscriptionConfig {
    pub provider: String,
    pub language: String,
    #[serde(default)]
    pub model_path: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorrectionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_correction_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NotesGenerationConfig {
    #[serde(default = "default_notes_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
}

fn default_true() -> bool { true }
fn default_correction_temperature() -> f32 { 0.3 }
fn default_notes_temperature() -> f32 { 0.7 }
fn default_top_p() -> f32 { 0.9 }

#[derive(Debug, Deserialize, Clone)]
pub struct AiModelConfig {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OutputConfig {
    pub notes_dir: String,
    pub tasks_dir: String,
    pub temp_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeaturesConfig {
    pub enable_task_extraction: bool,
    pub enable_auto_tags: bool,
    pub max_audio_size_mb: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub log_file: String,
}

impl Config {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file. Make sure config.toml exists.")?;

        let mut config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;

        // Override with environment variable if set
        if let Ok(token) = std::env::var("TELOXIDE_TOKEN") {
            config.telegram.bot_token = token;
        }

        Ok(config)
    }

    /// Create output directories if they don't exist
    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.output.notes_dir)
            .context("Failed to create notes directory")?;
        fs::create_dir_all(&self.output.tasks_dir)
            .context("Failed to create tasks directory")?;
        fs::create_dir_all(&self.output.temp_dir)
            .context("Failed to create temp directory")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let toml_str = r#"
            [telegram]
            bot_token = "test_token"
            poll_interval = 2

            [transcription]
            provider = "whisper_local"
            language = "it"
            model_path = "./models/ggml-large-v3.bin"

            [correction]
            enabled = true
            temperature = 0.3
            top_p = 0.9

            [notes_generation]
            temperature = 0.7
            top_p = 0.9

            [ai_model]
            provider = "ollama_local"
            model = "llama3.2:3b"
            endpoint = "http://localhost:11434"

            [output]
            notes_dir = "./output/notes"
            tasks_dir = "./output/tasks"
            temp_dir = "./temp"

            [features]
            enable_task_extraction = true
            enable_auto_tags = true
            max_audio_size_mb = 20

            [logging]
            level = "info"
            log_file = "./dot.log"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.telegram.bot_token, "test_token");
        assert_eq!(config.transcription.language, "it");
        assert_eq!(config.transcription.provider, "whisper_local");
        assert_eq!(config.transcription.model_path.as_deref(), Some("./models/ggml-large-v3.bin"));
        assert_eq!(config.correction.enabled, true);
        assert_eq!(config.correction.temperature, 0.3);
        assert_eq!(config.notes_generation.temperature, 0.7);
    }

    #[test]
    fn test_config_deepgram_provider() {
        let toml_str = r#"
            [telegram]
            bot_token = "test_token"
            poll_interval = 2

            [transcription]
            provider = "deepgram"
            language = "it"
            api_key_env = "DEEPGRAM_API_KEY"
            model = "nova-2"

            [correction]
            enabled = true

            [notes_generation]
            temperature = 0.7

            [ai_model]
            provider = "ollama_local"
            model = "llama3.2:3b"
            endpoint = "http://localhost:11434"

            [output]
            notes_dir = "./output/notes"
            tasks_dir = "./output/tasks"
            temp_dir = "./temp"

            [features]
            enable_task_extraction = true
            enable_auto_tags = true
            max_audio_size_mb = 20

            [logging]
            level = "info"
            log_file = "./dot.log"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.transcription.provider, "deepgram");
        assert_eq!(config.transcription.api_key_env.as_deref(), Some("DEEPGRAM_API_KEY"));
        assert_eq!(config.transcription.model.as_deref(), Some("nova-2"));
        assert_eq!(config.transcription.model_path, None);
        assert_eq!(config.ai_model.endpoint, "http://localhost:11434");
    }

    #[test]
    fn test_config_groq_provider() {
        let toml_str = r#"
            [telegram]
            bot_token = "test_token"
            poll_interval = 2

            [transcription]
            provider = "groq"
            language = "it"
            api_key_env = "GROQ_API_KEY"
            model = "whisper-large-v3-turbo"

            [correction]
            enabled = false

            [notes_generation]
            temperature = 0.5

            [ai_model]
            provider = "ollama_local"
            model = "llama3.2:3b"
            endpoint = "http://localhost:11434"

            [output]
            notes_dir = "./output/notes"
            tasks_dir = "./output/tasks"
            temp_dir = "./temp"

            [features]
            enable_task_extraction = true
            enable_auto_tags = true
            max_audio_size_mb = 20

            [logging]
            level = "info"
            log_file = "./dot.log"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.transcription.provider, "groq");
        assert_eq!(config.transcription.api_key_env.as_deref(), Some("GROQ_API_KEY"));
        assert_eq!(config.transcription.model.as_deref(), Some("whisper-large-v3-turbo"));
        assert_eq!(config.transcription.model_path, None);
        assert_eq!(config.correction.enabled, false);
        // Check defaults applied
        assert_eq!(config.correction.temperature, 0.3);
        assert_eq!(config.correction.top_p, 0.9);
        assert_eq!(config.notes_generation.temperature, 0.5);
        assert_eq!(config.notes_generation.top_p, 0.9);
    }
}
