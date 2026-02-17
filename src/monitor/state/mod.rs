use std::time::Instant;

use chrono::{DateTime, Utc};

use super::provider::Metrics;

mod alert_logic;
mod daily_summary;

#[derive(Debug, Default)]
pub struct AlertState {
    pub(crate) cpu_alerting: bool,
    pub(crate) ram_alerting: bool,
    pub(crate) disk_alerting: bool,
    pub(crate) last_cpu_alert: Option<Instant>,
    pub(crate) last_ram_alert: Option<Instant>,
    pub(crate) last_disk_alert: Option<Instant>,
    pub(crate) muted_until: Option<DateTime<Utc>>,
    pub(crate) last_mute_action_at: Option<DateTime<Utc>>,
    pub(crate) daily_summary: DailySummaryAccumulator,
}

#[derive(Debug, Clone)]
pub struct AlertSnapshot {
    pub cpu_alerting: bool,
    pub ram_alerting: bool,
    pub disk_alerting: bool,
    pub muted_until: Option<DateTime<Utc>>,
    pub last_daily_summary_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct DailySummaryReport {
    pub cpu_avg: f32,
    pub cpu_min: f32,
    pub cpu_max: f32,
    pub ram_avg: f32,
    pub ram_min: f32,
    pub ram_max: f32,
    pub disk_avg: f32,
    pub disk_min: f32,
    pub disk_max: f32,
    pub sample_count: u64,
    pub alert_count: u64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub(crate) struct DailySummaryAccumulator {
    sample_count: u64,
    alert_count: u64,
    cpu_sum: f64,
    ram_sum: f64,
    disk_sum: f64,
    cpu_min: Option<f32>,
    cpu_max: Option<f32>,
    ram_min: Option<f32>,
    ram_max: Option<f32>,
    disk_min: Option<f32>,
    disk_max: Option<f32>,
    last_generated_at: Option<DateTime<Utc>>,
}

impl AlertState {
    pub(crate) fn record_metrics(&mut self, metrics: Metrics) {
        self.daily_summary.add_sample(metrics);
    }

    pub(crate) fn record_alerts(&mut self, count: u64) {
        self.daily_summary.add_alerts(count);
    }

    pub(crate) fn take_daily_summary_report(
        &mut self,
        now: DateTime<Utc>,
    ) -> Option<DailySummaryReport> {
        self.daily_summary.take_report(now)
    }

    pub(crate) fn last_daily_summary_at(&self) -> Option<DateTime<Utc>> {
        self.daily_summary.last_generated_at
    }
}
