mod commands;
mod config;
mod monitor;
mod system;

use std::process::Command;
use std::sync::Arc;

use chrono::{Days, TimeZone, Utc};
use teloxide::prelude::*;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{interval, sleep, Duration};

use crate::commands::{answer, MyCommands};
use crate::config::{load_config, Config};
use crate::monitor::{
    check_alerts, take_daily_summary_report, AlertState, DailySummaryReport, RealMetricsProvider,
};

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

fn duration_until_next_daily_summary(hour_utc: u8, minute_utc: u8) -> Duration {
    let now = Utc::now();

    let today = now.date_naive();
    let Some(scheduled_today_naive) = today.and_hms_opt(hour_utc as u32, minute_utc as u32, 0) else {
        return Duration::from_secs(60);
    };

    let mut scheduled = Utc.from_utc_datetime(&scheduled_today_naive);
    if scheduled <= now {
        let tomorrow = today.checked_add_days(Days::new(1)).unwrap_or(today);
        let Some(scheduled_tomorrow_naive) =
            tomorrow.and_hms_opt(hour_utc as u32, minute_utc as u32, 0)
        else {
            return Duration::from_secs(60);
        };
        scheduled = Utc.from_utc_datetime(&scheduled_tomorrow_naive);
    }

    (scheduled - now)
        .to_std()
        .unwrap_or_else(|_| Duration::from_secs(60))
}

fn format_daily_summary_message(report: Option<DailySummaryReport>) -> String {
    match report {
        Some(report) => format!(
            "📅 Daily Summary\n\nSamples: {}\nAlerts triggered: {}\n\nCPU avg/min/max: {:.1}% / {:.1}% / {:.1}%\nRAM avg/min/max: {:.1}% / {:.1}% / {:.1}%\nDisk avg/min/max: {:.1}% / {:.1}% / {:.1}%\n\nGenerated at (UTC): {}",
            report.sample_count,
            report.alert_count,
            report.cpu_avg,
            report.cpu_min,
            report.cpu_max,
            report.ram_avg,
            report.ram_min,
            report.ram_max,
            report.disk_avg,
            report.disk_min,
            report.disk_max,
            report.generated_at.to_rfc3339(),
        ),
        None => format!(
            "📅 Daily Summary\n\nNo monitoring samples were collected since the last summary window.\nGenerated at (UTC): {}",
            Utc::now().to_rfc3339()
        ),
    }
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

    if config.daily_summary.enabled {
        let summary_bot = bot.clone();
        let summary_config = config.clone();
        let summary_state = alert_state.clone();

        tokio::spawn(async move {
            loop {
                let wait = duration_until_next_daily_summary(
                    summary_config.daily_summary.hour_utc,
                    summary_config.daily_summary.minute_utc,
                );
                sleep(wait).await;

                let report = take_daily_summary_report(&summary_state).await;
                let message = format_daily_summary_message(report);
                let owner_chat_id = match summary_config.owner_chat_id() {
                    Ok(chat_id) => chat_id,
                    Err(error) => {
                        log::error!("daily summary skipped: invalid owner chat id: {}", error);
                        continue;
                    }
                };

                if let Err(error) = summary_bot.send_message(owner_chat_id, message).await {
                    log::error!("failed to send daily summary: {}", error);
                }
            }
        });
    }

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