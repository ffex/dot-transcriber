use teloxide::{prelude::*, types::Me};
use crate::config::Config;

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
pub async fn audio_handler(bot: Bot, msg: Message, _config: Config) -> ResponseResult<()> {
    log::info!("Received audio message from user {}", msg.chat.id);

    // Send acknowledgment
    bot.send_message(msg.chat.id, "🎤 Messaggio vocale ricevuto! Sto elaborando...").await?;

    // TODO: Phase 2 - Implement actual transcription
    // For now, just acknowledge receipt

    let placeholder_response = "✅ Messaggio ricevuto!\n\n\
        🚧 Funzionalità di trascrizione in arrivo (Fase 2)...\n\n\
        Per ora sto solo ricevendo i tuoi messaggi vocali. \
        Presto sarò in grado di trascriverli e creare note automatiche!";

    bot.send_message(msg.chat.id, placeholder_response).await?;

    log::info!("Audio message acknowledged for user {}", msg.chat.id);
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
