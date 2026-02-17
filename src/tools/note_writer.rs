use anyhow::{Context, Result};
use std::path::PathBuf;
use crate::note_generator::Note;
use super::Tool;

/// Saves notes to the filesystem as Markdown files.
pub struct NoteWriter;

impl NoteWriter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for NoteWriter {
    type Input = (Vec<Note>, String);
    type Output = Vec<PathBuf>;

    fn name(&self) -> &str {
        "note_writer"
    }

    async fn run(&self, input: (Vec<Note>, String)) -> Result<Vec<PathBuf>> {
        let (notes, notes_dir) = input;

        std::fs::create_dir_all(&notes_dir)
            .context("Failed to create notes directory")?;

        let mut saved_paths = Vec::new();

        for note in &notes {
            let filename = note.generate_filename();
            let filepath = PathBuf::from(&notes_dir).join(&filename);

            std::fs::write(&filepath, note.to_markdown())
                .with_context(|| format!("Failed to write note: {}", filename))?;

            log::info!("NoteWriter: saved {}", filepath.display());
            saved_paths.push(filepath);
        }

        log::info!("NoteWriter: saved {} note(s)", saved_paths.len());
        Ok(saved_paths)
    }
}
