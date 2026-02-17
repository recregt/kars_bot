mod anomaly;
mod downsample;
mod summary;

pub(super) use anomaly::assess_anomaly_labels;
pub(super) use downsample::{downsample_points, GraphPoint};
pub(super) use summary::compute_metric_summary;

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use std::time::Instant;

    use crate::monitor::MetricSample;

    use super::{assess_anomaly_labels, compute_metric_summary, downsample_points};
    use crate::commands::features::graph::types::GraphMetric;

    #[test]
    fn computes_summary_for_data_path() {
        let now = Utc::now();
        let samples = vec![
            MetricSample {
                timestamp: now,
                cpu: 10.0,
                ram: 40.0,
                disk: 50.0,
            },
            MetricSample {
                timestamp: now + Duration::minutes(1),
                cpu: 30.0,
                ram: 60.0,
                disk: 70.0,
            },
        ];

        let summary = compute_metric_summary(GraphMetric::Cpu, &samples).expect("summary expected");
        assert!((summary.min - 10.0).abs() < f32::EPSILON);
        assert!((summary.max - 30.0).abs() < f32::EPSILON);
        assert!((summary.avg - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn downsamples_long_input_for_data_path() {
        let start = Utc::now();
        let mut samples = Vec::new();
        for idx in 0..100 {
            samples.push(MetricSample {
                timestamp: start + Duration::seconds(idx),
                cpu: (idx % 100) as f32,
                ram: 10.0,
                disk: 10.0,
            });
        }

        let reduced = downsample_points(&samples, GraphMetric::Cpu, 10);
        assert!(!reduced.is_empty());
        assert!(reduced.len() <= 20);
    }

    #[test]
    fn performance_smoke_long_window_high_sample_count() {
        let start = Utc::now();
        let mut samples = Vec::with_capacity(120_000);
        for idx in 0..120_000 {
            samples.push(MetricSample {
                timestamp: start + Duration::seconds(idx as i64),
                cpu: (idx % 100) as f32,
                ram: ((idx + 20) % 100) as f32,
                disk: ((idx + 40) % 100) as f32,
            });
        }

        let timer = Instant::now();
        let reduced = downsample_points(&samples, GraphMetric::Cpu, 1200);
        let elapsed = timer.elapsed();

        assert!(!reduced.is_empty());
        assert!(reduced.len() <= 2400);
        assert!(elapsed.as_secs() < 5);
    }

    #[test]
    fn labels_spike_when_value_jumps_and_returns_quickly() {
        let start = Utc::now();
        let mut samples = Vec::new();
        for idx in 0..12 {
            samples.push(MetricSample {
                timestamp: start + Duration::seconds(idx * 30),
                cpu: 20.0 + ((idx % 3) as f32 * 0.5),
                ram: 20.0,
                disk: 20.0,
            });
        }

        samples.push(MetricSample {
            timestamp: start + Duration::seconds(12 * 30),
            cpu: 95.0,
            ram: 20.0,
            disk: 20.0,
        });

        samples.push(MetricSample {
            timestamp: start + Duration::seconds(13 * 30),
            cpu: 20.5,
            ram: 20.0,
            disk: 20.0,
        });

        let assessment =
            assess_anomaly_labels(GraphMetric::Cpu, &samples, 85.0).expect("assessment should exist");
        assert!(assessment.spike_detected);
    }

    #[test]
    fn labels_sustained_when_threshold_exceeded_for_five_minutes() {
        let start = Utc::now();
        let samples = vec![
            MetricSample {
                timestamp: start,
                cpu: 86.0,
                ram: 10.0,
                disk: 10.0,
            },
            MetricSample {
                timestamp: start + Duration::minutes(3),
                cpu: 87.0,
                ram: 10.0,
                disk: 10.0,
            },
            MetricSample {
                timestamp: start + Duration::minutes(5),
                cpu: 88.0,
                ram: 10.0,
                disk: 10.0,
            },
        ];

        let assessment =
            assess_anomaly_labels(GraphMetric::Cpu, &samples, 85.0).expect("assessment should exist");
        assert!(assessment.sustained_high_load);
    }
}
