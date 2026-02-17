use crate::monitor::MetricSample;

use super::super::types::GraphMetric;

pub(crate) struct MetricSummary {
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) avg: f32,
}

pub(crate) fn compute_metric_summary(
    metric: GraphMetric,
    samples: &[MetricSample],
) -> Option<MetricSummary> {
    let mut values = samples.iter().map(|sample| metric.value(sample));
    let first = values.next()?;

    let mut min_value = first;
    let mut max_value = first;
    let mut sum = first as f64;
    let mut count: usize = 1;

    for value in values {
        if value < min_value {
            min_value = value;
        }
        if value > max_value {
            max_value = value;
        }
        sum += value as f64;
        count += 1;
    }

    Some(MetricSummary {
        min: min_value,
        max: max_value,
        avg: (sum / count as f64) as f32,
    })
}
