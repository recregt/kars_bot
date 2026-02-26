use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::architecture::use_cases::alert_snapshot_use_case;

use super::super::helpers::as_html_block;
use super::menu::main_menu_keyboard;

pub(crate) async fn handle_status_overview(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
) -> ResponseResult<()> {
    let graph_runtime = app_context.graph_runtime.read().await.clone();
    let runtime_config = app_context.runtime_config.read().await.clone();
    let alert_state = alert_snapshot_use_case(&app_context.monitor.alert_state).await;
    let last_monitor_tick = *app_context.monitor.last_monitor_tick.lock().await;
    let now = chrono::Utc::now();

    let mute_state = match alert_state.muted_until {
        Some(until) if now < until => format!("muted until {}", until.to_rfc3339()),
        _ => "not muted".to_string(),
    };

    let last_tick_text =
        last_monitor_tick.map_or_else(|| "not available yet".to_string(), |tick| tick.to_rfc3339());

    let capabilities = app_context.capabilities.as_ref();

    let body = format!(
        "Auth Mode: Owner Only (DM)\nStorage: Hierarchical JSONL + Indexed\nMaintenance: {}\nRetention: {} days\nAnomaly dir: {}\n\nRuntime:\n- Monitor interval: {}s\n- Last monitor tick: {}\n- Mute state: {}\n\nReporting Store:\n- enabled: {}\n- path: {}\n- retention: {} days\n\nSecurity:\n- redact_sensitive_output: {}\n\nSimulation:\n- enabled: {}\n- profile: {}\n\nGraph Runtime:\n- enabled: {}\n- default window: {}m\n- max window: {}h\n- max points: {}\n\nCapabilities:\n- is_systemd: {}\n- has_sensors: {}\n- has_free: {}\n- has_top: {}\n- has_ip: {}\n- has_ss: {}\n- has_uptime: {}\n\nButton-first UX:\n- Use menu buttons below to run actions directly.\n- Slash commands are optional for advanced queries.\n\nAdvanced examples:\n/recent\n/recent 5\n/recent 6h\n/recent cpu>85",
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

    bot.send_message(msg.chat.id, as_html_block("Bot Status", &body))
        .reply_markup(main_menu_keyboard())
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
