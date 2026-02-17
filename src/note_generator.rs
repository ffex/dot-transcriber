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
        let safe_title: String = self
            .title
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
        log::info!(
            "Agent: Step 1 - Correcting transcription (enabled={})",
            self.correction_enabled
        );
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
                log::info!(
                    "Agent: Step 2 - Reading existing notes index ({} notes)",
                    notes.len()
                );
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

        let llm_response = self
            .ollama
            .chat(ChatRequest {
                system_prompt,
                user_prompt,
                temperature: self.generation_temperature,
                top_p: self.generation_top_p,
                json_format: true,
            })
            .await
            .context("Agent: LLM note generation failed")?;

        let notes_response: NotesResponse = serde_json::from_str(&llm_response)
            .context("Agent: failed to parse notes JSON from LLM")?;

        let now = Utc::now();
        let notes: Vec<Note> = notes_response
            .notes
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

        // Step 3b: Post-process — inject [[links]] for existing note titles and cross-link batch notes
        let notes = Self::post_process_links(notes, &existing_notes);

        // Step 4: Save notes
        log::info!("Agent: Step 4 - Saving notes");
        let saved_paths = self
            .note_writer
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
        let mut prompt = String::new();

        // Existing notes context first — so the LLM sees them prominently
        if !existing_notes.is_empty() {
            prompt.push_str("## NOTE ESISTENTI NEL SISTEMA\n\n");
            prompt.push_str("Queste sono le note già presenti nel vault. DEVI consultare questa lista per i link interni e i related_notes.\n\n");

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

            prompt.push('\n');
        }

        prompt.push_str(r#"Sei un assistente esperto nella creazione di note strutturate per un sistema di gestione della conoscenza personale (second brain) in Obsidian.

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

## LINK INTERNI (OBBLIGATORIO)

Questa è una funzionalità CRITICA. Devi creare collegamenti tra le note usando la sintassi Obsidian `[[Titolo Nota]]`.

### Regole per i link inline nel contenuto:
- Quando nel contenuto fai riferimento a un concetto o argomento che corrisponde a una nota esistente, DEVI racchiuderlo in `[[Titolo Nota]]` usando il titolo ESATTO dalla lista delle note esistenti
- Inserisci i link in modo naturale nel testo, non forzarli dove non hanno senso
- Esempio: se esiste una nota "Architettura Microservizi", scrivi "...come descritto in [[Architettura Microservizi]]..."

### Regole per related_notes:
- DEVI popolare il campo "related_notes" con i titoli ESATTI delle note esistenti che sono tematicamente correlate
- Controlla i tag in comune e gli argomenti affini per identificare le correlazioni
- Non lasciare "related_notes" vuoto se ci sono note esistenti pertinenti

### Regole per note multiple dalla stessa trascrizione:
- Se crei più note dalla stessa trascrizione, DEVI farle riferimento tra loro con [[link]] nel contenuto
- Ogni nota deve menzionare le altre note generate nello stesso batch dove pertinente"#);

        prompt.push_str(r#"

Formato di output: JSON valido con array "notes" contenente oggetti con campi:
- "title" (stringa)
- "content" (markdown — DEVE contenere [[link]] a note esistenti e note sorelle dove pertinente)
- "tags" (array di stringhe)
- "related_notes" (array di stringhe — titoli ESATTI di note esistenti correlate, NON lasciare vuoto se ci sono correlazioni)

Rispondi SOLO con il JSON, senza testo aggiuntivo prima o dopo."#);

        prompt
    }

    /// Post-process notes to ensure internal links are present.
    ///
    /// 1. Scans each note's content for exact title matches of existing notes
    ///    and wraps unlinked mentions in `[[]]`.
    /// 2. Cross-links notes generated in the same batch: adds sibling titles
    ///    to `related_notes` when they share at least one tag.
    fn post_process_links(mut notes: Vec<Note>, existing_notes: &[NoteMeta]) -> Vec<Note> {
        // Collect all titles to link against: existing notes + batch siblings
        let existing_titles: Vec<&str> = existing_notes.iter().map(|n| n.title.as_str()).collect();
        let batch_titles: Vec<String> = notes.iter().map(|n| n.title.clone()).collect();
        let batch_tags: Vec<std::collections::HashSet<String>> = notes
            .iter()
            .map(|n| n.tags.iter().cloned().collect())
            .collect();

        for i in 0..notes.len() {
            // --- Inject [[links]] for existing note titles mentioned in content ---
            for title in &existing_titles {
                // Skip if already linked
                let wiki_link = format!("[[{}]]", title);
                if notes[i].content.contains(&wiki_link) {
                    continue;
                }
                // Replace plain mentions with [[links]] (case-sensitive exact match)
                if notes[i].content.contains(*title) {
                    notes[i].content = notes[i].content.replace(*title, &wiki_link);
                }
            }

            // --- Cross-link sibling notes from the same batch ---
            for j in 0..notes.len() {
                if i == j {
                    continue;
                }
                let sibling_title = &batch_titles[j];

                // Add to related_notes if they share at least one tag
                if !batch_tags[i].is_disjoint(&batch_tags[j])
                    && !notes[i].related_notes.contains(sibling_title)
                {
                    notes[i].related_notes.push(sibling_title.clone());
                }
            }
        }

        notes
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
        let existing = vec![NoteMeta {
            title: "Rust Tips".to_string(),
            date: "2024-01-15".to_string(),
            tags: vec!["rust".to_string(), "programming".to_string()],
            filename: "20240115_rust-tips.md".to_string(),
            source: "voice-memo".to_string(),
        }];
        let prompt = NoteGeneratorAgent::build_system_prompt(&existing);
        assert!(prompt.contains("NOTE ESISTENTI NEL SISTEMA"));
        assert!(prompt.contains("Rust Tips"));
        assert!(prompt.contains("rust, programming"));
        // New: verify internal links section is present
        assert!(prompt.contains("LINK INTERNI (OBBLIGATORIO)"));
        assert!(prompt.contains("[[Titolo Nota]]"));
        // Existing notes should appear before the main instructions
        let notes_pos = prompt.find("NOTE ESISTENTI").unwrap();
        let rules_pos = prompt.find("Regole per la creazione").unwrap();
        assert!(notes_pos < rules_pos, "Existing notes should appear before rules");
    }

    #[test]
    fn test_post_process_links_injects_wiki_links() {
        let existing = vec![NoteMeta {
            title: "Architettura Microservizi".to_string(),
            date: "2024-01-10".to_string(),
            tags: vec!["architettura".to_string()],
            filename: "arch.md".to_string(),
            source: "voice-memo".to_string(),
        }];
        let notes = vec![Note {
            title: "API Gateway".to_string(),
            content: "Il pattern API Gateway si integra con Architettura Microservizi per gestire il routing.".to_string(),
            tags: vec!["api".to_string()],
            date: Utc::now(),
            source: "voice-memo".to_string(),
            related_notes: vec![],
        }];

        let result = NoteGeneratorAgent::post_process_links(notes, &existing);
        assert!(result[0].content.contains("[[Architettura Microservizi]]"));
        // Should not double-wrap
        assert!(!result[0].content.contains("[[[["));
    }

    #[test]
    fn test_post_process_links_does_not_double_wrap() {
        let existing = vec![NoteMeta {
            title: "Rust Tips".to_string(),
            date: "2024-01-10".to_string(),
            tags: vec!["rust".to_string()],
            filename: "rust.md".to_string(),
            source: "voice-memo".to_string(),
        }];
        let notes = vec![Note {
            title: "Appunti".to_string(),
            content: "Vedi [[Rust Tips]] per dettagli.".to_string(),
            tags: vec!["rust".to_string()],
            date: Utc::now(),
            source: "voice-memo".to_string(),
            related_notes: vec![],
        }];

        let result = NoteGeneratorAgent::post_process_links(notes, &existing);
        assert!(result[0].content.contains("[[Rust Tips]]"));
        assert!(!result[0].content.contains("[[[[Rust Tips]]]]"));
    }

    #[test]
    fn test_post_process_cross_links_batch_notes() {
        let notes = vec![
            Note {
                title: "Nota A".to_string(),
                content: "Contenuto A".to_string(),
                tags: vec!["rust".to_string(), "coding".to_string()],
                date: Utc::now(),
                source: "voice-memo".to_string(),
                related_notes: vec![],
            },
            Note {
                title: "Nota B".to_string(),
                content: "Contenuto B".to_string(),
                tags: vec!["rust".to_string()],
                date: Utc::now(),
                source: "voice-memo".to_string(),
                related_notes: vec![],
            },
            Note {
                title: "Nota C".to_string(),
                content: "Contenuto C".to_string(),
                tags: vec!["unrelated".to_string()],
                date: Utc::now(),
                source: "voice-memo".to_string(),
                related_notes: vec![],
            },
        ];

        let result = NoteGeneratorAgent::post_process_links(notes, &[]);
        // A and B share "rust" tag — should be cross-linked
        assert!(result[0].related_notes.contains(&"Nota B".to_string()));
        assert!(result[1].related_notes.contains(&"Nota A".to_string()));
        // C has no shared tags — should not be linked
        assert!(!result[0].related_notes.contains(&"Nota C".to_string()));
        assert!(!result[2].related_notes.contains(&"Nota A".to_string()));
    }
}
