use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::Config;
use crate::ollama::{ChatRequest, OllamaClient};
use crate::tools::{Corrector, NoteMeta, NoteWriter, NotesReader, Tool};

/// Represents a generated note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub date: DateTime<Utc>,
    pub source: String,
    #[serde(default)]
    pub related_notes: Vec<String>,
}

impl Note {
    /// Convert note to markdown with YAML frontmatter.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("---\n");
        md.push_str(&format!("title: \"{}\"\n", self.title));
        md.push_str(&format!("date: {}\n", self.date.format("%Y-%m-%d")));
        md.push_str(&format!("source: {}\n", self.source));

        if !self.tags.is_empty() {
            md.push_str("tags:\n");
            for tag in &self.tags {
                md.push_str(&format!("  - {}\n", tag));
            }
        }

        if !self.related_notes.is_empty() {
            md.push_str("related:\n");
            for rel in &self.related_notes {
                md.push_str(&format!("  - \"{}\"\n", rel));
            }
        }

        md.push_str("---\n\n");
        md.push_str(&self.content);

        // Render related notes as Obsidian wiki-links
        if !self.related_notes.is_empty() {
            md.push_str("\n\n---\n\n## Note correlate\n\n");
            for rel in &self.related_notes {
                md.push_str(&format!("- [[{}]]\n", rel));
            }
        }

        md
    }

    /// Generate a sanitized filename for this note.
    pub fn generate_filename(&self) -> String {
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

        let safe_title = if safe_title.len() > 50 {
            &safe_title[..50]
        } else {
            &safe_title
        };

        format!("{}_{}.md", self.date.format("%Y%m%d_%H%M%S"), safe_title)
    }
}

/// Result returned by the agent after processing a transcript.
pub struct AgentResult {
    pub notes: Vec<Note>,
    pub saved_paths: Vec<PathBuf>,
    pub cleaned_transcript: String,
    pub raw_transcript: String,
}

/// Agent that orchestrates tools to generate notes from voice transcripts.
pub struct NoteGeneratorAgent {
    corrector: Corrector,
    notes_reader: NotesReader,
    note_writer: NoteWriter,
    ollama: OllamaClient,
    notes_dir: String,
    correction_enabled: bool,
    generation_temperature: f32,
    generation_top_p: f32,
}

impl NoteGeneratorAgent {
    pub fn new(config: &Config) -> Self {
        let corrector_ollama = OllamaClient::new(
            config.ai_model.endpoint.clone(),
            config.ai_model.model.clone(),
        );
        let agent_ollama = OllamaClient::new(
            config.ai_model.endpoint.clone(),
            config.ai_model.model.clone(),
        );

        Self {
            corrector: Corrector::new(
                corrector_ollama,
                config.correction.temperature,
                config.correction.top_p,
            ),
            notes_reader: NotesReader::new(),
            note_writer: NoteWriter::new(),
            ollama: agent_ollama,
            notes_dir: config.output.notes_dir.clone(),
            correction_enabled: config.correction.enabled,
            generation_temperature: config.notes_generation.temperature,
            generation_top_p: config.notes_generation.top_p,
        }
    }

    /// Process a raw transcript through the full agent pipeline.
    pub async fn process_transcript(&self, raw_transcript: String) -> Result<AgentResult> {
        // Step 1: Correct transcription (if enabled)
        log::info!("Agent: Step 1 - Correcting transcription (enabled={})", self.correction_enabled);
        let cleaned_transcript = if self.correction_enabled {
            match self.corrector.run(raw_transcript.clone()).await {
                Ok(cleaned) => cleaned,
                Err(e) => {
                    log::warn!("Agent: correction failed, using raw transcript: {}", e);
                    raw_transcript.clone()
                }
            }
        } else {
            raw_transcript.clone()
        };

        // Step 2: Read existing notes index
        log::info!("Agent: Step 2 - Reading existing notes index");
        let existing_notes = match self.notes_reader.run(self.notes_dir.clone()).await {
            Ok(notes) => {
                log::info!("Agent: Step 2 - Reading existing notes index ({} notes)", notes.len());
                notes
            }
            Err(e) => {
                log::warn!("Agent: failed to read existing notes: {}", e);
                Vec::new()
            }
        };

        // Step 3: Generate notes with LLM (context-aware)
        log::info!("Agent: Step 3 - Generating notes with LLM");
        let system_prompt = Self::build_system_prompt(&existing_notes);
        let user_prompt = Self::build_user_prompt(&cleaned_transcript);

        let llm_response = self.ollama.chat(ChatRequest {
            system_prompt,
            user_prompt,
            temperature: self.generation_temperature,
            top_p: self.generation_top_p,
            json_format: true,
        }).await.context("Agent: LLM note generation failed")?;

        let notes_response: NotesResponse = serde_json::from_str(&llm_response)
            .context("Agent: failed to parse notes JSON from LLM")?;

        let now = Utc::now();
        let notes: Vec<Note> = notes_response.notes
            .into_iter()
            .map(|nd| Note {
                title: nd.title,
                content: nd.content,
                tags: nd.tags,
                date: now,
                source: "voice-memo".to_string(),
                related_notes: nd.related_notes.unwrap_or_default(),
            })
            .collect();

        log::info!("Agent: Step 3 - Generated {} note(s)", notes.len());

        // Step 4: Save notes
        log::info!("Agent: Step 4 - Saving notes");
        let saved_paths = self.note_writer
            .run((notes.clone(), self.notes_dir.clone()))
            .await
            .context("Agent: failed to save notes")?;

        Ok(AgentResult {
            notes,
            saved_paths,
            cleaned_transcript,
            raw_transcript,
        })
    }

    /// Build the system prompt, injecting existing notes context.
    fn build_system_prompt(existing_notes: &[NoteMeta]) -> String {
        let base = r#"Sei un assistente esperto nella creazione di note strutturate per un sistema di gestione della conoscenza personale (second brain).

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
- Se ci sono task o azioni da fare, evidenziali chiaramente"#;

        let mut prompt = base.to_string();

        if !existing_notes.is_empty() {
            prompt.push_str("\n\n## NOTE ESISTENTI NEL SISTEMA\n\n");
            prompt.push_str("Di seguito le note già presenti. Riutilizza i tag esistenti quando pertinente e identifica eventuali note correlate nel campo \"related_notes\" (usa i titoli esatti).\n\n");

            for note in existing_notes {
                prompt.push_str(&format!("- **{}**", note.title));
                if !note.date.is_empty() {
                    prompt.push_str(&format!(" ({})", note.date));
                }
                if !note.tags.is_empty() {
                    prompt.push_str(&format!(" [{}]", note.tags.join(", ")));
                }
                prompt.push('\n');
            }
        }

        prompt.push_str(&format!(
            r#"

Formato di output: JSON valido con array "notes" contenente oggetti con campi:
- "title" (stringa)
- "content" (markdown)
- "tags" (array di stringhe)
- "related_notes" (array di stringhe — titoli di note esistenti correlate, oppure array vuoto)

Rispondi SOLO con il JSON, senza testo aggiuntivo prima o dopo."#
        ));

        prompt
    }

    /// Build the user prompt from the transcript.
    fn build_user_prompt(transcript: &str) -> String {
        format!(
            "Trascrizione del messaggio vocale:\n\n---\n{}\n---\n\nCrea note strutturate da questa trascrizione.",
            transcript
        )
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
    related_notes: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename() {
        let note = Note {
            title: "Test Note: Example!".to_string(),
            content: "content".to_string(),
            tags: vec![],
            date: chrono::DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: "voice-memo".to_string(),
            related_notes: vec![],
        };
        let filename = note.generate_filename();
        assert!(filename.starts_with("20240115_103000_"));
        assert!(filename.ends_with(".md"));
        assert!(!filename.contains('!'));
        assert!(!filename.contains(':'));
    }

    #[test]
    fn test_to_markdown_with_related_notes() {
        let note = Note {
            title: "Test".to_string(),
            content: "Some content".to_string(),
            tags: vec!["rust".to_string()],
            date: Utc::now(),
            source: "voice-memo".to_string(),
            related_notes: vec!["Other Note".to_string(), "Another".to_string()],
        };
        let md = note.to_markdown();
        assert!(md.contains("[[Other Note]]"));
        assert!(md.contains("[[Another]]"));
        assert!(md.contains("related:"));
    }

    #[test]
    fn test_build_system_prompt_without_existing() {
        let prompt = NoteGeneratorAgent::build_system_prompt(&[]);
        assert!(!prompt.contains("NOTE ESISTENTI"));
        assert!(prompt.contains("related_notes"));
    }

    #[test]
    fn test_build_system_prompt_with_existing() {
        let existing = vec![
            NoteMeta {
                title: "Rust Tips".to_string(),
                date: "2024-01-15".to_string(),
                tags: vec!["rust".to_string(), "programming".to_string()],
                filename: "20240115_rust-tips.md".to_string(),
                source: "voice-memo".to_string(),
            },
        ];
        let prompt = NoteGeneratorAgent::build_system_prompt(&existing);
        assert!(prompt.contains("NOTE ESISTENTI NEL SISTEMA"));
        assert!(prompt.contains("Rust Tips"));
        assert!(prompt.contains("rust, programming"));
    }
}
