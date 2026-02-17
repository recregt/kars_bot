use teloxide::{prelude::*, types::InputFile, types::ParseMode};

use crate::app_context::AppContext;

use super::super::helpers::{acquire_command_slot, as_html_block};
use parser::{format_window_suffix, parse_export_request};
use payload::build_export_payload;

mod parser;
mod payload;

const EXPORT_USAGE_TEXT: &str = "Usage: /export cpu|ram|disk [<Nm|Nh>] [csv|json]";

pub(crate) async fn handle_export(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
    query: &str,
) -> ResponseResult<()> {
    let Some(_permit) = acquire_command_slot(&app_context.command_slots, msg, bot).await? else {
        return Ok(());
    };

    let graph_runtime = app_context.graph_runtime.read().await.clone();
    if !graph_runtime.enabled {
        bot.send_message(
            msg.chat.id,
            as_html_block("Export Disabled", "Export feature is disabled in config."),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some(request) = parse_export_request(
        query,
        graph_runtime.default_window_minutes as i64,
        graph_runtime.max_window_hours as i64,
    ) else {
        bot.send_message(msg.chat.id, as_html_block("Export Usage", EXPORT_USAGE_TEXT))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    };

    let samples = {
        let history = app_context.metric_history.lock().await;
        history.latest_window(request.window_minutes)
    };

    if samples.is_empty() {
        bot.send_message(
            msg.chat.id,
            as_html_block("Export", "not enough samples yet"),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let file_name = format!(
        "{}-{}.{},",
        request.metric.as_str(),
        format_window_suffix(request.window_minutes),
        request.format.extension()
    );
    let file_name = file_name.trim_end_matches(',').to_string();

    let body = match build_export_payload(&samples, request.metric, request.format) {
        Ok(body) => body,
        Err(error) => {
            bot.send_message(
                msg.chat.id,
                as_html_block("Export", &format!("Could not build export: {}", error)),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    bot.send_document(msg.chat.id, InputFile::memory(body).file_name(file_name))
        .caption(format!(
            "Exported {} samples for {} ({})",
            samples.len(),
            request.metric.as_str(),
            format_window_suffix(request.window_minutes)
        ))
        .await?;

    Ok(())
}
