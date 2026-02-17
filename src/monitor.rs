use std::{path::Path, sync::Arc, time::Instant};

use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use teloxide::prelude::*;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::config::Config;

#[derive(Debug, Default)]
pub struct AlertState {
    cpu_alerting: bool,
    ram_alerting: bool,
    disk_alerting: bool,
    last_cpu_alert: Option<Instant>,
    last_ram_alert: Option<Instant>,
    last_disk_alert: Option<Instant>,
}

impl AlertState {
    fn cpu_should_alert(
        &mut self,
        usage: f32,
        threshold: f32,
        cooldown_secs: u64,
        hysteresis: f32,
        now: Instant,
    ) -> bool {
        should_send_alert(
            usage,
            threshold,
            &mut self.cpu_alerting,
            &mut self.last_cpu_alert,
            cooldown_secs,
            hysteresis,
            now,
        )
    }

    fn ram_should_alert(
        &mut self,
        usage: f32,
        threshold: f32,
        cooldown_secs: u64,
        hysteresis: f32,
        now: Instant,
    ) -> bool {
        should_send_alert(
            usage,
            threshold,
            &mut self.ram_alerting,
            &mut self.last_ram_alert,
            cooldown_secs,
            hysteresis,
            now,
        )
    }

    fn disk_should_alert(
        &mut self,
        usage: f32,
        threshold: f32,
        cooldown_secs: u64,
        hysteresis: f32,
        now: Instant,
    ) -> bool {
        should_send_alert(
            usage,
            threshold,
            &mut self.disk_alerting,
            &mut self.last_disk_alert,
            cooldown_secs,
            hysteresis,
            now,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    cpu: f32,
    ram: f32,
    disk: f32,
}

impl Metrics {
    #[cfg(test)]
    fn new(cpu: f32, ram: f32, disk: f32) -> Self {
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
    fn mock_metrics_exhausted() -> Self {
        Self {
            message: "mock metrics exhausted".to_string(),
        }
    }
}

pub trait MetricsProvider {
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

#[cfg(test)]
pub struct MockMetricsProvider {
    sequence: Vec<Metrics>,
}

#[cfg(test)]
impl MockMetricsProvider {
    fn new(sequence: Vec<Metrics>) -> Self {
        Self { sequence }
    }
}

#[cfg(test)]
impl MetricsProvider for MockMetricsProvider {
    async fn collect_metrics(&mut self) -> Result<Metrics, MonitorError> {
        if self.sequence.is_empty() {
            return Err(MonitorError::mock_metrics_exhausted());
        }

        Ok(self.sequence.remove(0))
    }
}

fn should_send_alert(
    usage: f32,
    threshold: f32,
    is_alerting: &mut bool,
    last_sent: &mut Option<Instant>,
    cooldown_secs: u64,
    hysteresis: f32,
    now: Instant,
) -> bool {
    if !*is_alerting && usage > threshold {
        *is_alerting = true;
        *last_sent = Some(now);
        return true;
    }

    let clear_threshold = (threshold - hysteresis).max(0.0);
    if *is_alerting && usage <= clear_threshold {
        *is_alerting = false;
        return false;
    }

    if *is_alerting {
        if let Some(last) = *last_sent {
            if now.duration_since(last).as_secs() >= cooldown_secs {
                *last_sent = Some(now);
                return true;
            }
        }
    }

    false
}

async fn evaluate_alerts_at(
    config: &Config,
    state: &Arc<Mutex<AlertState>>,
    metrics: Metrics,
    now: Instant,
) -> Vec<String> {
    let mut notifications = Vec::new();

    {
        let mut alert_state = state.lock().await;
        if alert_state.cpu_should_alert(
            metrics.cpu,
            config.alerts.cpu,
            config.alerts.cooldown_secs,
            config.alerts.hysteresis,
            now,
        ) {
            notifications.push(format!("⚠️ ALERT: CPU usage is high ({:.1}%)", metrics.cpu));
        }

        if alert_state.ram_should_alert(
            metrics.ram,
            config.alerts.ram,
            config.alerts.cooldown_secs,
            config.alerts.hysteresis,
            now,
        ) {
            notifications.push(format!("⚠️ ALERT: RAM usage is high ({:.1}%)", metrics.ram));
        }

        if alert_state.disk_should_alert(
            metrics.disk,
            config.alerts.disk,
            config.alerts.cooldown_secs,
            config.alerts.hysteresis,
            now,
        ) {
            notifications.push(format!("⚠️ ALERT: Disk usage is high ({:.1}%)", metrics.disk));
        }
    }

    notifications
}

pub async fn check_alerts<P: MetricsProvider>(
    bot: &Bot,
    config: &Config,
    state: &Arc<Mutex<AlertState>>,
    provider: &mut P,
) {
    let metrics = match provider.collect_metrics().await {
        Ok(metrics) => metrics,
        Err(error) => {
            log::warn!("monitoring provider error: {}", error);
            return;
        }
    };

    let notifications = evaluate_alerts_at(config, state, metrics, Instant::now()).await;

    let owner_chat_id = match config.owner_chat_id() {
        Ok(chat_id) => chat_id,
        Err(error) => {
            log::error!("CRITICAL: invalid owner chat id in config: {}", error);
            return;
        }
    };

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

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::{Duration, Instant}};

    use tokio::sync::Mutex;

    use super::{evaluate_alerts_at, AlertState, Metrics, MetricsProvider, MockMetricsProvider};
    use crate::config::{Alerts, Config};

    fn test_config() -> Config {
        Config {
            bot_token: "token".to_string(),
            owner_id: 1,
            allowed_user_ids: vec![1],
            allowed_chat_ids: Some(vec![1]),
            alerts: Alerts {
                cpu: 80.0,
                ram: 80.0,
                disk: 80.0,
                cooldown_secs: 300,
                hysteresis: 5.0,
            },
            monitor_interval: 10,
            command_timeout_secs: 30,
        }
    }

    #[tokio::test]
    async fn mock_provider_returns_sequence() {
        let mut provider = MockMetricsProvider::new(vec![Metrics::new(81.0, 10.0, 10.0)]);
        let metrics = provider.collect_metrics().await.expect("mock should return metrics");
        assert!(metrics.cpu > 80.0);
    }

    #[tokio::test]
    async fn cooldown_and_hysteresis_work() {
        let config = test_config();
        let state = Arc::new(Mutex::new(AlertState::default()));
        let start = Instant::now();

        let first = evaluate_alerts_at(&config, &state, Metrics::new(90.0, 10.0, 10.0), start).await;
        assert_eq!(first.len(), 1);

        let cooldown_block = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(92.0, 10.0, 10.0),
            start + Duration::from_secs(60),
        )
        .await;
        assert_eq!(cooldown_block.len(), 0);

        let after_cooldown = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(93.0, 10.0, 10.0),
            start + Duration::from_secs(301),
        )
        .await;
        assert_eq!(after_cooldown.len(), 1);

        let clear = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(74.0, 10.0, 10.0),
            start + Duration::from_secs(320),
        )
        .await;
        assert_eq!(clear.len(), 0);

        let retrigger = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(88.0, 10.0, 10.0),
            start + Duration::from_secs(321),
        )
        .await;
        assert_eq!(retrigger.len(), 1);
    }
}