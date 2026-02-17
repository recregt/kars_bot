use chrono::{DateTime, Utc};

use super::{DailySummaryAccumulator, DailySummaryReport};
use crate::monitor::provider::Metrics;

impl DailySummaryAccumulator {
    pub(super) fn add_sample(&mut self, metrics: Metrics) {
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

    pub(super) fn add_alerts(&mut self, count: u64) {
        self.alert_count += count;
    }

    pub(super) fn take_report(&mut self, now: DateTime<Utc>) -> Option<DailySummaryReport> {
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
