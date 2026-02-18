use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::config::Config;

use super::model::{AnomalyEvent, AnomalyIndexEntry};
use super::paths::paths_from_config;

pub fn recent_anomalies(config: &Config, limit: usize) -> Vec<AnomalyEvent> {
    if !config.anomaly_db.enabled || limit == 0 {
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
        let left_modified = left.metadata().and_then(|meta| meta.modified()).ok();
        let right_modified = right.metadata().and_then(|meta| meta.modified()).ok();

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
