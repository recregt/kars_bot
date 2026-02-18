use std::collections::HashSet;
use std::fs;
use std::path::Path;

use chrono::{Duration as ChronoDuration, NaiveDate, Utc};

use crate::config::Config;

use super::paths::{DbPaths, ensure_db_dirs, paths_from_config};

pub fn run_maintenance(config: &Config) {
    if !config.anomaly_db.enabled {
        return;
    }

    let paths = paths_from_config(config);
    if let Err(error) = ensure_db_dirs(&paths) {
        log::warn!("anomaly db maintenance: failed to ensure dirs: {}", error);
        return;
    }

    prune_old_daily_files(&paths, config.anomaly_db.retention_days);
}

fn prune_old_daily_files(paths: &DbPaths, retention_days: u16) {
    let removed_events_days =
        prune_directory_by_date_prefix(&paths.events_dir, "events-", retention_days);

    let mut removed_index_days =
        prune_directory_by_date_prefix(&paths.index_dir, "index-", retention_days);

    for day in removed_events_days {
        if removed_index_days.contains(&day) {
            continue;
        }

        remove_matching_date_files(&paths.index_dir, "index-", &day);
        removed_index_days.insert(day);
    }
}

fn prune_directory_by_date_prefix(
    dir: &Path,
    prefix: &str,
    retention_days: u16,
) -> HashSet<String> {
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

        let date_part = file_name
            .strip_prefix(prefix)
            .and_then(|tail| tail.get(0..10));
        let Some(date_part) = date_part else {
            continue;
        };

        let Ok(file_date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") else {
            continue;
        };

        if today.signed_duration_since(file_date) > keep_for {
            if let Err(error) = fs::remove_file(&path) {
                log::warn!(
                    "anomaly db: failed to remove old file {}: {}",
                    path.display(),
                    error
                );
            } else {
                log::info!(
                    "anomaly db maintenance: removed old file {}",
                    path.display()
                );
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
                "anomaly db: failed to remove synchronized file {}: {}",
                path.display(),
                error
            );
        } else {
            log::info!(
                "anomaly db maintenance: removed synchronized file {}",
                path.display()
            );
        }
    }
}
