use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnomalyEvent {
    pub timestamp: String,
    pub cpu: f32,
    pub ram: f32,
    pub disk: f32,
    pub cpu_threshold: f32,
    pub ram_threshold: f32,
    pub disk_threshold: f32,
    pub cpu_over: bool,
    pub ram_over: bool,
    pub disk_over: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AnomalyIndexEntry {
    timestamp: String,
    cpu: f32,
    ram: f32,
    disk: f32,
    cpu_threshold: f32,
    ram_threshold: f32,
    disk_threshold: f32,
    cpu_over: bool,
    ram_over: bool,
    disk_over: bool,
}

#[derive(Debug, Clone)]
struct JournalPaths {
    events_dir: PathBuf,
    index_dir: PathBuf,
    _meta_dir: PathBuf,
}

pub fn record_anomaly_if_needed(config: &Config, cpu: f32, ram: f32, disk: f32) {
    if !config.anomaly_journal.enabled {
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
    if let Err(error) = ensure_journal_dirs(&paths) {
        log::warn!("anomaly journal: failed to create directory: {}", error);
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
        config.anomaly_journal.max_file_size_bytes,
        config.anomaly_journal.retention_days,
    ) {
        log::warn!("anomaly journal: failed to write event line: {}", error);
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
        log::warn!("anomaly journal: failed to write index line: {}", error);
    }
}

pub fn recent_anomalies(config: &Config, limit: usize) -> Vec<AnomalyEvent> {
    if !config.anomaly_journal.enabled || limit == 0 {
        return Vec::new();
    }

    let paths = paths_from_config(config);
    let files = newest_index_files(&paths.index_dir);
    if files.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(limit);
    for file_path in files {
        let remaining = limit.saturating_sub(out.len());
        if remaining == 0 {
            break;
        }

        let lines = match read_tail_lines(&file_path, remaining) {
            Ok(lines) => lines,
            Err(_) => continue,
        };

        for line in lines.into_iter().rev() {
            let Ok(index_entry) = serde_json::from_str::<AnomalyIndexEntry>(&line) else {
                continue;
            };

            out.push(AnomalyEvent {
                timestamp: index_entry.timestamp,
                cpu: index_entry.cpu,
                ram: index_entry.ram,
                disk: index_entry.disk,
                cpu_threshold: index_entry.cpu_threshold,
                ram_threshold: index_entry.ram_threshold,
                disk_threshold: index_entry.disk_threshold,
                cpu_over: index_entry.cpu_over,
                ram_over: index_entry.ram_over,
                disk_over: index_entry.disk_over,
            });

            if out.len() >= limit {
                break;
            }
        }
    }

    out
}

pub fn run_maintenance(config: &Config) {
    if !config.anomaly_journal.enabled {
        return;
    }

    let paths = paths_from_config(config);
    if let Err(error) = ensure_journal_dirs(&paths) {
        log::warn!("anomaly journal maintenance: failed to ensure dirs: {}", error);
        return;
    }

    prune_old_daily_files(&paths, config.anomaly_journal.retention_days);
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

fn paths_from_config(config: &Config) -> JournalPaths {
    let root = PathBuf::from(&config.anomaly_journal.dir);
    JournalPaths {
        events_dir: root.join("events"),
        index_dir: root.join("index"),
        _meta_dir: root.join("meta"),
    }
}

fn ensure_journal_dirs(paths: &JournalPaths) -> Result<(), std::io::Error> {
    fs::create_dir_all(&paths.events_dir)?;
    fs::create_dir_all(&paths.index_dir)?;
    fs::create_dir_all(&paths._meta_dir)?;
    Ok(())
}

fn prune_old_daily_files(paths: &JournalPaths, retention_days: u16) {
    let removed_events_days = prune_directory_by_date_prefix(
        &paths.events_dir,
        "events-",
        retention_days,
    );

    let mut removed_index_days = prune_directory_by_date_prefix(
        &paths.index_dir,
        "index-",
        retention_days,
    );

    for day in removed_events_days {
        if removed_index_days.contains(&day) {
            continue;
        }

        remove_matching_date_files(&paths.index_dir, "index-", &day);
        removed_index_days.insert(day);
    }
}

fn prune_directory_by_date_prefix(dir: &Path, prefix: &str, retention_days: u16) -> HashSet<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return HashSet::new();
    };

    let today = Utc::now().date_naive();
    let keep_for = ChronoDuration::days(retention_days as i64);
    let mut removed_days = HashSet::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !file_name.starts_with(prefix) || !file_name.contains(".jsonl") {
            continue;
        }

        let date_part = file_name.strip_prefix(prefix).and_then(|tail| tail.get(0..10));
        let Some(date_part) = date_part else {
            continue;
        };

        let Ok(file_date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") else {
            continue;
        };

        if today.signed_duration_since(file_date) > keep_for {
            if let Err(error) = fs::remove_file(&path) {
                log::warn!(
                    "anomaly journal: failed to remove old file {}: {}",
                    path.display(),
                    error
                );
            } else {
                removed_days.insert(date_part.to_string());
            }
        }
    }

    removed_days
}

fn remove_matching_date_files(dir: &Path, prefix: &str, day: &str) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let expected_prefix = format!("{}{}", prefix, day);
        if !name.starts_with(&expected_prefix) || !name.contains(".jsonl") {
            continue;
        }

        if let Err(error) = fs::remove_file(&path) {
            log::warn!(
                "anomaly journal: failed to remove synchronized file {}: {}",
                path.display(),
                error
            );
        }
    }
}

fn newest_index_files(index_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(index_dir) else {
        return Vec::new();
    };

    let mut files = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("index-") && name.contains(".jsonl"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    files.sort_by(|left, right| {
        let left_modified = left
            .metadata()
            .and_then(|meta| meta.modified())
            .ok();
        let right_modified = right
            .metadata()
            .and_then(|meta| meta.modified())
            .ok();

        right_modified.cmp(&left_modified)
    });

    files
}

fn read_tail_lines(path: &Path, max_lines: usize) -> Result<Vec<String>, std::io::Error> {
    let mut file = File::open(path)?;
    let file_len = file.seek(SeekFrom::End(0))?;
    if file_len == 0 || max_lines == 0 {
        return Ok(Vec::new());
    }

    const CHUNK_SIZE: usize = 4096;
    let mut pos = file_len;
    let mut bytes = Vec::new();
    let mut newline_count = 0usize;

    while pos > 0 && newline_count <= max_lines {
        let read_size = CHUNK_SIZE.min(pos as usize);
        pos -= read_size as u64;

        file.seek(SeekFrom::Start(pos))?;
        let mut chunk = vec![0u8; read_size];
        file.read_exact(&mut chunk)?;

        newline_count += chunk.iter().filter(|&&byte| byte == b'\n').count();

        chunk.extend_from_slice(&bytes);
        bytes = chunk;
    }

    let mut lines = String::from_utf8_lossy(&bytes)
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    if lines.len() > max_lines {
        lines.drain(0..(lines.len() - max_lines));
    }

    Ok(lines)
}