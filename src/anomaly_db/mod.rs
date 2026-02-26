mod maintenance;
mod model;
mod paths;
mod read;
mod write;

pub use maintenance::run_maintenance;
pub use model::AnomalyEvent;

/// Storage abstraction for anomaly database operations.  `record_if_needed` is
/// responsible for threshold logic and durable persistence; `recent` fetches
/// the latest events up to a limit.
use async_trait::async_trait;

#[async_trait]
pub trait AnomalyStorage: Send + Sync {
    async fn record_if_needed(&self, config: &crate::config::Config, cpu: f32, ram: f32, disk: f32);
    async fn recent(&self, config: &crate::config::Config, limit: usize) -> Vec<AnomalyEvent>;
}

/// Concrete implementation that uses the normal filesystem-based storage.
pub struct FileAnomalyStorage;

impl FileAnomalyStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AnomalyStorage for FileAnomalyStorage {
    async fn record_if_needed(
        &self,
        config: &crate::config::Config,
        cpu: f32,
        ram: f32,
        disk: f32,
    ) {
        write::record_anomaly_if_needed(config, cpu, ram, disk);
    }

    async fn recent(&self, config: &crate::config::Config, limit: usize) -> Vec<AnomalyEvent> {
        read::recent_anomalies(config, limit)
    }
}

#[cfg(test)]
/// In-memory fake storage useful for unit tests.  Does not touch filesystem.
pub struct InMemoryAnomalyStorage {
    events: tokio::sync::Mutex<Vec<AnomalyEvent>>,
}

#[cfg(test)]
impl InMemoryAnomalyStorage {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl AnomalyStorage for InMemoryAnomalyStorage {
    async fn record_if_needed(
        &self,
        config: &crate::config::Config,
        cpu: f32,
        ram: f32,
        disk: f32,
    ) {
        if !config.anomaly_db.enabled {
            return;
        }
        let cpu_over = cpu > config.alerts.cpu;
        let ram_over = ram > config.alerts.ram;
        let disk_over = disk > config.alerts.disk;
        if !(cpu_over || ram_over || disk_over) {
            return;
        }
        let event = AnomalyEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            cpu,
            ram,
            disk,
            cpu_threshold: config.alerts.cpu,
            ram_threshold: config.alerts.ram,
            disk_threshold: config.alerts.disk,
            cpu_over,
            ram_over,
            disk_over,
        };
        let mut guard = self.events.lock().await;
        guard.push(event);
    }

    async fn recent(&self, _config: &crate::config::Config, limit: usize) -> Vec<AnomalyEvent> {
        let guard = self.events.lock().await;
        guard.iter().rev().take(limit).cloned().collect()
    }
}
