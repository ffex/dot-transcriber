use teloxide::{prelude::*, types::Me};
use crate::config::Config;
use crate::transcription;

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
        Ok(transcript) => {
            // Delete acknowledgment message
            let _ = bot.delete_message(msg.chat.id, ack_msg.id).await;

            // Send transcription result
            let response = format!(
                "✅ Trascrizione completata!\n\n📝 Testo:\n{}",
                transcript
            );
            bot.send_message(msg.chat.id, response).await?;

            log::info!("Transcription successful for user {}: {} chars",
                       msg.chat.id, transcript.len());
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
