use chrono::{DateTime, Utc};

use crate::monitor::MetricSample;

use super::super::types::GraphMetric;

pub(crate) struct AnomalyAssessment {
    pub(crate) spike_detected: bool,
    pub(crate) sustained_high_load: bool,
}

impl AnomalyAssessment {
    pub(crate) fn labels(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if self.spike_detected {
            labels.push("[!] SPIKE DETECTED");
        }
        if self.sustained_high_load {
            labels.push("[!!!] SUSTAINED HIGH LOAD");
        }
        labels
    }
}

const SPIKE_STDDEV_MULTIPLIER: f64 = 3.0;
const SPIKE_RETURN_STDDEV_MULTIPLIER: f64 = 1.0;
const SPIKE_RETURN_WINDOW_SECS: i64 = 120;
const SUSTAINED_MIN_DURATION_SECS: i64 = 5 * 60;

pub(crate) fn assess_anomaly_labels(
    metric: GraphMetric,
    samples: &[MetricSample],
    threshold: f32,
) -> Option<AnomalyAssessment> {
    if samples.len() < 2 {
        return None;
    }

    let values: Vec<f64> = samples
        .iter()
        .map(|sample| metric.value(sample) as f64)
        .collect();

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    let stddev = variance.sqrt();

    let spike_detected = detect_spike(samples, metric, mean, stddev);
    let sustained_high_load = detect_sustained_high_load(samples, metric, threshold);

    Some(AnomalyAssessment {
        spike_detected,
        sustained_high_load,
    })
}

fn detect_spike(samples: &[MetricSample], metric: GraphMetric, mean: f64, stddev: f64) -> bool {
    if stddev <= f64::EPSILON {
        return false;
    }

    let spike_threshold = mean + (SPIKE_STDDEV_MULTIPLIER * stddev);
    let return_threshold = mean + (SPIKE_RETURN_STDDEV_MULTIPLIER * stddev);

    for (idx, sample) in samples.iter().enumerate() {
        let value = metric.value(sample) as f64;
        if value < spike_threshold {
            continue;
        }

        let peak_timestamp = sample.timestamp;
        for next in samples.iter().skip(idx + 1) {
            let elapsed = next
                .timestamp
                .signed_duration_since(peak_timestamp)
                .num_seconds();

            if elapsed > SPIKE_RETURN_WINDOW_SECS {
                break;
            }

            if (metric.value(next) as f64) <= return_threshold {
                return true;
            }
        }
    }

    false
}

fn detect_sustained_high_load(samples: &[MetricSample], metric: GraphMetric, threshold: f32) -> bool {
    let mut block_start: Option<DateTime<Utc>> = None;

    for sample in samples {
        let value = metric.value(sample);
        if value >= threshold {
            if block_start.is_none() {
                block_start = Some(sample.timestamp);
            }

            if let Some(start) = block_start {
                let sustained_secs = sample.timestamp.signed_duration_since(start).num_seconds();
                if sustained_secs >= SUSTAINED_MIN_DURATION_SECS {
                    return true;
                }
            }
        } else {
            block_start = None;
        }
    }

    false
}
