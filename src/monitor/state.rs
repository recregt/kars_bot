use std::time::Instant;

use chrono::{DateTime, Utc};

use super::provider::Metrics;

#[derive(Debug, Default)]
pub struct AlertState {
    pub(crate) cpu_alerting: bool,
    pub(crate) ram_alerting: bool,
    pub(crate) disk_alerting: bool,
    pub(crate) last_cpu_alert: Option<Instant>,
    pub(crate) last_ram_alert: Option<Instant>,
    pub(crate) last_disk_alert: Option<Instant>,
    pub(crate) muted_until: Option<DateTime<Utc>>,
    pub(crate) daily_summary: DailySummaryAccumulator,
}

impl AlertState {
    pub(crate) fn cpu_should_alert(
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

    pub(crate) fn ram_should_alert(
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

    pub(crate) fn disk_should_alert(
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

impl DailySummaryAccumulator {
    fn add_sample(&mut self, metrics: Metrics) {
        self.sample_count += 1;
        self.cpu_sum += metrics.cpu as f64;
        self.ram_sum += metrics.ram as f64;
        self.disk_sum += metrics.disk as f64;

        self.cpu_min = Some(self.cpu_min.map_or(metrics.cpu, |value| value.min(metrics.cpu)));
        self.cpu_max = Some(self.cpu_max.map_or(metrics.cpu, |value| value.max(metrics.cpu)));
        self.ram_min = Some(self.ram_min.map_or(metrics.ram, |value| value.min(metrics.ram)));
        self.ram_max = Some(self.ram_max.map_or(metrics.ram, |value| value.max(metrics.ram)));
        self.disk_min = Some(self.disk_min.map_or(metrics.disk, |value| value.min(metrics.disk)));
        self.disk_max = Some(self.disk_max.map_or(metrics.disk, |value| value.max(metrics.disk)));
    }

    fn add_alerts(&mut self, count: u64) {
        self.alert_count += count;
    }

    fn take_report(&mut self, now: DateTime<Utc>) -> Option<DailySummaryReport> {
        if self.sample_count == 0 {
            self.last_generated_at = Some(now);
            self.alert_count = 0;
            return None;
        }

        let sample_count = self.sample_count;
        let report = DailySummaryReport {
            cpu_avg: (self.cpu_sum / sample_count as f64) as f32,
            cpu_min: self.cpu_min.unwrap_or(0.0),
            cpu_max: self.cpu_max.unwrap_or(0.0),
            ram_avg: (self.ram_sum / sample_count as f64) as f32,
            ram_min: self.ram_min.unwrap_or(0.0),
            ram_max: self.ram_max.unwrap_or(0.0),
            disk_avg: (self.disk_sum / sample_count as f64) as f32,
            disk_min: self.disk_min.unwrap_or(0.0),
            disk_max: self.disk_max.unwrap_or(0.0),
            sample_count,
            alert_count: self.alert_count,
            generated_at: now,
        };

        self.sample_count = 0;
        self.alert_count = 0;
        self.cpu_sum = 0.0;
        self.ram_sum = 0.0;
        self.disk_sum = 0.0;
        self.cpu_min = None;
        self.cpu_max = None;
        self.ram_min = None;
        self.ram_max = None;
        self.disk_min = None;
        self.disk_max = None;
        self.last_generated_at = Some(now);

        Some(report)
    }
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