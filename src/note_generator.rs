use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::{DateTime, Utc};

use crate::config::{CorrectionConfig, NotesGenerationConfig};

/// Represents a generated note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub date: DateTime<Utc>,
    pub source: String, // "voice-memo"
}

impl Note {
    /// Convert note to markdown with frontmatter
    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();

        // Frontmatter
        markdown.push_str("---\n");
        markdown.push_str(&format!("title: \"{}\"\n", self.title));
        markdown.push_str(&format!("date: {}\n", self.date.format("%Y-%m-%d")));
        markdown.push_str(&format!("source: {}\n", self.source));

        if !self.tags.is_empty() {
            markdown.push_str("tags:\n");
            for tag in &self.tags {
                markdown.push_str(&format!("  - {}\n", tag));
            }
        }

        markdown.push_str("---\n\n");

        // Content
        markdown.push_str(&self.content);

        markdown
    }

    /// Save note to file
    pub fn save_to_file(&self, directory: &str) -> Result<std::path::PathBuf> {
        use std::fs;

        // Create directory if it doesn't exist
        fs::create_dir_all(directory)
            .context("Failed to create notes directory")?;

        // Generate filename from title and date
        let filename = self.generate_filename();
        let filepath = Path::new(directory).join(&filename);

        // Write markdown to file
        fs::write(&filepath, self.to_markdown())
            .context("Failed to write note to file")?;

        log::info!("Note saved to: {}", filepath.display());
        Ok(filepath)
    }

    fn generate_filename(&self) -> String {
        // Sanitize title for filename
        let safe_title: String = self.title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .replace("  ", " ")
            .trim()
            .to_lowercase()
            .replace(' ', "-");

        // Limit length
        let safe_title = if safe_title.len() > 50 {
            &safe_title[..50]
        } else {
            &safe_title
        };

        format!("{}_{}.md", self.date.format("%Y%m%d_%H%M%S"), safe_title)
    }
}

/// AI Provider trait for generating notes
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_notes(&self, transcript: &str) -> Result<Vec<Note>>;
}

/// Ollama provider for local/remote LLM
pub struct OllamaProvider {
    endpoint: String,
    model: String,
    client: reqwest::Client,
    correction_temperature: f32,
    correction_top_p: f32,
    notes_temperature: f32,
    notes_top_p: f32,
}

impl OllamaProvider {
    pub fn new(
        endpoint: String,
        model: String,
        correction: &CorrectionConfig,
        notes_generation: &NotesGenerationConfig,
    ) -> Self {
        Self {
            endpoint,
            model,
            client: reqwest::Client::new(),
            correction_temperature: correction.temperature,
            correction_top_p: correction.top_p,
            notes_temperature: notes_generation.temperature,
            notes_top_p: notes_generation.top_p,
        }
    }

    /// Get the system prompt for transcription cleanup
    fn get_cleanup_system_prompt() -> &'static str {
        r#"Sei un esperto correttore di trascrizioni vocali italiane.

Il tuo compito è correggere errori di trascrizione automatica mantenendo il significato originale.

Correzioni da fare:
- Parole mal riconosciute dal sistema di trascrizione
- Errori grammaticali dovuti alla trascrizione automatica
- Punteggiatura mancante o errata
- Maiuscole appropriate (nomi propri, inizio frasi)
- Parole incomplete o frammentate

IMPORTANTE:
- NON aggiungere informazioni che non ci sono
- NON cambiare il significato originale
- NON rimuovere dettagli importanti
- Mantieni lo stile colloquiale se presente
- Se una parola sembra tecnica o è un nome proprio, mantienila anche se sembra strana

Rispondi SOLO con il testo corretto, senza commenti o spiegazioni."#
    }

    /// Get the system prompt for note generation
    fn get_system_prompt() -> &'static str {
        r#"Sei un assistente esperto nella creazione di note strutturate per un sistema di gestione della conoscenza personale (second brain).

Il tuo compito è:
1. Analizzare la trascrizione di un messaggio vocale
2. Identificare i concetti chiave, idee e informazioni importanti
3. Creare una o più note in formato Markdown ben strutturate

Regole per la creazione delle note:
- Se la trascrizione contiene più argomenti distinti, crea note separate per ciascuno
- Ogni nota deve avere un titolo chiaro e descrittivo
- Struttura il contenuto con headers (##), elenchi puntati e formattazione appropriata
- Suggerisci 2-5 tag rilevanti per ogni nota
- Mantieni il tono e l'intento originale del messaggio
- Se ci sono task o azioni da fare, evidenziali chiaramente

Formato di output: JSON valido con array "notes" contenente oggetti con campi "title", "content" (markdown) e "tags" (array di stringhe).

Rispondi SOLO con il JSON, senza testo aggiuntivo prima o dopo."#
    }

    /// Generate prompt for the user message
    fn generate_user_prompt(transcript: &str) -> String {
        format!(
            "Trascrizione del messaggio vocale:\n\n---\n{}\n---\n\nCrea note strutturate da questa trascrizione.",
            transcript
        )
    }

    /// Generate prompt for transcription cleanup
    fn generate_cleanup_prompt(transcript: &str) -> String {
        format!(
            "Trascrizione automatica da correggere:\n\n---\n{}\n---\n\nCorreggi eventuali errori mantenendo il significato originale.",
            transcript
        )
    }

    /// Clean and fix transcription errors using LLM
    pub async fn cleanup_transcription(&self, raw_transcript: &str) -> Result<String> {
        log::info!("Cleaning transcription with LLM...");

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": Self::get_cleanup_system_prompt()
                },
                {
                    "role": "user",
                    "content": Self::generate_cleanup_prompt(raw_transcript)
                }
            ],
            "stream": false,
            "options": {
                "temperature": self.correction_temperature,
                "top_p": self.correction_top_p
            }
        });

        let response = self.client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&request_body)
            .send()
            .await
            .context("Failed to send cleanup request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama cleanup API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await
            .context("Failed to parse Ollama cleanup response")?;

        let cleaned = response_json["message"]["content"]
            .as_str()
            .context("No content in Ollama cleanup response")?
            .to_string();

        log::info!("Transcription cleaned successfully");
        log::debug!("Original length: {} chars, Cleaned length: {} chars",
                   raw_transcript.len(), cleaned.len());

        Ok(cleaned)
    }
}

#[async_trait::async_trait]
impl AiProvider for OllamaProvider {
    async fn generate_notes(&self, transcript: &str) -> Result<Vec<Note>> {
        log::info!("Generating notes with Ollama model: {}", self.model);

        // Prepare the request
        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": Self::get_system_prompt()
                },
                {
                    "role": "user",
                    "content": Self::generate_user_prompt(transcript)
                }
            ],
            "stream": false,
            "format": "json",
            "options": {
                "temperature": self.notes_temperature,
                "top_p": self.notes_top_p
            }
        });

        log::debug!("Sending request to Ollama: {}", self.endpoint);

        // Call Ollama API
        let response = self.client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await
            .context("Failed to parse Ollama response")?;

        log::debug!("Ollama response: {}", response_json);

        // Extract the message content
        let content = response_json["message"]["content"]
            .as_str()
            .context("No content in Ollama response")?;

        // Parse the JSON response from the model
        let notes_response: NotesResponse = serde_json::from_str(content)
            .context("Failed to parse notes from model response")?;

        // Convert to Note structs
        let now = Utc::now();
        let notes: Vec<Note> = notes_response.notes
            .into_iter()
            .map(|note_data| Note {
                title: note_data.title,
                content: note_data.content,
                tags: note_data.tags,
                date: now,
                source: "voice-memo".to_string(),
            })
            .collect();

        log::info!("Generated {} note(s)", notes.len());

        Ok(notes)
    }
}

#[derive(Debug, Deserialize)]
struct NotesResponse {
    notes: Vec<NoteData>,
}

#[derive(Debug, Deserialize)]
struct NoteData {
    title: String,
    content: String,
    tags: Vec<String>,
}
