use std::path::Path;

use async_trait::async_trait;
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub(crate) cpu: f32,
    pub(crate) ram: f32,
    pub(crate) disk: f32,
}

impl Metrics {
    #[cfg(test)]
    pub(crate) fn new(cpu: f32, ram: f32, disk: f32) -> Self {
        Self { cpu, ram, disk }
    }
}

#[derive(Debug, Error, Clone)]
#[error("{message}")]
pub struct MonitorError {
    message: String,
}

impl MonitorError {
    #[cfg(test)]
    pub(crate) fn mock_metrics_exhausted() -> Self {
        Self {
            message: "mock metrics exhausted".to_string(),
        }
    }
}

#[async_trait]
pub trait MetricsProvider: Send {
    async fn collect_metrics(&mut self) -> Result<Metrics, MonitorError>;
}

pub struct RealMetricsProvider {
    system: System,
}

impl RealMetricsProvider {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

pub struct SimulatedMetricsProvider {
    tick: u64,
}

impl SimulatedMetricsProvider {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}

#[async_trait]
impl MetricsProvider for SimulatedMetricsProvider {
    async fn collect_metrics(&mut self) -> Result<Metrics, MonitorError> {
        self.tick = self.tick.saturating_add(1);
        let phase = self.tick as f32 / 8.0;

        let mut cpu = 45.0 + (phase.sin() * 20.0);
        let mut ram = 55.0 + ((phase * 0.7).sin() * 12.0);
        let mut disk = 60.0 + ((phase * 0.2).sin() * 5.0);

        if self.tick.is_multiple_of(30) {
            cpu = 95.0;
        }
        if self.tick.is_multiple_of(47) {
            ram = 93.0;
        }
        if self.tick.is_multiple_of(83) {
            disk = 91.0;
        }

        Ok(Metrics {
            cpu: cpu.clamp(0.0, 100.0),
            ram: ram.clamp(0.0, 100.0),
            disk: disk.clamp(0.0, 100.0),
        })
    }
}

#[async_trait]
impl MetricsProvider for RealMetricsProvider {
    async fn collect_metrics(&mut self) -> Result<Metrics, MonitorError> {
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.system.refresh_disks_list();
        self.system.refresh_disks();

        let cpu = self.system.global_cpu_info().cpu_usage();

        let total_memory = self.system.total_memory() as f32;
        let used_memory = self.system.used_memory() as f32;
        let ram = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };

        let disk = self
            .system
            .disks()
            .iter()
            .find(|disk| disk.mount_point() == Path::new("/"))
            .or_else(|| self.system.disks().first())
            .map(|disk| {
                let total_space = disk.total_space() as f32;
                let used_space = (disk.total_space() - disk.available_space()) as f32;
                if total_space > 0.0 {
                    (used_space / total_space) * 100.0
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        Ok(Metrics { cpu, ram, disk })
    }
}

/// ActiveMetricsProvider kaldırıldı — yerine Box<dyn MetricsProvider> kullanılır.
/// Oluşturmak için: new_metrics_provider(simulation_enabled)
#[async_trait]
impl MetricsProvider for Box<dyn MetricsProvider> {
    async fn collect_metrics(&mut self) -> Result<Metrics, MonitorError> {
        (**self).collect_metrics().await
    }
}

pub fn new_metrics_provider(simulation_enabled: bool) -> Box<dyn MetricsProvider> {
    if simulation_enabled {
        Box::new(SimulatedMetricsProvider::new())
    } else {
        Box::new(RealMetricsProvider::new())
    }
}

#[cfg(test)]
pub(crate) struct MockMetricsProvider {
    sequence: Vec<Metrics>,
}

#[cfg(test)]
impl MockMetricsProvider {
    pub(crate) fn new(sequence: Vec<Metrics>) -> Self {
        Self { sequence }
    }
}

#[cfg(test)]
#[async_trait]
impl MetricsProvider for MockMetricsProvider {
    async fn collect_metrics(&mut self) -> Result<Metrics, MonitorError> {
        if self.sequence.is_empty() {
            return Err(MonitorError::mock_metrics_exhausted());
        }
        Ok(self.sequence.remove(0))
    }
}
