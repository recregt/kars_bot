use serde::{Deserialize, Serialize};

use crate::monitor::MetricSample;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct StoredMetricSample {
    pub timestamp_utc: String,
    pub cpu: f32,
    pub ram: f32,
    pub disk: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct DailyRollup {
    pub day_utc: String,
    pub sample_count: u64,
    pub cpu_sum: f64,
    pub cpu_min: f32,
    pub cpu_max: f32,
    pub ram_sum: f64,
    pub ram_min: f32,
    pub ram_max: f32,
    pub disk_sum: f64,
    pub disk_min: f32,
    pub disk_max: f32,
}

impl DailyRollup {
    pub(super) fn new(day_utc: String, sample: MetricSample) -> Self {
        Self {
            day_utc,
            sample_count: 1,
            cpu_sum: f64::from(sample.cpu),
            cpu_min: sample.cpu,
            cpu_max: sample.cpu,
            ram_sum: f64::from(sample.ram),
            ram_min: sample.ram,
            ram_max: sample.ram,
            disk_sum: f64::from(sample.disk),
            disk_min: sample.disk,
            disk_max: sample.disk,
        }
    }

    pub(super) fn update_with_sample(&mut self, sample: MetricSample) {
        self.sample_count = self.sample_count.saturating_add(1);
        self.cpu_sum += f64::from(sample.cpu);
        self.ram_sum += f64::from(sample.ram);
        self.disk_sum += f64::from(sample.disk);
        self.cpu_min = self.cpu_min.min(sample.cpu);
        self.cpu_max = self.cpu_max.max(sample.cpu);
        self.ram_min = self.ram_min.min(sample.ram);
        self.ram_max = self.ram_max.max(sample.ram);
        self.disk_min = self.disk_min.min(sample.disk);
        self.disk_max = self.disk_max.max(sample.disk);
    }
}

#[derive(Debug, Clone)]
pub struct RollingMetricSummary {
    pub sample_count: u64,
    pub cpu_avg: f32,
    pub cpu_min: f32,
    pub cpu_max: f32,
    pub ram_avg: f32,
    pub ram_min: f32,
    pub ram_max: f32,
    pub disk_avg: f32,
    pub disk_min: f32,
    pub disk_max: f32,
    cpu_sum: f64,
    ram_sum: f64,
    disk_sum: f64,
}

impl RollingMetricSummary {
    pub(super) fn empty() -> Self {
        Self {
            sample_count: 0,
            cpu_avg: 0.0,
            cpu_min: f32::MAX,
            cpu_max: f32::MIN,
            ram_avg: 0.0,
            ram_min: f32::MAX,
            ram_max: f32::MIN,
            disk_avg: 0.0,
            disk_min: f32::MAX,
            disk_max: f32::MIN,
            cpu_sum: 0.0,
            ram_sum: 0.0,
            disk_sum: 0.0,
        }
    }

    pub(super) fn accumulate_rollup(&mut self, rollup: &DailyRollup) {
        self.sample_count = self.sample_count.saturating_add(rollup.sample_count);
        self.cpu_sum += rollup.cpu_sum;
        self.ram_sum += rollup.ram_sum;
        self.disk_sum += rollup.disk_sum;
        self.cpu_min = self.cpu_min.min(rollup.cpu_min);
        self.cpu_max = self.cpu_max.max(rollup.cpu_max);
        self.ram_min = self.ram_min.min(rollup.ram_min);
        self.ram_max = self.ram_max.max(rollup.ram_max);
        self.disk_min = self.disk_min.min(rollup.disk_min);
        self.disk_max = self.disk_max.max(rollup.disk_max);
    }

    pub(super) fn finalize(mut self) -> Self {
        if self.sample_count == 0 {
            return self;
        }

        self.cpu_avg = (self.cpu_sum / self.sample_count as f64) as f32;
        self.ram_avg = (self.ram_sum / self.sample_count as f64) as f32;
        self.disk_avg = (self.disk_sum / self.sample_count as f64) as f32;

        self
    }
}
