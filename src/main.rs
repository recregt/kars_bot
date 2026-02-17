mod commands;
mod config;
mod monitor;
mod system;

use std::process::Command;
use std::sync::Arc;

use chrono::Utc;
use teloxide::prelude::*;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{interval, Duration};

use crate::commands::{answer, MyCommands};
use crate::config::{load_config, Config};
use crate::monitor::{check_alerts, AlertState, RealMetricsProvider};

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
    pretty_env_logger::init();

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

    let alert_state = Arc::new(Mutex::new(AlertState::default()));
    let last_monitor_tick = Arc::new(Mutex::new(None));
    let command_slots = Arc::new(Semaphore::new(2));

    let bot_clone = bot.clone();
    let config_clone = config.clone();
    let state_clone = alert_state.clone();
    let tick_clone = last_monitor_tick.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(config_clone.monitor_interval));
        let mut metrics_provider = RealMetricsProvider::new();

        loop {
            interval.tick().await;
            {
                let mut tick = tick_clone.lock().await;
                *tick = Some(Utc::now());
            }
            check_alerts(&bot_clone, &config_clone, &state_clone, &mut metrics_provider).await;
        }
    });

    MyCommands::repl(bot, move |bot, msg, cmd| {
        let config = config.clone();
        let last_monitor_tick = last_monitor_tick.clone();
        let alert_state = alert_state.clone();
        let command_slots = command_slots.clone();
        async move {
            answer(
                bot,
                msg,
                cmd,
                &config,
                &last_monitor_tick,
                &alert_state,
                &command_slots,
            )
            .await
        }
    }).await;
}