use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::{Datelike, Utc};
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use serde::Serialize;

use crate::config::Config;

use super::model::{AnomalyEvent, AnomalyIndexEntry};
use super::paths::{ensure_db_dirs, paths_from_config};

pub fn record_anomaly_if_needed(config: &Config, cpu: f32, ram: f32, disk: f32) {
    if !config.anomaly_db.enabled {
        return;
    }

    let cpu_over = cpu > config.alerts.cpu;
    let ram_over = ram > config.alerts.ram;
    let disk_over = disk > config.alerts.disk;
    if !(cpu_over || ram_over || disk_over) {
        return;
    }

    let now = Utc::now();
    let timestamp = now.to_rfc3339();
    let event = AnomalyEvent {
        timestamp: timestamp.clone(),
        cpu,
        ram,
        disk,
        cpu_threshold: config.alerts.cpu,
        ram_threshold: config.alerts.ram,
        disk_threshold: config.alerts.disk,
        cpu_over,
        ram_over,
        disk_over,
    };

    let paths = paths_from_config(config);
    if let Err(error) = ensure_db_dirs(&paths) {
        log::warn!("anomaly db: failed to create directory: {}", error);
        return;
    }

    let events_file_name = format!(
        "events-{:04}-{:02}-{:02}.jsonl",
        now.year(),
        now.month(),
        now.day()
    );
    let events_path = paths.events_dir.join(&events_file_name);

    if let Err(error) = append_event_with_rotation(
        &events_path,
        &event,
        config.anomaly_db.max_file_size_bytes,
        config.anomaly_db.retention_days,
    ) {
        log::warn!("anomaly db: failed to write event line: {}", error);
        return;
    }

    let index_file_name = format!(
        "index-{:04}-{:02}-{:02}.jsonl",
        now.year(),
        now.month(),
        now.day()
    );

    let index_entry = AnomalyIndexEntry {
        timestamp,
        cpu,
        ram,
        disk,
        cpu_threshold: config.alerts.cpu,
        ram_threshold: config.alerts.ram,
        disk_threshold: config.alerts.disk,
        cpu_over,
        ram_over,
        disk_over,
    };
    let index_path = paths.index_dir.join(index_file_name);
    if let Err(error) = append_json_line(&index_path, &index_entry) {
        log::warn!("anomaly db: failed to write index line: {}", error);
    }
}

fn append_json_line<T: Serialize>(path: &Path, value: &T) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, value).map_err(std::io::Error::other)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn append_event_with_rotation(
    path: &Path,
    event: &AnomalyEvent,
    max_file_size_bytes: u64,
    retention_days: u16,
) -> Result<(), std::io::Error> {
    let max_bytes = usize::try_from(max_file_size_bytes).unwrap_or(usize::MAX);
    let mut writer = FileRotate::new(
        path,
        AppendCount::new(retention_days as usize),
        ContentLimit::BytesSurpassed(max_bytes),
        Compression::None,
        None,
    );

    serde_json::to_writer(&mut writer, event).map_err(std::io::Error::other)?;
    writer.write_all(b"\n")?;
    Ok(())
}