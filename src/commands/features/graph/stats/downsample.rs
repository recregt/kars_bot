use chrono::{DateTime, Utc};

use crate::monitor::MetricSample;

use super::super::types::GraphMetric;

#[derive(Clone, Copy)]
pub(crate) struct GraphPoint {
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) value: f32,
}

struct BucketAggregate {
    min: GraphPoint,
    max: GraphPoint,
}

pub(crate) fn downsample_points(
    samples: &[MetricSample],
    metric: GraphMetric,
    width_px: usize,
) -> Vec<GraphPoint> {
    if samples.len() <= 2 || samples.len() <= width_px {
        return samples
            .iter()
            .map(|sample| GraphPoint {
                timestamp: sample.timestamp,
                value: metric.value(sample),
            })
            .collect();
    }

    let start_ts = samples
        .first()
        .map(|sample| sample.timestamp.timestamp_millis());
    let end_ts = samples
        .last()
        .map(|sample| sample.timestamp.timestamp_millis());
    let (Some(start_ts), Some(end_ts)) = (start_ts, end_ts) else {
        return Vec::new();
    };

    if end_ts <= start_ts {
        return samples
            .iter()
            .map(|sample| GraphPoint {
                timestamp: sample.timestamp,
                value: metric.value(sample),
            })
            .collect();
    }

    let bucket_count = width_px.max(1);
    let mut buckets: Vec<Option<BucketAggregate>> =
        std::iter::repeat_with(|| None).take(bucket_count).collect();

    for sample in samples {
        let position =
            (sample.timestamp.timestamp_millis() - start_ts) as f64 / (end_ts - start_ts) as f64;
        let bucket_index = (position * (bucket_count - 1) as f64).floor() as usize;
        let point = GraphPoint {
            timestamp: sample.timestamp,
            value: metric.value(sample),
        };

        let bucket = &mut buckets[bucket_index];
        match bucket {
            Some(existing) => {
                if point.value < existing.min.value {
                    existing.min = point;
                }
                if point.value > existing.max.value {
                    existing.max = point;
                }
            }
            None => {
                *bucket = Some(BucketAggregate {
                    min: point,
                    max: point,
                });
            }
        }
    }

    let mut reduced = Vec::with_capacity(bucket_count * 2);
    for bucket in buckets.into_iter().flatten() {
        if bucket.min.timestamp <= bucket.max.timestamp {
            reduced.push(bucket.min);
            if bucket.max.timestamp != bucket.min.timestamp {
                reduced.push(bucket.max);
            }
        } else {
            reduced.push(bucket.max);
            if bucket.max.timestamp != bucket.min.timestamp {
                reduced.push(bucket.min);
            }
        }
    }

    reduced
}
