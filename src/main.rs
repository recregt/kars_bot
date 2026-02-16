mod commands;
mod config;
mod monitor;
mod system;

use teloxide::prelude::*;
use tokio::time::{interval, Duration};

use crate::commands::{answer, MyCommands};
use crate::config::{load_config, Config};
use crate::monitor::check_alerts;

// Main
#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Load config file
    let config: Config = load_config("config.toml");

    log::info!("Kars Server Bot is starting...");
    let bot = Bot::new(&config.bot_token);

    // Spawn background alert monitoring
    let bot_clone = bot.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(config_clone.monitor_interval));
        loop {
            interval.tick().await;
            check_alerts(&bot_clone, &config_clone).await;
        }
    });

    MyCommands::repl(bot, move |bot, msg, cmd| {
        let config = config.clone();
        async move { answer(bot, msg, cmd, &config).await }
    }).await;
}