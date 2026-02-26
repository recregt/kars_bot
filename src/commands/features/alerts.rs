use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::architecture::{
    ports::MuteActionError,
    use_cases::{alert_snapshot_use_case, mute_alerts_use_case, unmute_alerts_use_case},
};

use super::super::helpers::{as_html_card, escape_html_text, parse_mute_duration};
use super::menu::{main_menu_keyboard, upsert_message_with_menu};

pub(crate) async fn handle_alerts(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
) -> ResponseResult<()> {
    let runtime_config = app_context.runtime_config.read().await.clone();
    let snapshot = alert_snapshot_use_case(&app_context.monitor.alert_state).await;
    let now = chrono::Utc::now();
    let mute_line = match snapshot.muted_until {
        Some(until) if now <= until => {
            let remaining = until.signed_duration_since(now).num_seconds().max(0);
            format!(
                "muted ({}s remaining until {})",
                remaining,
                until.to_rfc3339()
            )
        }
        _ => "not muted".to_string(),
    };
    let summary_line = snapshot
        .last_daily_summary_at
        .map_or_else(|| "not generated yet".to_string(), |time| time.to_rfc3339());
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

    let alert_html = as_html_card(
        "Alert Configuration",
        &escape_html_text(&body).replace('\n', "<br/>"),
    );

    bot.send_message(msg.chat.id, alert_html)
        .reply_markup(main_menu_keyboard())
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
        let message = as_html_card(
            "Mute failed",
            "Invalid duration. Use format like: <b>30s</b>, <b>15m</b>, <b>2h</b>, <b>1d</b>.",
        );
        upsert_message_with_menu(bot, msg, message, "monitor").await?;
        return Ok(());
    };

    let muted_until = match mute_alerts_use_case(&app_context.monitor.alert_state, duration).await {
        Ok(until) => until,
        Err(MuteActionError::Cooldown { retry_after_secs }) => {
            let message = as_html_card(
                "Mute cooldown",
                &format!(
                    "Please wait <b>{retry_after_secs}s</b> before changing mute state again."
                ),
            );
            upsert_message_with_menu(bot, msg, message, "monitor").await?;
            return Ok(());
        }
    };
    let message = as_html_card(
        "Alerts muted ✅",
        &format!(
            "Alerts are muted until <b>{}</b>.<br/><br/>You can continue from the Monitor menu below.",
            escape_html_text(&muted_until.to_rfc3339())
        ),
    );
    upsert_message_with_menu(bot, msg, message, "monitor").await?;

    Ok(())
}

pub(crate) async fn handle_unmute(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
) -> ResponseResult<()> {
    if let Err(MuteActionError::Cooldown { retry_after_secs }) =
        unmute_alerts_use_case(&app_context.monitor.alert_state).await
    {
        let message = as_html_card(
            "Unmute cooldown",
            &format!("Please wait <b>{retry_after_secs}s</b> before changing mute state again."),
        );
        upsert_message_with_menu(bot, msg, message, "monitor").await?;
        return Ok(());
    }
    let message = as_html_card(
        "Alerts unmuted ✅",
        "Alerts are active again.<br/><br/>You can continue from the Monitor menu below.",
    );
    upsert_message_with_menu(bot, msg, message, "monitor").await?;

    Ok(())
}
