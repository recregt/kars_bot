use std::sync::Arc;

use chrono::Utc;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};
use tokio::sync::{Mutex, Semaphore};

use crate::config::Config;
use crate::monitor::{alert_snapshot, mute_alerts_for, unmute_alerts, AlertState};
use crate::system::run_cmd;

use super::command_def::MyCommands;
use super::helpers::{
    acquire_command_slot, as_html_block, command_body, command_error_html, is_authorized,
    parse_mute_duration, timeout_for,
};

pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: MyCommands,
    config: &Config,
    last_monitor_tick: &Arc<Mutex<Option<chrono::DateTime<Utc>>>>,
    alert_state: &Arc<Mutex<AlertState>>,
    command_slots: &Arc<Semaphore>,
) -> ResponseResult<()> {
    if !is_authorized(&msg, config) {
        let user_id = msg
            .from()
            .map(|user| user.id.0.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        log::warn!(
            "SECURITY: Unauthorized access attempt. user_id={}, chat_id={}, command_text={:?}",
            user_id,
            msg.chat.id.0,
            msg.text()
        );
        return Ok(());
    }

    match cmd {
        MyCommands::Help => {
            bot.send_message(
                msg.chat.id,
                as_html_block("Available commands", &MyCommands::descriptions().to_string()),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        MyCommands::Status => {
            let Some(_permit) = acquire_command_slot(command_slots, &msg, &bot).await? else {
                return Ok(());
            };
            let timeout = timeout_for(&cmd, config);
            let ram = run_cmd("free", &["-h"], timeout).await;
            let disk = run_cmd("df", &["-h", "/"], timeout).await;

            let message = match (ram, disk) {
                (Ok(ram_out), Ok(disk_out)) => {
                    let body = format!(
                        "RAM:\n{}\n\nDisk:\n{}",
                        command_body(&ram_out),
                        command_body(&disk_out)
                    );
                    as_html_block("System Status", &body)
                }
                (Err(error), _) | (_, Err(error)) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Ports => {
            let Some(_permit) = acquire_command_slot(command_slots, &msg, &bot).await? else {
                return Ok(());
            };
            let message = match run_cmd("ss", &["-tuln"], timeout_for(&cmd, config)).await {
                Ok(output) => as_html_block("Open Ports", &command_body(&output)),
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Services => {
            let Some(_permit) = acquire_command_slot(command_slots, &msg, &bot).await? else {
                return Ok(());
            };
            let services = run_cmd(
                "systemctl",
                &[
                    "list-units",
                    "--type=service",
                    "--state=running",
                    "--no-pager",
                ],
                timeout_for(&cmd, config),
            )
            .await;

            let message = match services {
                Ok(output) => {
                    let short = output
                        .stdout
                        .lines()
                        .filter(|line| line.contains(".service"))
                        .take(10)
                        .collect::<Vec<_>>()
                        .join("\n");
                    let body = if short.is_empty() {
                        "No service output."
                    } else {
                        &short
                    };
                    as_html_block("Active Services", body)
                }
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Cpu => {
            let Some(_permit) = acquire_command_slot(command_slots, &msg, &bot).await? else {
                return Ok(());
            };
            let message = match run_cmd("top", &["-bn1"], timeout_for(&cmd, config)).await {
                Ok(output) => {
                    let short = output
                        .stdout
                        .lines()
                        .filter(|line| line.contains("Cpu(s)"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let body = if short.is_empty() { "No CPU output." } else { &short };
                    as_html_block("CPU Usage", body)
                }
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Network => {
            let Some(_permit) = acquire_command_slot(command_slots, &msg, &bot).await? else {
                return Ok(());
            };
            let message =
                match run_cmd("ip", &["-s", "link"], timeout_for(&cmd, config)).await {
                    Ok(output) => as_html_block("Network Statistics", &command_body(&output)),
                    Err(error) => command_error_html(&error),
                };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Uptime => {
            let Some(_permit) = acquire_command_slot(command_slots, &msg, &bot).await? else {
                return Ok(());
            };
            let message = match run_cmd("uptime", &[], timeout_for(&cmd, config)).await {
                Ok(output) => as_html_block("System Uptime", &command_body(&output)),
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Temp => {
            let Some(_permit) = acquire_command_slot(command_slots, &msg, &bot).await? else {
                return Ok(());
            };
            let message = match run_cmd("sensors", &[], timeout_for(&cmd, config)).await {
                Ok(output) => as_html_block("Temperature Sensors", &command_body(&output)),
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Health => {
            let last_tick = *last_monitor_tick.lock().await;
            let now = Utc::now();
            let threshold_secs = (config.monitor_interval * 2) as i64;

            let body = match last_tick {
                Some(tick) => {
                    let lag_secs = now.signed_duration_since(tick).num_seconds().max(0);
                    let status_line = if lag_secs > threshold_secs {
                        format!(
                            "⚠️ CRITICAL: Monitor loop is delayed. Last tick: {}s ago (threshold: {}s)",
                            lag_secs, threshold_secs
                        )
                    } else {
                        format!(
                            "✅ Healthy. Last monitor tick: {}s ago (threshold: {}s)",
                            lag_secs, threshold_secs
                        )
                    };

                    format!(
                        "{}\n\nMonitor interval: {}s\nCurrent time: {}\nLast tick: {}",
                        status_line,
                        config.monitor_interval,
                        now.to_rfc3339(),
                        tick.to_rfc3339()
                    )
                }
                None => format!(
                    "⏳ Warming up...\n\nMonitor loop has not produced the first tick yet.\nMonitor interval: {}s\nCurrent time: {}",
                    config.monitor_interval,
                    now.to_rfc3339()
                ),
            };

            bot.send_message(msg.chat.id, as_html_block("Bot Health", &body))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Alerts => {
            let snapshot = alert_snapshot(alert_state).await;
            let now = Utc::now();
            let mute_line = match snapshot.muted_until {
                Some(until) if now <= until => {
                    let remaining = until.signed_duration_since(now).num_seconds().max(0);
                    format!("muted ({}s remaining until {})", remaining, until.to_rfc3339())
                }
                _ => "not muted".to_string(),
            };
            let summary_line = snapshot
                .last_daily_summary_at
                .map(|time| time.to_rfc3339())
                .unwrap_or_else(|| "not generated yet".to_string());
            let body = format!(
                "Thresholds:\n- CPU: {:.1}%\n- RAM: {:.1}%\n- Disk: {:.1}%\n\nControl:\n- Cooldown: {}s\n- Hysteresis: {:.1}%\n- Mute: {}\n- Last daily summary (UTC): {}\n\nCurrent State:\n- CPU alerting: {}\n- RAM alerting: {}\n- Disk alerting: {}",
                config.alerts.cpu,
                config.alerts.ram,
                config.alerts.disk,
                config.alerts.cooldown_secs,
                config.alerts.hysteresis,
                mute_line,
                summary_line,
                if snapshot.cpu_alerting { "yes" } else { "no" },
                if snapshot.ram_alerting { "yes" } else { "no" },
                if snapshot.disk_alerting { "yes" } else { "no" }
            );

            bot.send_message(msg.chat.id, as_html_block("Alert Configuration", &body))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Mute(duration_str) => {
            let Some(duration) = parse_mute_duration(&duration_str) else {
                let message = as_html_block(
                    "Mute failed",
                    "Invalid duration. Use format like: 30s, 15m, 2h, 1d",
                );
                bot.send_message(msg.chat.id, message)
                    .parse_mode(ParseMode::Html)
                    .await?;
                return Ok(());
            };

            let muted_until = mute_alerts_for(alert_state, duration).await;
            let body = format!("Alerts are muted until {}", muted_until.to_rfc3339());
            bot.send_message(msg.chat.id, as_html_block("Alerts muted", &body))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Unmute => {
            unmute_alerts(alert_state).await;
            bot.send_message(
                msg.chat.id,
                as_html_block("Alerts unmuted", "Alerts are active again."),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }

    Ok(())
}