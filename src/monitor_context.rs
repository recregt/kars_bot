use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::monitor::{AlertState, MetricHistory};

#[derive(Clone)]
pub struct MonitorContext {
    pub alert_state: Arc<Mutex<AlertState>>,
    pub metric_history: Arc<Mutex<MetricHistory>>,
    pub last_monitor_tick: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl MonitorContext {
    pub fn new(monitor_interval: u64) -> Self {
        Self {
            alert_state: Arc::new(Mutex::new(AlertState::default())),
            metric_history: Arc::new(Mutex::new(MetricHistory::with_monitor_interval_secs(
                monitor_interval,
            ))),
            last_monitor_tick: Arc::new(Mutex::new(None)),
        }
    }
}
