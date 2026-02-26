use std::sync::Arc;

use tokio::sync::Mutex;

use crate::architecture::ports::{
    AnomalyStoragePort, MetricsProviderPort, NotifierPort, ReportingStoragePort,
};
use crate::config::{Config, RuntimeConfig};

use super::super::{
    evaluator::evaluate_alerts_at,
    history::{MetricHistory, MetricSample},
    state::AlertState,
};

use super::clock::{Clock, SystemClock};

pub struct CheckAlertsContext<'a, N: NotifierPort> {
    pub notifier: &'a N,
    pub config: &'a Config,
    pub runtime_config: &'a RuntimeConfig,
    pub reporting_store: &'a dyn ReportingStoragePort,
    pub anomaly_storage: &'a dyn AnomalyStoragePort,
    pub state: &'a Arc<Mutex<AlertState>>,
    pub metric_history: &'a Arc<Mutex<MetricHistory>>,
}

pub async fn check_alerts<P: MetricsProviderPort, N: NotifierPort>(
    context: CheckAlertsContext<'_, N>,
    provider: &mut P,
) {
    let clock = SystemClock;
    check_alerts_with_clock(context, provider, &clock).await;
}

pub(super) async fn check_alerts_with_clock<
    P: MetricsProviderPort,
    N: NotifierPort,
    C: Clock + ?Sized,
>(
    context: CheckAlertsContext<'_, N>,
    provider: &mut P,
    clock: &C,
) {
    let CheckAlertsContext {
        notifier,
        config,
        runtime_config,
        reporting_store,
        anomaly_storage,
        state,
        metric_history,
    } = context;

    let metrics = match provider.collect_metrics().await {
        Ok(metrics) => metrics,
        Err(error) => {
            log::warn!("monitoring provider error: {error}");
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

    anomaly_storage
        .record_if_needed(&effective_config, metrics.cpu, metrics.ram, metrics.disk)
        .await;

    let notifications =
        evaluate_alerts_at(&effective_config, state, metrics, clock.now_instant()).await;

    {
        let mut state = state.lock().await;
        state.record_metrics(metrics);
        state.record_alerts(notifications.len() as u64);
    }

    let sample = MetricSample {
        timestamp: clock.now_utc(),
        cpu: metrics.cpu,
        ram: metrics.ram,
        disk: metrics.disk,
    };

    {
        let mut history = metric_history.lock().await;
        history.push(sample);
    }

    if let Err(error) = reporting_store.record_sample(sample) {
        log::warn!("reporting_store_write_failed error={error}");
    }

    let owner_chat_id = match config.owner_chat_id() {
        Ok(chat_id) => chat_id,
        Err(error) => {
            log::error!("CRITICAL: invalid owner chat id in config: {error}");
            return;
        }
    };

    let muted_until = {
        let state = state.lock().await;
        state.muted_until
    };
    if let Some(until) = muted_until
        && clock.now_utc() < until
    {
        return;
    }

    for notification in notifications {
        if let Err(error) = notifier.send_message(owner_chat_id, notification).await {
            log::error!(
                "CRITICAL: Failed to send alert to {}: {}",
                owner_chat_id.0,
                error
            );
        }
    }
}
