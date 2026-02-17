use std::collections::VecDeque;

use chrono::{DateTime, Duration, Utc};

const DEFAULT_RETENTION_SECS: u64 = 7 * 24 * 3600;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct MetricSample {
    pub timestamp: DateTime<Utc>,
    pub cpu: f32,
    pub ram: f32,
    pub disk: f32,
}

#[derive(Debug)]
pub struct MetricHistory {
    samples: VecDeque<MetricSample>,
    capacity: usize,
}

impl MetricHistory {
    pub fn with_monitor_interval_secs(monitor_interval_secs: u64) -> Self {
        Self::with_retention_secs(monitor_interval_secs, DEFAULT_RETENTION_SECS)
    }

    pub fn with_retention_secs(monitor_interval_secs: u64, retention_secs: u64) -> Self {
        let computed_capacity = if monitor_interval_secs == 0 {
            1
        } else {
            (retention_secs / monitor_interval_secs).max(1)
        } as usize;

        Self {
            samples: VecDeque::with_capacity(computed_capacity),
            capacity: computed_capacity,
        }
    }

    pub fn push(&mut self, sample: MetricSample) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn latest_window(&self, minutes: i64) -> Vec<MetricSample> {
        let now = Utc::now();
        let cutoff = now - Duration::minutes(minutes.max(1));

        self.samples
            .iter()
            .copied()
            .filter(|sample| sample.timestamp >= cutoff)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::{MetricHistory, MetricSample};

    #[test]
    fn computes_capacity_from_monitor_interval() {
        let history = MetricHistory::with_monitor_interval_secs(30);
        assert_eq!(history.capacity, 20160);

        let minimum = MetricHistory::with_monitor_interval_secs(0);
        assert_eq!(minimum.capacity, 1);
    }

    #[test]
    fn keeps_capacity_by_overwriting_oldest() {
        let mut history = MetricHistory::with_retention_secs(1800, 3600);
        let start = Utc::now();

        history.push(MetricSample {
            timestamp: start,
            cpu: 10.0,
            ram: 20.0,
            disk: 30.0,
        });
        history.push(MetricSample {
            timestamp: start + Duration::minutes(30),
            cpu: 40.0,
            ram: 50.0,
            disk: 60.0,
        });
        history.push(MetricSample {
            timestamp: start + Duration::minutes(60),
            cpu: 70.0,
            ram: 80.0,
            disk: 90.0,
        });

        let samples = history.latest_window(180);
        assert_eq!(samples.len(), 2);
        assert!((samples[0].cpu - 40.0).abs() < f32::EPSILON);
        assert!((samples[1].cpu - 70.0).abs() < f32::EPSILON);
    }

    #[test]
    fn latest_window_preserves_time_order() {
        let mut history = MetricHistory::with_retention_secs(60, 24 * 3600);
        let now = Utc::now();

        history.push(MetricSample {
            timestamp: now - Duration::minutes(50),
            cpu: 10.0,
            ram: 10.0,
            disk: 10.0,
        });
        history.push(MetricSample {
            timestamp: now - Duration::minutes(40),
            cpu: 20.0,
            ram: 20.0,
            disk: 20.0,
        });
        history.push(MetricSample {
            timestamp: now - Duration::minutes(10),
            cpu: 30.0,
            ram: 30.0,
            disk: 30.0,
        });

        let samples = history.latest_window(45);
        assert_eq!(samples.len(), 2);
        assert!(samples[0].timestamp < samples[1].timestamp);
        assert!((samples[0].cpu - 20.0).abs() < f32::EPSILON);
        assert!((samples[1].cpu - 30.0).abs() < f32::EPSILON);
    }
}