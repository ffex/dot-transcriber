use teloxide::{prelude::*, types::Me};
use crate::config::Config;
use crate::transcription;
use crate::note_generator::{AiProvider, OllamaProvider};

/// Handler for /start command
pub async fn start_handler(bot: Bot, msg: Message, me: Me) -> ResponseResult<()> {
    let text = format!(
        "👋 Ciao! Sono {}, il tuo assistente per la trascrizione vocale.\n\n\
        Inviami un messaggio vocale e lo trasformerò in note strutturate!\n\n\
        Comandi disponibili:\n\
        /start - Mostra questo messaggio\n\
        /help - Aiuto e istruzioni\n\
        /status - Stato del bot",
        me.username()
    );

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

/// Handler for /help command
pub async fn help_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = "📖 Come usare Dot:\n\n\
        1️⃣ Registra un messaggio vocale\n\
        2️⃣ Inviamelo qui in chat\n\
        3️⃣ Aspetta mentre lo trascrivo\n\
        4️⃣ Riceverai note strutturate in formato Markdown\n\n\
        💡 Funzionalità:\n\
        - Trascrizione automatica (italiano)\n\
        - Generazione di note strutturate\n\
        - Estrazione di task (per progetti di sviluppo)\n\
        - Formato compatibile con Obsidian\n\n\
        ⚙️ Configurazione:\n\
        - Lingua: Italiano\n\
        - Dimensione max audio: 20MB\n\
        - Formato output: Markdown (.md)\n\n\
        Problemi? Contatta il tuo amministratore.";

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

/// Handler for /status command
pub async fn status_handler(bot: Bot, msg: Message, config: Config) -> ResponseResult<()> {
    let text = format!(
        "🤖 Stato Bot\n\n\
        ✅ Online e funzionante\n\
        📝 Servizio trascrizione: {}\n\
        🤖 AI Provider: {}\n\
        📁 Directory note: {}\n\
        🔧 Task extraction: {}\n\n\
        Pronto a ricevere messaggi vocali!",
        config.transcription.service,
        config.ai_model.provider,
        config.output.notes_dir,
        if config.features.enable_task_extraction { "Abilitata" } else { "Disabilitata" }
    );

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

/// Handler for audio/voice messages
pub async fn audio_handler(bot: Bot, msg: Message, config: Config) -> ResponseResult<()> {
    log::info!("Received audio message from user {}", msg.chat.id);

    // Send acknowledgment
    let ack_msg = bot.send_message(msg.chat.id, "🎤 Messaggio vocale ricevuto! Sto trascrivendo...").await?;

    // Get the file info from the message
    let file_info = if let Some(voice) = msg.voice() {
        Some(voice.file.clone())
    } else if let Some(audio) = msg.audio() {
        Some(audio.file.clone())
    } else {
        None
    };

    if file_info.is_none() {
        bot.send_message(msg.chat.id, "❌ Errore: Nessun file audio trovato nel messaggio.")
            .await?;
        return Ok(());
    }

    // Get full file information
    let file_meta = file_info.unwrap();
    let file = match bot.get_file(&file_meta.id).await {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to get file info: {}", e);
            bot.send_message(msg.chat.id, "❌ Errore nel recupero del file audio.")
                .await?;
            return Ok(());
        }
    };

    // Transcribe the audio
    match transcription::transcribe_audio(
        &bot,
        &file,
        &config.output.temp_dir,
        &config.transcription.model_path,
        &config.transcription.language,
    ).await {
        Ok(raw_transcript) => {
            log::info!("Transcription successful for user {}: {} chars",
                       msg.chat.id, raw_transcript.len());

            // Update status message - cleanup phase
            let _ = bot.edit_message_text(
                msg.chat.id,
                ack_msg.id,
                "✅ Trascritto! Correggo eventuali errori..."
            ).await;

            // Initialize Ollama provider
            let ollama = OllamaProvider::new(
                config.ai_model.endpoint.clone(),
                config.ai_model.model.clone(),
            );

            // Step 1: Clean the transcription
            let cleaned_transcript = match ollama.cleanup_transcription(&raw_transcript).await {
                Ok(cleaned) => {
                    log::info!("Transcription cleaned successfully");
                    cleaned
                }
                Err(e) => {
                    log::warn!("Failed to clean transcription, using raw: {}", e);
                    raw_transcript.clone() // Fallback to raw if cleanup fails
                }
            };

            // Update status message - note generation phase
            let _ = bot.edit_message_text(
                msg.chat.id,
                ack_msg.id,
                "✅ Testo corretto! Genero le note..."
            ).await;

            // Step 2: Generate notes from cleaned transcript
            match ollama.generate_notes(&cleaned_transcript).await {
                Ok(notes) => {
                    // Delete status message
                    let _ = bot.delete_message(msg.chat.id, ack_msg.id).await;

                    // Save notes to files
                    let mut saved_files = Vec::new();
                    for note in &notes {
                        match note.save_to_file(&config.output.notes_dir) {
                            Ok(path) => saved_files.push(path),
                            Err(e) => log::error!("Failed to save note: {}", e),
                        }
                    }

                    // Send success message with note details
                    let mut response = format!(
                        "🎉 Completato!\n\n📝 {} nota/e generata/e:\n\n",
                        notes.len()
                    );

                    for (i, note) in notes.iter().enumerate() {
                        response.push_str(&format!("{}. **{}**\n", i + 1, note.title));
                        response.push_str(&format!("   Tags: {}\n", note.tags.join(", ")));
                        response.push_str(&format!("   File: {}\n\n",
                            saved_files.get(i)
                                .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                                .unwrap_or_else(|| "errore".to_string())
                        ));
                    }

                    // Show both raw and cleaned transcription if different
                    if cleaned_transcript != raw_transcript {
                        response.push_str("\n📊 Trascrizione (corretta):\n");
                        response.push_str(&cleaned_transcript);
                        response.push_str(&format!("\n\n🔍 Originale (Whisper):\n{}", raw_transcript));
                    } else {
                        response.push_str(&format!("\n📊 Trascrizione:\n{}", cleaned_transcript));
                    }

                    bot.send_message(msg.chat.id, response).await?;
                    log::info!("Notes generated and saved for user {}", msg.chat.id);
                }
                Err(e) => {
                    log::error!("Note generation failed: {}", e);

                    // Delete status message
                    let _ = bot.delete_message(msg.chat.id, ack_msg.id).await;

                    let error_msg = format!(
                        "✅ Trascrizione completata, ma errore nella generazione note.\n\n\
                        📝 Trascrizione:\n{}\n\n\
                        ❌ Errore generazione note: {}\n\n\
                        💡 Verifica che Ollama sia in esecuzione: ollama list",
                        cleaned_transcript, e
                    );
                    bot.send_message(msg.chat.id, error_msg).await?;
                }
            }
        }
        Err(e) => {
            log::error!("Transcription failed: {}", e);

            // Delete acknowledgment message
            let _ = bot.delete_message(msg.chat.id, ack_msg.id).await;

            let error_msg = format!(
                "❌ Errore nella trascrizione.\n\n\
                Dettagli: {}\n\n\
                💡 Suggerimenti:\n\
                - Verifica che il modello Whisper sia scaricato in: {}\n\
                - Controlla i log per maggiori dettagli\n\
                - Usa /status per verificare la configurazione",
                e,
                config.transcription.model_path
            );
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Handler for text messages (fallback)
pub async fn text_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let text = "📝 Ho ricevuto il tuo messaggio di testo.\n\n\
        Per ora, sono specializzato solo in messaggi vocali! 🎤\n\
        Inviami un messaggio vocale e lo trasformerò in note strutturate.\n\n\
        Usa /help per maggiori informazioni.";

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}
