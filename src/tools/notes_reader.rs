use anyhow::Result;
use serde::Deserialize;
use std::path::Path;
use super::Tool;

/// Metadata extracted from a note's YAML frontmatter.
#[derive(Debug, Clone)]
pub struct NoteMeta {
    pub title: String,
    pub date: String,
    pub tags: Vec<String>,
    pub filename: String,
    pub source: String,
}

/// Raw YAML frontmatter structure for deserialization.
#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: Option<String>,
    date: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    source: Option<String>,
}

/// Scans a notes directory and reads YAML frontmatter from .md files.
pub struct NotesReader;

impl NotesReader {
    pub fn new() -> Self {
        Self
    }

    /// Parse YAML frontmatter from markdown content between `---` markers.
    fn parse_frontmatter(content: &str) -> Option<Frontmatter> {
        let content = content.trim_start();
        if !content.starts_with("---") {
            return None;
        }

        let after_first = &content[3..];
        let end = after_first.find("---")?;
        let yaml_str = &after_first[..end];

        serde_yaml::from_str(yaml_str).ok()
    }
}

#[async_trait::async_trait]
impl Tool for NotesReader {
    type Input = String;
    type Output = Vec<NoteMeta>;

    fn name(&self) -> &str {
        "notes_reader"
    }

    async fn run(&self, notes_dir: String) -> Result<Vec<NoteMeta>> {
        let dir = Path::new(&notes_dir);
        if !dir.exists() {
            log::info!("NotesReader: directory does not exist yet: {}", notes_dir);
            return Ok(Vec::new());
        }

        let mut notes = Vec::new();

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("NotesReader: failed to read dir entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let filename = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("NotesReader: failed to read {}: {}", filename, e);
                    continue;
                }
            };

            match Self::parse_frontmatter(&content) {
                Some(fm) => {
                    notes.push(NoteMeta {
                        title: fm.title.unwrap_or_else(|| filename.clone()),
                        date: fm.date.unwrap_or_default(),
                        tags: fm.tags,
                        filename: filename.clone(),
                        source: fm.source.unwrap_or_default(),
                    });
                }
                None => {
                    log::warn!("NotesReader: no valid frontmatter in {}", filename);
                }
            }
        }

        log::info!("NotesReader: found {} existing notes", notes.len());
        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = r#"---
title: "Test Note"
date: 2024-01-15
source: voice-memo
tags:
  - rust
  - coding
---

# Some content here
"#;
        let fm = NotesReader::parse_frontmatter(content).unwrap();
        assert_eq!(fm.title.unwrap(), "Test Note");
        assert_eq!(fm.date.unwrap(), "2024-01-15");
        assert_eq!(fm.tags, vec!["rust", "coding"]);
        assert_eq!(fm.source.unwrap(), "voice-memo");
    }

    #[test]
    fn test_parse_frontmatter_missing_fields() {
        let content = "---\ntitle: \"Minimal\"\n---\n\nContent";
        let fm = NotesReader::parse_frontmatter(content).unwrap();
        assert_eq!(fm.title.unwrap(), "Minimal");
        assert!(fm.tags.is_empty());
        assert!(fm.date.is_none());
    }

    #[test]
    fn test_parse_frontmatter_no_markers() {
        let content = "# Just a heading\nNo frontmatter here.";
        assert!(NotesReader::parse_frontmatter(content).is_none());
    }
}
