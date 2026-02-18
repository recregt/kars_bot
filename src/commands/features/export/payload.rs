use std::fmt::Write;

use serde::Serialize;

use super::parser::{ExportFormat, ExportMetric};

#[derive(Serialize)]
struct JsonExportRow {
    timestamp_utc: String,
    metric: String,
    value_percent: f32,
}

pub(super) fn build_export_payload(
    samples: &[crate::monitor::MetricSample],
    metric: ExportMetric,
    format: ExportFormat,
) -> Result<Vec<u8>, String> {
    match format {
        ExportFormat::Csv => Ok(build_csv(samples, metric).into_bytes()),
        ExportFormat::Json => build_json(samples, metric),
    }
}

fn build_csv(samples: &[crate::monitor::MetricSample], metric: ExportMetric) -> String {
    let mut out = String::from("timestamp_utc,metric,value_percent\n");
    for sample in samples {
        let _ = writeln!(
            out,
            "{},{},{:.2}",
            sample.timestamp.to_rfc3339(),
            metric.as_str(),
            metric.value(sample)
        );
    }
    out
}

fn build_json(
    samples: &[crate::monitor::MetricSample],
    metric: ExportMetric,
) -> Result<Vec<u8>, String> {
    let rows: Vec<JsonExportRow> = samples
        .iter()
        .map(|sample| JsonExportRow {
            timestamp_utc: sample.timestamp.to_rfc3339(),
            metric: metric.as_str().to_string(),
            value_percent: metric.value(sample),
        })
        .collect();

    serde_json::to_vec_pretty(&rows).map_err(|error| error.to_string())
}
