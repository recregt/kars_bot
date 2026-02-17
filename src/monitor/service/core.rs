use std::{sync::Arc, time::Instant};

use chrono::Utc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::anomaly_db::record_anomaly_if_needed;
use crate::config::{Config, RuntimeConfig};
use crate::reporting_store::ReportingStore;

use super::super::{
    evaluator::evaluate_alerts_at,
    history::{MetricHistory, MetricSample},
    provider::MetricsProvider,
    state::AlertState,
};

pub async fn check_alerts<P: MetricsProvider>(
    bot: &Bot,
    config: &Config,
    runtime_config: &RuntimeConfig,
    reporting_store: Option<&ReportingStore>,
    state: &Arc<Mutex<AlertState>>,
    metric_history: &Arc<Mutex<MetricHistory>>,
    provider: &mut P,
) {
    let metrics = match provider.collect_metrics().await {
        Ok(metrics) => metrics,
        Err(error) => {
            log::warn!("monitoring provider error: {}", error);
            return;
        }
    };

    tracing::info!(
        target: "monitor",
        module = "monitor",
        cpu = metrics.cpu,
        ram = metrics.ram,
        disk = metrics.disk,
        cpu_threshold = runtime_config.alerts.cpu,
        ram_threshold = runtime_config.alerts.ram,
        disk_threshold = runtime_config.alerts.disk,
        cpu_over = metrics.cpu > runtime_config.alerts.cpu,
        ram_over = metrics.ram > runtime_config.alerts.ram,
        disk_over = metrics.disk > runtime_config.alerts.disk,
        "monitor_metrics"
    );

    let mut effective_config = config.clone();
    effective_config.alerts = runtime_config.alerts.clone();

    record_anomaly_if_needed(&effective_config, metrics.cpu, metrics.ram, metrics.disk);

    let notifications = evaluate_alerts_at(&effective_config, state, metrics, Instant::now()).await;

    {
        let mut state = state.lock().await;
        state.record_metrics(metrics);
        state.record_alerts(notifications.len() as u64);
    }

    let sample = MetricSample {
        timestamp: Utc::now(),
        cpu: metrics.cpu,
        ram: metrics.ram,
        disk: metrics.disk,
    };

    {
        let mut history = metric_history.lock().await;
        history.push(sample);
    }

    if let Some(store) = reporting_store
        && let Err(error) = store.record_sample(sample)
    {
        log::warn!("reporting_store_write_failed error={}", error);
    }

    let owner_chat_id = match config.owner_chat_id() {
        Ok(chat_id) => chat_id,
        Err(error) => {
            log::error!("CRITICAL: invalid owner chat id in config: {}", error);
            return;
        }
    };

    let muted_until = {
        let state = state.lock().await;
        state.muted_until
    };
    if let Some(until) = muted_until
        && Utc::now() < until
    {
        return;
    }

    for notification in notifications {
        if let Err(error) = bot.send_message(owner_chat_id, notification).await {
            log::error!(
                "CRITICAL: Failed to send alert to {}: {}",
                owner_chat_id.0,
                error
            );
        }
    }
}
