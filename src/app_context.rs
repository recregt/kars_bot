use std::{sync::Arc, time::Instant};

use chrono::{DateTime, Utc};
use tokio::sync::{Mutex, Notify, RwLock, Semaphore};

use crate::{
    capabilities::Capabilities,
    config::{Config, Graph, RuntimeConfig},
    monitor::{AlertState, MetricHistory},
    reporting_store::ReportingStore,
};

#[derive(Clone)]
pub struct AppContext {
    pub config: Config,
    pub runtime_config: Arc<RwLock<RuntimeConfig>>,
    pub config_path: Arc<String>,
    pub graph_runtime: Arc<RwLock<Graph>>,
    pub alert_state: Arc<Mutex<AlertState>>,
    pub metric_history: Arc<Mutex<MetricHistory>>,
    pub last_graph_command_at: Arc<Mutex<Option<Instant>>>,
    pub last_monitor_tick: Arc<Mutex<Option<DateTime<Utc>>>>,
    pub command_slots: Arc<Semaphore>,
    pub graph_render_slots: Arc<Semaphore>,
    pub capabilities: Arc<Capabilities>,
    pub runtime_update_notify: Arc<Notify>,
    pub reporting_store: Option<ReportingStore>,
}

impl AppContext {
    pub fn new(
        config: Config,
        command_concurrency: usize,
        config_path: impl Into<String>,
        capabilities: Capabilities,
    ) -> Self {
        let monitor_interval = config.monitor_interval;
        let graph_runtime = config.graph.clone();
        let runtime_config = RuntimeConfig::from_config(&config);
        let reporting_store = match ReportingStore::open_from_config(&config) {
            Ok(store) => store,
            Err(error) => {
                log::warn!("reporting_store_disabled reason=open_failed error={}", error);
                None
            }
        };

        Self {
            config,
            runtime_config: Arc::new(RwLock::new(runtime_config)),
            config_path: Arc::new(config_path.into()),
            graph_runtime: Arc::new(RwLock::new(graph_runtime)),
            alert_state: Arc::new(Mutex::new(AlertState::default())),
            metric_history: Arc::new(Mutex::new(MetricHistory::with_monitor_interval_secs(
                monitor_interval,
            ))),
            last_graph_command_at: Arc::new(Mutex::new(None)),
            last_monitor_tick: Arc::new(Mutex::new(None)),
            command_slots: Arc::new(Semaphore::new(command_concurrency)),
            graph_render_slots: Arc::new(Semaphore::new(1)),
            capabilities: Arc::new(capabilities),
            runtime_update_notify: Arc::new(Notify::new()),
            reporting_store,
        }
    }

    pub async fn update_graph_runtime(&self, graph: Graph) {
        let mut runtime = self.graph_runtime.write().await;
        *runtime = graph;
    }

    pub async fn update_runtime_config(&self, runtime_config: RuntimeConfig) {
        {
            let mut runtime = self.runtime_config.write().await;
            *runtime = runtime_config.clone();
        }

        self.update_graph_runtime(runtime_config.graph).await;
        self.runtime_update_notify.notify_waiters();
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{timeout, Duration};

    use crate::{
        capabilities::Capabilities,
        config::{
            Alerts, AnomalyDb, Config, DailySummary, Graph, ReportingStoreConfig, RuntimeConfig,
            ReleaseNotifierConfig, Simulation, WeeklyReport,
        },
    };

    use super::AppContext;

    fn test_config() -> Config {
        Config {
            bot_token: "token".to_string(),
            owner_id: 1,
            alerts: Alerts {
                cpu: 85.0,
                ram: 90.0,
                disk: 90.0,
                cooldown_secs: 300,
                hysteresis: 3.0,
            },
            monitor_interval: 10,
            command_timeout_secs: 30,
            daily_summary: DailySummary::default(),
            weekly_report: WeeklyReport::default(),
            graph: Graph::default(),
            anomaly_db: AnomalyDb::default(),
            simulation: Simulation::default(),
            reporting_store: ReportingStoreConfig {
                enabled: false,
                ..ReportingStoreConfig::default()
            },
            release_notifier: ReleaseNotifierConfig::default(),
        }
    }

    #[tokio::test]
    async fn runtime_update_triggers_notify() {
        let app = AppContext::new(test_config(), 2, "config.toml", Capabilities::detect());
        let notify = app.runtime_update_notify.clone();

        let wait = tokio::spawn(async move { notify.notified().await });
        tokio::task::yield_now().await;

        app.update_runtime_config(RuntimeConfig {
            alerts: app.config.alerts.clone(),
            monitor_interval: 30,
            command_timeout_secs: 60,
            graph: app.config.graph.clone(),
        })
        .await;

        timeout(Duration::from_secs(1), wait)
            .await
            .expect("notify wait should complete")
            .expect("join should succeed");
    }
}