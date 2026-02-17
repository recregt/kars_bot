use std::time::Instant;

use teloxide::{prelude::*, types::InputFile, types::ParseMode};

use crate::app_context::AppContext;

use super::cooldown::graph_cooldown_remaining_secs;
use super::parser::parse_graph_request;
use super::render::{render_graph_png, GRAPH_WIDTH_PX};
use super::stats::{assess_anomaly_labels, compute_metric_summary, downsample_points};
use super::types::GraphRequest;
use super::super::super::helpers::{acquire_command_slot, as_html_block};

const GRAPH_USAGE_TEXT: &str = "Usage: /graph cpu|ram|disk [<Nm|Nh>]";

pub(crate) async fn handle_graph(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
    query: &str,
) -> ResponseResult<()> {
    let command_started_at = Instant::now();
    let graph_runtime = app_context.graph_runtime.read().await.clone();
    let runtime_config = app_context.runtime_config.read().await.clone();

    let Some(_permit) = acquire_command_slot(&app_context.command_slots, msg, bot).await? else {
        return Ok(());
    };

    if !graph_runtime.enabled {
        bot.send_message(
            msg.chat.id,
            as_html_block("Graph Disabled", "Graph feature is disabled in config."),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some(request) = parse_graph_request(
        query,
        graph_runtime.default_window_minutes as i64,
        graph_runtime.max_window_hours as i64,
    ) else {
        bot.send_message(msg.chat.id, as_html_block("Graph Usage", GRAPH_USAGE_TEXT))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    };

    if let Some(remaining_secs) = graph_cooldown_remaining_secs(app_context).await {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                "Graph Cooldown",
                &format!("Please wait {}s before using /graph again.", remaining_secs),
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let samples = {
        let history = app_context.metric_history.lock().await;
        history.latest_window(request.window.minutes())
    };

    if samples.len() < 2 {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                &format!("{} Graph", request.metric.title()),
                "not enough samples yet",
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let summary = match compute_metric_summary(request.metric, &samples) {
        Some(summary) => summary,
        None => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    &format!("{} Graph", request.metric.title()),
                    "not enough samples yet",
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    let threshold = request.metric.threshold(&runtime_config.alerts);
    let anomaly_labels = assess_anomaly_labels(request.metric, &samples, threshold)
        .map(|assessment| assessment.labels().join(" | "))
        .unwrap_or_default();
    let max_points = usize::from(graph_runtime.max_points).max(2);
    let width_limit = usize::try_from(GRAPH_WIDTH_PX).unwrap_or(max_points);
    let points_limit = max_points.min(width_limit);
    let points = downsample_points(&samples, request.metric, points_limit);
    let point_count = points.len();
    let GraphRequest { metric, window } = request;

    let render_slot = match app_context.graph_render_slots.clone().acquire_owned().await {
        Ok(permit) => permit,
        Err(error) => {
            log::error!("failed to acquire graph render slot: {}", error);
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Graph Render",
                    "Could not acquire render slot. Please try again.",
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    let render_result = tokio::task::spawn_blocking(move || {
        let _render_slot = render_slot;
        render_graph_png(points, metric, threshold)
    })
    .await;

    match render_result {
        Ok(Ok(png_bytes)) => {
            bot.send_photo(
                msg.chat.id,
                InputFile::memory(png_bytes)
                    .file_name(format!("{}-{}.png", metric.file_name(), window.suffix())),
            )
            .caption(format!(
                "{} ({}) | min: {:.1}% | max: {:.1}% | avg: {:.1}%{}",
                metric.caption(),
                window.suffix(),
                summary.min,
                summary.max,
                summary.avg,
                if anomaly_labels.is_empty() {
                    "".to_string()
                } else {
                    format!(" | {}", anomaly_labels)
                }
            ))
            .await?;

            log::info!(
                "graph_command_completed metric={} window_minutes={} source_samples={} rendered_points={} elapsed_ms={}",
                metric.title(),
                window.minutes(),
                samples.len(),
                point_count,
                command_started_at.elapsed().as_millis()
            );
        }
        Ok(Err(error_message)) => {
            log::error!("graph render failed: {}", error_message);
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    &format!("{} Graph", metric.title()),
                    "Could not render graph right now. Please try again.",
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(join_error) => {
            log::error!("graph render task failed: {}", join_error);
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    &format!("{} Graph", metric.title()),
                    "Could not render graph right now. Please try again.",
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }

    Ok(())
}
