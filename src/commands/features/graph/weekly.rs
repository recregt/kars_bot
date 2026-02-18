use crate::app_context::AppContext;

use super::GeneratedGraphReport;
use super::executor::{acquire_render_slot, run_render_task};
use super::render::GRAPH_WIDTH_PX;
use super::stats::{assess_anomaly_labels, compute_metric_summary, downsample_points};
use super::types::GraphMetric;

const WEEKLY_WINDOW_MINUTES: i64 = 7 * 24 * 60;
const RENDER_SLOT_WAIT_TIMEOUT_SECS: u64 = 3;
const RENDER_EXECUTION_TIMEOUT_SECS: u64 = 8;

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

    let persisted_rollup = app_context
        .reporting_store
        .as_ref()
        .and_then(|store| store.rolling_summary_days(7));

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

    let render_slot = acquire_render_slot(
        app_context.graph_render_slots.clone(),
        RENDER_SLOT_WAIT_TIMEOUT_SECS,
    )
    .await
    .map_err(|error| {
        format!(
            "weekly render slot failed code={} error={}",
            error.code(),
            error
        )
    })?;

    let png_bytes = run_render_task(
        points,
        GraphMetric::Cpu,
        threshold,
        render_slot,
        RENDER_EXECUTION_TIMEOUT_SECS,
    )
    .await
    .map_err(|error| format!("weekly render failed code={} error={}", error.code(), error))?;

    let (min_cpu, max_cpu, avg_cpu, samples_count, rollup_suffix) = if let Some(rollup) =
        persisted_rollup
    {
        let suffix = format!(
            "\nRAM avg/min/max: {:.1}% / {:.1}% / {:.1}% | Disk avg/min/max: {:.1}% / {:.1}% / {:.1}%",
            rollup.ram_avg,
            rollup.ram_min,
            rollup.ram_max,
            rollup.disk_avg,
            rollup.disk_min,
            rollup.disk_max
        );
        (
            rollup.cpu_min,
            rollup.cpu_max,
            rollup.cpu_avg,
            rollup.sample_count,
            suffix,
        )
    } else {
        (
            summary.min,
            summary.max,
            summary.avg,
            samples.len() as u64,
            String::new(),
        )
    };

    Ok(GeneratedGraphReport {
        png_bytes,
        file_name: "cpu-weekly-7d.png".to_string(),
        caption: format!(
            "📈 Weekly CPU (7d) | samples: {} | min: {:.1}% | max: {:.1}% | avg: {:.1}%{}{}",
            samples_count,
            min_cpu,
            max_cpu,
            avg_cpu,
            if anomaly_labels.is_empty() {
                "".to_string()
            } else {
                format!(" | {}", anomaly_labels)
            },
            rollup_suffix,
        ),
    })
}
