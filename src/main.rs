mod config;
mod handlers;
mod note_generator;
mod ollama;
mod tools;
mod transcription;

use anyhow::Result;
use config::Config;
use handlers::{audio_handler, help_handler, start_handler, status_handler, text_handler};
use teloxide::prelude::*;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    pretty_env_logger::init();
    log::info!("Starting Dot Transcriber Bot...");

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Load configuration
    let config = Config::from_file("config.toml")?;
    log::info!("Configuration loaded successfully");

    // Ensure output directories exist
    config.ensure_directories()?;
    log::info!("Output directories verified");

    // Create bot instance
    let bot = Bot::new(&config.telegram.bot_token);
    log::info!("Bot instance created");

    // Get bot info
    let me = bot.get_me().await?;
    log::info!("Bot started as @{}", me.username());

    // Print startup info
    println!("🤖 Dot Bot is running!");
    println!("   Username: @{}", me.username());
    println!("   Press Ctrl+C to stop");

    // Clone config for use in closures
    let config_voice = config.clone();
    let config_audio = config.clone();

    // Create dispatcher with command and message handlers
    let handler = dptree::entry()
        // Handle commands
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        // Handle voice messages
        .branch(
            Update::filter_message()
                .filter(|msg: Message| msg.voice().is_some())
                .endpoint(move |bot, msg| audio_handler(bot, msg, config_voice.clone())),
        )
        // Handle audio files
        .branch(
            Update::filter_message()
                .filter(|msg: Message| msg.audio().is_some())
                .endpoint(move |bot, msg| audio_handler(bot, msg, config_audio.clone())),
        )
        // Handle all other text messages
        .branch(Update::filter_message().endpoint(text_handler));

    // Start the dispatcher
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("Bot stopped");
    Ok(())
}

/// Command enumeration
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Comandi disponibili:")]
enum Command {
    #[command(description = "Avvia il bot")]
    Start,
    #[command(description = "Mostra l'aiuto")]
    Help,
    #[command(description = "Mostra lo stato del bot")]
    Status,
}

/// Command handler that routes to specific command functions
async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    me: Me,
) -> ResponseResult<()> {
    match cmd {
        Command::Start => start_handler(bot, msg, me).await,
        Command::Help => help_handler(bot, msg).await,
        Command::Status => {
            // Load config for status display
            let config = Config::from_file("config.toml")
                .expect("Failed to load config");
            status_handler(bot, msg, config).await
        }
    }
}
