use anyhow::Result;
use crate::ollama::{OllamaClient, ChatRequest};
use super::Tool;

/// Corrects transcription errors using an LLM.
pub struct Corrector {
    ollama: OllamaClient,
    temperature: f32,
    top_p: f32,
}

impl Corrector {
    pub fn new(ollama: OllamaClient, temperature: f32, top_p: f32) -> Self {
        Self { ollama, temperature, top_p }
    }

    fn system_prompt() -> &'static str {
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

    fn user_prompt(transcript: &str) -> String {
        format!(
            "Trascrizione automatica da correggere:\n\n---\n{}\n---\n\nCorreggi eventuali errori mantenendo il significato originale.",
            transcript
        )
    }
}

#[async_trait::async_trait]
impl Tool for Corrector {
    type Input = String;
    type Output = String;

    fn name(&self) -> &str {
        "corrector"
    }

    async fn run(&self, raw_transcript: String) -> Result<String> {
        log::info!("Corrector: cleaning transcription with LLM...");

        let result = self.ollama.chat(ChatRequest {
            system_prompt: Self::system_prompt().to_string(),
            user_prompt: Self::user_prompt(&raw_transcript),
            temperature: self.temperature,
            top_p: self.top_p,
            json_format: false,
        }).await?;

        log::info!("Corrector: transcription cleaned ({} → {} chars)",
                   raw_transcript.len(), result.len());

        Ok(result)
    }
}
