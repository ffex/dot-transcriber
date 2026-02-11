pub mod corrector;
pub mod notes_reader;
pub mod note_writer;

pub use corrector::Corrector;
pub use notes_reader::{NotesReader, NoteMeta};
pub use note_writer::NoteWriter;

use anyhow::Result;

/// Tool trait for agent-orchestrated operations.
///
/// Not object-safe (associated types) — intentional.
/// The agent calls tools by concrete type, not `dyn Tool`.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    type Input: Send;
    type Output: Send;

    fn name(&self) -> &str;
    async fn run(&self, input: Self::Input) -> Result<Self::Output>;
}
