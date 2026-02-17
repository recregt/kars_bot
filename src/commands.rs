use std::sync::Arc;

use chrono::Utc;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::system::{run_cmd, CommandError, CommandOutput};

const TELEGRAM_TEXT_HARD_LIMIT: usize = 4096;
const TELEGRAM_TEXT_SAFE_LIMIT: usize = 3900;
const TRUNCATE_NOTICE: &str = "\n\n⚠️ (Output was truncated...)";

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum MyCommands {
    #[command(description = "Show help menu.")]
    Help,
    #[command(description = "Check RAM and Disk usage.")]
    Status,
    #[command(description = "List open ports.")]
    Ports,
    #[command(description = "List running services.")]
    Services,
    #[command(description = "Show CPU usage.")]
    Cpu,
    #[command(description = "Show network statistics.")]
    Network,
    #[command(description = "Show system uptime.")]
    Uptime,
    #[command(description = "Show temperature sensors.")]
    Temp,
    #[command(description = "Show bot health and monitor liveness.")]
    Health,
}

fn truncate_to_char_boundary(input: &str, max_bytes: usize) -> &str {
    if input.len() <= max_bytes {
        return input;
    }

    let mut end = max_bytes;
    while !input.is_char_boundary(end) {
        end -= 1;
    }

    &input[..end]
}

fn sanitize_and_truncate(input: &str, max_escaped_len: usize) -> String {
    let escaped_full = html_escape::encode_text(input);
    if escaped_full.len() <= max_escaped_len {
        return escaped_full.into_owned();
    }

    let mut low = 0usize;
    let mut high = input.len();
    let mut best = "";

    while low <= high {
        let mid = (low + high) / 2;
        let candidate = truncate_to_char_boundary(input, mid);
        let escaped = html_escape::encode_text(candidate);

        if escaped.len() <= max_escaped_len {
            best = candidate;
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }

    html_escape::encode_text(best).into_owned()
}

fn is_authorized(msg: &Message, config: &Config) -> bool {
    let Some(from) = msg.from() else {
        return false;
    };

    let user_id = from.id.0;
    if !config.allowed_user_ids.contains(&user_id) {
        return false;
    }

    let chat_id = msg.chat.id.0;
    let is_dm = chat_id == user_id as i64;

    match &config.allowed_chat_ids {
        Some(allowed_chats) => is_dm || allowed_chats.contains(&chat_id),
        None => is_dm,
    }
}

fn command_body(output: &CommandOutput) -> String {
    let mut content = String::new();
    let stdout = output.stdout.trim();
    let stderr = output.stderr.trim();

    if !stdout.is_empty() {
        content.push_str(stdout);
    }

    if !stderr.is_empty() {
        if !content.is_empty() {
            content.push_str("\n\n--- stderr ---\n");
        }
        content.push_str(stderr);
    }

    if content.is_empty() {
        content.push_str("No output.");
    }

    if output.status != 0 {
        content.push_str(&format!("\n\n(exit status: {})", output.status));
    }

    content
}

fn as_html_block(title: &str, body: &str) -> String {
    let escaped_title = html_escape::encode_text(title);
    let body_budget = TELEGRAM_TEXT_SAFE_LIMIT.saturating_sub(TRUNCATE_NOTICE.len());
    let mut escaped_body = sanitize_and_truncate(body, body_budget);
    let was_truncated = html_escape::encode_text(body).len() > escaped_body.len();

    if was_truncated {
        escaped_body.push_str(TRUNCATE_NOTICE);
    }

    let message = format!("<b>{}</b>\n<pre>{}</pre>", escaped_title, escaped_body);
    if message.len() > TELEGRAM_TEXT_HARD_LIMIT {
        log::warn!("formatted Telegram message is close to hard limit");
    }
    message
}

fn command_error_html(error: &CommandError) -> String {
    format!(
        "<b>Command execution failed</b>\n<pre>{}</pre>",
        sanitize_and_truncate(&error.to_string(), TELEGRAM_TEXT_SAFE_LIMIT)
    )
}

pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: MyCommands,
    config: &Config,
    last_monitor_tick: &Arc<Mutex<chrono::DateTime<Utc>>>,
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
            let ram = run_cmd("free", &["-h"], config.command_timeout_secs).await;
            let disk = run_cmd("df", &["-h", "/"], config.command_timeout_secs).await;

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

            bot.send_message(
                msg.chat.id,
                message,
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        MyCommands::Ports => {
            let message = match run_cmd("ss", &["-tuln"], config.command_timeout_secs).await {
                Ok(output) => as_html_block("Open Ports", &command_body(&output)),
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Services => {
            let services = run_cmd(
                "systemctl",
                &[
                    "list-units",
                    "--type=service",
                    "--state=running",
                    "--no-pager",
                ],
                config.command_timeout_secs,
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
                    let body = if short.is_empty() { "No service output." } else { &short };
                    as_html_block("Active Services", body)
                }
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Cpu => {
            let message = match run_cmd("top", &["-bn1"], config.command_timeout_secs).await {
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
            let message = match run_cmd("ip", &["-s", "link"], config.command_timeout_secs).await {
                Ok(output) => as_html_block("Network Statistics", &command_body(&output)),
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Uptime => {
            let message = match run_cmd("uptime", &[], config.command_timeout_secs).await {
                Ok(output) => as_html_block("System Uptime", &command_body(&output)),
                Err(error) => command_error_html(&error),
            };

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        MyCommands::Temp => {
            let message = match run_cmd("sensors", &[], config.command_timeout_secs).await {
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
            let lag_secs = now.signed_duration_since(last_tick).num_seconds().max(0);
            let threshold_secs = (config.monitor_interval * 2) as i64;

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

            let body = format!(
                "{}\n\nMonitor interval: {}s\nCurrent time: {}\nLast tick: {}",
                status_line,
                config.monitor_interval,
                now.to_rfc3339(),
                last_tick.to_rfc3339()
            );

            bot.send_message(msg.chat.id, as_html_block("Bot Health", &body))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}