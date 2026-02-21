use chrono::Utc;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
    utils::command::BotCommands,
};

use crate::app_context::AppContext;

use super::super::{command_def::MyCommands, helpers::as_html_block};

pub(crate) async fn handle_help(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    let quick_actions = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📊 Status", "cmd:status"),
            InlineKeyboardButton::callback("💓 Health", "cmd:health"),
        ],
        vec![
            InlineKeyboardButton::callback("📈 Graph CPU", "cmd:graph:cpu 1h"),
            InlineKeyboardButton::callback("🧾 Recent 6h", "cmd:recent:6h"),
        ],
        vec![
            InlineKeyboardButton::callback("🚨 Alerts", "cmd:alerts"),
            InlineKeyboardButton::callback("🔇 Mute 1h", "cmd:mute:1h"),
            InlineKeyboardButton::callback("🔔 Unmute", "cmd:unmute"),
        ],
    ]);

    bot.send_message(
        msg.chat.id,
        as_html_block(
            "Available commands",
            &MyCommands::descriptions().to_string(),
        ),
    )
    .reply_markup(quick_actions)
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

pub(crate) async fn handle_health(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
) -> ResponseResult<()> {
    let runtime_config = app_context.runtime_config.read().await.clone();
    let last_tick = *app_context.last_monitor_tick.lock().await;
    let now = Utc::now();
    let threshold_secs = (runtime_config.monitor_interval * 2) as i64;

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
                runtime_config.monitor_interval,
                now.to_rfc3339(),
                tick.to_rfc3339()
            )
        }
        None => format!(
            "⏳ Warming up...\n\nMonitor loop has not produced the first tick yet.\nMonitor interval: {}s\nCurrent time: {}",
            runtime_config.monitor_interval,
            now.to_rfc3339()
        ),
    };

    bot.send_message(msg.chat.id, as_html_block("Bot Health", &body))
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
