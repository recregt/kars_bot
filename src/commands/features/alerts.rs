use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::monitor::{alert_snapshot, mute_alerts_for, unmute_alerts, MuteActionError};

use super::super::helpers::{as_html_block, parse_mute_duration};

pub(crate) async fn handle_alerts(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
) -> ResponseResult<()> {
    let runtime_config = app_context.runtime_config.read().await.clone();
    let snapshot = alert_snapshot(&app_context.alert_state).await;
    let now = chrono::Utc::now();
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
        runtime_config.alerts.cpu,
        runtime_config.alerts.ram,
        runtime_config.alerts.disk,
        runtime_config.alerts.cooldown_secs,
        runtime_config.alerts.hysteresis,
        mute_line,
        summary_line,
        if snapshot.cpu_alerting { "yes" } else { "no" },
        if snapshot.ram_alerting { "yes" } else { "no" },
        if snapshot.disk_alerting { "yes" } else { "no" }
    );

    bot.send_message(msg.chat.id, as_html_block("Alert Configuration", &body))
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_mute(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
    duration_str: &str,
) -> ResponseResult<()> {
    let Some(duration) = parse_mute_duration(duration_str) else {
        let message = as_html_block(
            "Mute failed",
            "Invalid duration. Use format like: 30s, 15m, 2h, 1d",
        );
        bot.send_message(msg.chat.id, message)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    };

    let muted_until = match mute_alerts_for(&app_context.alert_state, duration).await {
        Ok(until) => until,
        Err(MuteActionError::Cooldown { retry_after_secs }) => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Mute cooldown",
                    &format!(
                        "Please wait {}s before changing mute state again.",
                        retry_after_secs
                    ),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };
    let body = format!("Alerts are muted until {}", muted_until.to_rfc3339());
    bot.send_message(msg.chat.id, as_html_block("Alerts muted", &body))
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_unmute(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
) -> ResponseResult<()> {
    if let Err(MuteActionError::Cooldown { retry_after_secs }) =
        unmute_alerts(&app_context.alert_state).await
    {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                "Unmute cooldown",
                &format!(
                    "Please wait {}s before changing mute state again.",
                    retry_after_secs
                ),
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }
    bot.send_message(
        msg.chat.id,
        as_html_block("Alerts unmuted", "Alerts are active again."),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
