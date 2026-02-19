use std::sync::{Arc, atomic::AtomicU32};

use chrono::{Duration, Utc};

use crate::monitor::MetricSample;

use super::ReportingStore;

fn open_test_store(path: &std::path::Path) -> ReportingStore {
    let db = sled::open(path).expect("open db");
    let samples = db.open_tree("samples").expect("open samples tree");
    let daily_rollups = db
        .open_tree("daily_rollups")
        .expect("open daily rollups tree");

    ReportingStore {
        samples,
        daily_rollups,
        sequence: Arc::new(AtomicU32::new(0)),
        retention_days: 7,
        db_path: path.to_string_lossy().to_string(),
    }
}

#[test]
fn records_and_reads_recent_samples() {
    let temp = tempfile::tempdir().expect("temp dir");
    let store = open_test_store(temp.path());

    let now = Utc::now();
    store
        .record_sample(MetricSample {
            timestamp: now - Duration::minutes(5),
            cpu: 10.0,
            ram: 20.0,
            disk: 30.0,
        })
        .expect("record old sample");

    store
        .record_sample(MetricSample {
            timestamp: now,
            cpu: 90.0,
            ram: 80.0,
            disk: 70.0,
        })
        .expect("record latest sample");

    let recent = store.latest_window(10);
    assert!(!recent.is_empty());
    assert!(recent.iter().any(|sample| sample.cpu >= 90.0));
}

#[test]
fn rolling_summary_aggregates_persisted_days() {
    let temp = tempfile::tempdir().expect("temp dir");
    let store = open_test_store(temp.path());

    let now = Utc::now();
    store
        .record_sample(MetricSample {
            timestamp: now - Duration::days(1),
            cpu: 40.0,
            ram: 50.0,
            disk: 60.0,
        })
        .expect("record day-1 sample");
    store
        .record_sample(MetricSample {
            timestamp: now,
            cpu: 80.0,
            ram: 70.0,
            disk: 90.0,
        })
        .expect("record day-0 sample");

    let summary = store
        .rolling_summary_days(7)
        .expect("rolling summary should exist");
    assert_eq!(summary.sample_count, 2);
    assert!(summary.cpu_avg > 59.0 && summary.cpu_avg < 61.0);
    assert_eq!(summary.cpu_min, 40.0);
    assert_eq!(summary.cpu_max, 80.0);
}

#[test]
fn rolling_summary_survives_store_reopen() {
    let temp = tempfile::tempdir().expect("temp dir");

    let store = open_test_store(temp.path());
    store
        .record_sample(MetricSample {
            timestamp: Utc::now(),
            cpu: 55.0,
            ram: 45.0,
            disk: 35.0,
        })
        .expect("record sample before restart");
    drop(store);
    std::thread::sleep(std::time::Duration::from_millis(25));

    let reopened = open_test_store(temp.path());
    let summary = reopened
        .rolling_summary_days(7)
        .expect("summary should remain after reopen");
    assert!(summary.sample_count >= 1);
    assert!(summary.cpu_avg >= 55.0);
}
