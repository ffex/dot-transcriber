use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub telegram: TelegramConfig,
    pub transcription: TranscriptionConfig,
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
    pub service: String,
    pub language: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiModelConfig {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub temperature: f32,
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
            service = "whisper_api"
            language = "it"
            model = "whisper-1"

            [ai_model]
            provider = "ollama"
            model = "llama2"
            endpoint = "http://localhost:11434"
            temperature = 0.7

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
    }
}
