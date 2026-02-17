use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{Mutex, Semaphore};

use crate::{config::Config, monitor::AlertState};

#[derive(Clone)]
pub struct AppContext {
    pub config: Config,
    pub alert_state: Arc<Mutex<AlertState>>,
    pub last_monitor_tick: Arc<Mutex<Option<DateTime<Utc>>>>,
    pub command_slots: Arc<Semaphore>,
}

impl AppContext {
    pub fn new(config: Config, command_concurrency: usize) -> Self {
        Self {
            config,
            alert_state: Arc::new(Mutex::new(AlertState::default())),
            last_monitor_tick: Arc::new(Mutex::new(None)),
            command_slots: Arc::new(Semaphore::new(command_concurrency)),
        }
    }
}