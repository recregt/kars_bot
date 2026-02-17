mod anomaly_journal;
mod app_context;
mod commands;
mod config;
mod jobs;
mod monitor;
mod system;

use std::process::Command;

use teloxide::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::app_context::AppContext;
use crate::commands::{answer, MyCommands};
use crate::config::{load_config, Config};
use crate::jobs::start_background_jobs;

fn init_json_logging() {
    if let Err(error) = tracing_log::LogTracer::init() {
        eprintln!(
            "logging bridge initialization failed (continuing with existing logger): {}",
            error
        );
    }

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .finish();

    if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("global logger initialization failed: {}", error);
    }
}

fn check_external_command(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_preflight_checks() -> bool {
    let required_commands = ["systemctl", "sensors"];
    let mut all_ok = true;

    for command in required_commands {
        if !check_external_command(command) {
            log::error!(
                "Preflight failed: required external command '{}' was not found in PATH",
                command
            );
            all_ok = false;
        }
    }

    all_ok
}

// Main
#[tokio::main]
async fn main() {
    init_json_logging();

    let config: Config = match load_config("config.toml") {
        Ok(config) => config,
        Err(error) => {
            log::error!("Configuration error: {}", error);
            return;
        }
    };

    if let Err(error) = config.validate() {
        log::error!("Configuration validation failed: {}", error);
        return;
    }

    if !run_preflight_checks() {
        log::error!("Startup aborted due to failed preflight checks");
        return;
    }

    log::info!("Kars Server Bot is starting...");
    let bot = Bot::new(&config.bot_token);

    let app_context = AppContext::new(config.clone(), 2);

    start_background_jobs(bot.clone(), app_context.clone());

    MyCommands::repl(bot, move |bot, msg, cmd| {
        let app_context = app_context.clone();
        async move {
            answer(bot, msg, cmd, &app_context).await
        }
    })
    .await;
}