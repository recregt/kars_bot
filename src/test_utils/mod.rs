use std::sync::Arc;

use tokio::sync::Mutex;

use crate::config::Config;
use crate::monitor::{AlertState, MetricHistory};

pub fn base_test_config() -> Config {
    Config {
        bot_token: "token".to_string(),
        owner_id: 1,
        monitor_interval: 10,
        command_timeout_secs: 30,
        alerts: Default::default(),
        daily_summary: Default::default(),
        weekly_report: Default::default(),
        graph: Default::default(),
        anomaly_db: Default::default(),
        simulation: Default::default(),
        reporting_store: Default::default(),
        release_notifier: Default::default(),
        security: Default::default(),
    }
}

pub fn test_alert_state() -> Arc<Mutex<AlertState>> {
    Arc::new(Mutex::new(AlertState::default()))
}

pub fn test_metric_history(monitor_interval_secs: u64) -> Arc<Mutex<MetricHistory>> {
    Arc::new(Mutex::new(MetricHistory::with_monitor_interval_secs(
        monitor_interval_secs,
    )))
}
