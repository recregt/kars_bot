use std::sync::Arc;

use tokio::sync::{Notify, RwLock};

use crate::{
    bot_runtime::BotRuntime,
    capabilities::Capabilities,
    config::{Config, Graph, RuntimeConfig},
    contracts::{AnomalyStorage, ReportingStorage},
    monitor_context::MonitorContext,
    reporting_store::ReportingStore,
};

#[derive(Clone)]
pub struct AppContext {
    pub config: Config,
    pub runtime_config: Arc<RwLock<RuntimeConfig>>,
    pub config_path: Arc<String>,
    pub graph_runtime: Arc<RwLock<Graph>>,
    pub runtime_update_notify: Arc<Notify>,
    pub monitor: MonitorContext,
    pub bot_runtime: BotRuntime,
    pub capabilities: Arc<Capabilities>,
    pub reporting_store: Arc<dyn ReportingStorage>,
    pub anomaly_storage: Arc<dyn AnomalyStorage>,
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
        let reporting_store = ReportingStore::new_arc_from_config(&config);
        let anomaly_storage: Arc<dyn AnomalyStorage> =
            Arc::new(crate::anomaly_db::FileAnomalyStorage::new());

        Self {
            config,
            runtime_config: Arc::new(RwLock::new(runtime_config)),
            config_path: Arc::new(config_path.into()),
            graph_runtime: Arc::new(RwLock::new(graph_runtime)),
            runtime_update_notify: Arc::new(Notify::new()),
            monitor: MonitorContext::new(monitor_interval),
            bot_runtime: BotRuntime::new(command_concurrency),
            capabilities: Arc::new(capabilities),
            reporting_store,
            anomaly_storage,
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
    use tokio::time::{Duration, timeout};

    use crate::{
        capabilities::Capabilities,
        config::{ReportingStoreConfig, RuntimeConfig},
        test_utils::base_test_config,
    };

    use super::AppContext;

    fn test_config() -> crate::config::Config {
        let mut config = base_test_config();
        config.alerts.cpu = 85.0;
        config.alerts.ram = 90.0;
        config.alerts.disk = 90.0;
        config.alerts.cooldown_secs = 300;
        config.alerts.hysteresis = 3.0;
        config.reporting_store = ReportingStoreConfig {
            enabled: false,
            ..ReportingStoreConfig::default()
        };
        config
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

    #[tokio::test]
    async fn runtime_config_concurrent_reads_and_updates_remain_consistent() {
        let app = AppContext::new(test_config(), 2, "config.toml", Capabilities::detect());

        let writer_app = app.clone();
        let writer = tokio::spawn(async move {
            for offset in 0..100u64 {
                writer_app
                    .update_runtime_config(RuntimeConfig {
                        alerts: writer_app.config.alerts.clone(),
                        monitor_interval: 10 + offset,
                        command_timeout_secs: 30 + offset,
                        graph: writer_app.config.graph.clone(),
                    })
                    .await;
            }
        });

        let mut readers = Vec::new();
        for _ in 0..8 {
            let reader_app = app.clone();
            readers.push(tokio::spawn(async move {
                for _ in 0..200 {
                    let snapshot = reader_app.runtime_config.read().await.clone();
                    assert!(snapshot.monitor_interval >= 10);
                    assert!(snapshot.command_timeout_secs >= 30);
                }
            }));
        }

        writer.await.expect("writer should complete");
        for reader in readers {
            reader.await.expect("reader should complete");
        }

        let final_snapshot = app.runtime_config.read().await.clone();
        assert!(final_snapshot.monitor_interval >= 10);
        assert!(final_snapshot.command_timeout_secs >= 30);
    }
}
