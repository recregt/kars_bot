use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::app_context::AppContext;
use crate::monitor::alert_snapshot;

use super::super::helpers::as_html_block;

pub(crate) async fn handle_status_overview(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
) -> ResponseResult<()> {
    let graph_runtime = app_context.graph_runtime.read().await.clone();
    let runtime_config = app_context.runtime_config.read().await.clone();
    let alert_state = alert_snapshot(&app_context.alert_state).await;
    let last_monitor_tick = *app_context.last_monitor_tick.lock().await;
    let now = chrono::Utc::now();

    let mute_state = match alert_state.muted_until {
        Some(until) if now < until => format!("muted until {}", until.to_rfc3339()),
        _ => "not muted".to_string(),
    };

    let last_tick_text = last_monitor_tick
        .map(|tick| tick.to_rfc3339())
        .unwrap_or_else(|| "not available yet".to_string());

    let capabilities = app_context.capabilities.as_ref();

    let body = format!(
        "Auth Mode: Owner Only (DM)\nStorage: Hierarchical JSONL + Indexed\nMaintenance: {}\nRetention: {} days\nAnomaly dir: {}\n\nRuntime:\n- Monitor interval: {}s\n- Last monitor tick: {}\n- Mute state: {}\n\nReporting Store:\n- enabled: {}\n- path: {}\n- retention: {} days\n\nSecurity:\n- redact_sensitive_output: {}\n\nSimulation:\n- enabled: {}\n- profile: {}\n\nGraph Runtime:\n- enabled: {}\n- default window: {}m\n- max window: {}h\n- max points: {}\n\nCapabilities:\n- is_systemd: {}\n- has_sensors: {}\n- has_free: {}\n- has_top: {}\n- has_ip: {}\n- has_ss: {}\n- has_uptime: {}\n\nQuick Action Safety:\n- Buttons prefill commands only; execution happens after you manually send.\n\nSmart Query Examples:\n/recent\n/recent 5\n/recent 6h\n/recent cpu>85",
        if app_context.config.anomaly_db.enabled {
            "Active (Hourly)"
        } else {
            "Disabled"
        },
        app_context.config.anomaly_db.retention_days,
        app_context.config.anomaly_db.dir,
        runtime_config.monitor_interval,
        last_tick_text,
        mute_state,
        app_context.config.reporting_store.enabled,
        app_context.config.reporting_store.path,
        app_context.config.reporting_store.retention_days,
        app_context.config.security.redact_sensitive_output,
        app_context.config.simulation.enabled,
        app_context.config.simulation.profile,
        graph_runtime.enabled,
        graph_runtime.default_window_minutes,
        graph_runtime.max_window_hours,
        graph_runtime.max_points,
        capabilities.is_systemd,
        capabilities.has_sensors,
        capabilities.has_free,
        capabilities.has_top,
        capabilities.has_ip,
        capabilities.has_ss,
        capabilities.has_uptime,
    );

    let quick_actions = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📈 Graph CPU", "cmd:graph:cpu 1h"),
            InlineKeyboardButton::callback("🚨 Alerts", "cmd:alerts"),
        ],
        vec![
            InlineKeyboardButton::callback("🔇 Mute 1h", "cmd:mute:1h"),
            InlineKeyboardButton::callback("🔔 Unmute", "cmd:unmute"),
        ],
        vec![InlineKeyboardButton::switch_inline_query_current_chat(
            "🩺 Health",
            "/health",
        )],
    ]);

    bot.send_message(msg.chat.id, as_html_block("Bot Status", &body))
        .reply_markup(quick_actions)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
