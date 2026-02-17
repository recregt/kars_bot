use crate::app_context::AppContext;

use super::render::{render_graph_png, GRAPH_WIDTH_PX};
use super::stats::{assess_anomaly_labels, compute_metric_summary, downsample_points};
use super::types::GraphMetric;
use super::GeneratedGraphReport;

const WEEKLY_WINDOW_MINUTES: i64 = 7 * 24 * 60;

pub(crate) async fn build_weekly_cpu_report(
    app_context: &AppContext,
) -> Result<GeneratedGraphReport, String> {
    let graph_runtime = app_context.graph_runtime.read().await.clone();
    let runtime_config = app_context.runtime_config.read().await.clone();
    if !graph_runtime.enabled {
        return Err("graph feature is disabled in config".to_string());
    }

    let samples = if let Some(store) = app_context.reporting_store.as_ref() {
        let persisted = store.latest_window(WEEKLY_WINDOW_MINUTES);
        if persisted.len() >= 2 {
            persisted
        } else {
            let history = app_context.metric_history.lock().await;
            history.latest_window(WEEKLY_WINDOW_MINUTES)
        }
    } else {
        let history = app_context.metric_history.lock().await;
        history.latest_window(WEEKLY_WINDOW_MINUTES)
    };

    if samples.len() < 2 {
        return Err("not enough samples yet".to_string());
    }

    let summary = compute_metric_summary(GraphMetric::Cpu, &samples)
        .ok_or_else(|| "not enough samples yet".to_string())?;

    let threshold = runtime_config.alerts.cpu;
    let anomaly_labels = assess_anomaly_labels(GraphMetric::Cpu, &samples, threshold)
        .map(|assessment| assessment.labels().join(" | "))
        .unwrap_or_default();
    let max_points = usize::from(graph_runtime.max_points).max(2);
    let width_limit = usize::try_from(GRAPH_WIDTH_PX).unwrap_or(max_points);
    let points_limit = max_points.min(width_limit);
    let points = downsample_points(&samples, GraphMetric::Cpu, points_limit);

    let render_slot = app_context
        .graph_render_slots
        .clone()
        .acquire_owned()
        .await
        .map_err(|error| format!("could not acquire render slot: {}", error))?;

    let png_bytes = tokio::task::spawn_blocking(move || {
        let _render_slot = render_slot;
        render_graph_png(points, GraphMetric::Cpu, threshold)
    })
    .await
    .map_err(|error| format!("weekly render task failed: {}", error))??;

    Ok(GeneratedGraphReport {
        png_bytes,
        file_name: "cpu-weekly-7d.png".to_string(),
        caption: format!(
            "📈 Weekly CPU (7d) | min: {:.1}% | max: {:.1}% | avg: {:.1}%{}",
            summary.min,
            summary.max,
            summary.avg,
            if anomaly_labels.is_empty() {
                "".to_string()
            } else {
                format!(" | {}", anomaly_labels)
            }
        ),
    })
}
