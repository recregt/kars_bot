use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    monitor::MetricSample,
};

#[derive(Clone)]
pub struct ReportingStore {
    db: sled::Db,
    sequence: Arc<AtomicU32>,
    retention_days: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredMetricSample {
    timestamp_utc: String,
    cpu: f32,
    ram: f32,
    disk: f32,
}

impl ReportingStore {
    pub fn open_from_config(config: &Config) -> Result<Option<Self>, sled::Error> {
        if !config.reporting_store.enabled {
            return Ok(None);
        }

        let db = sled::open(&config.reporting_store.path)?;
        Ok(Some(Self {
            db,
            sequence: Arc::new(AtomicU32::new(0)),
            retention_days: config.reporting_store.retention_days,
        }))
    }

    pub fn record_sample(&self, sample: MetricSample) -> Result<(), sled::Error> {
        let mut key = Vec::with_capacity(12);
        key.extend_from_slice(&sample.timestamp.timestamp_millis().to_be_bytes());
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        key.extend_from_slice(&seq.to_be_bytes());

        let payload = StoredMetricSample {
            timestamp_utc: sample.timestamp.to_rfc3339(),
            cpu: sample.cpu,
            ram: sample.ram,
            disk: sample.disk,
        };

        if let Ok(value) = serde_json::to_vec(&payload) {
            self.db.insert(key, value)?;
        }

        if seq.is_multiple_of(120) {
            self.prune_old()?;
        }

        Ok(())
    }

    pub fn latest_window(&self, minutes: i64) -> Vec<MetricSample> {
        let now = Utc::now();
        let cutoff = now - ChronoDuration::minutes(minutes.max(1));

        let mut start_key = Vec::with_capacity(12);
        start_key.extend_from_slice(&cutoff.timestamp_millis().to_be_bytes());
        start_key.extend_from_slice(&0u32.to_be_bytes());

        self.db
            .range(start_key..)
            .filter_map(|item| item.ok())
            .filter_map(|(_, value)| serde_json::from_slice::<StoredMetricSample>(&value).ok())
            .filter_map(|item| {
                chrono::DateTime::parse_from_rfc3339(&item.timestamp_utc)
                    .ok()
                    .map(|ts| MetricSample {
                        timestamp: ts.with_timezone(&Utc),
                        cpu: item.cpu,
                        ram: item.ram,
                        disk: item.disk,
                    })
            })
            .collect()
    }

    fn prune_old(&self) -> Result<(), sled::Error> {
        let cutoff = Utc::now() - ChronoDuration::days(self.retention_days as i64);
        let cutoff_key = cutoff.timestamp_millis().to_be_bytes();

        let keys_to_remove = self
            .db
            .iter()
            .keys()
            .filter_map(|key| key.ok())
            .take_while(|key| key.as_ref().len() >= 8 && &key.as_ref()[0..8] < cutoff_key.as_slice())
            .collect::<Vec<_>>();

        for key in keys_to_remove {
            self.db.remove(key)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use crate::monitor::MetricSample;

    use super::ReportingStore;

    #[test]
    fn records_and_reads_recent_samples() {
        let temp = tempfile::tempdir().expect("temp dir");
        let db = sled::open(temp.path()).expect("open db");
        let store = ReportingStore {
            db,
            sequence: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            retention_days: 7,
        };

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
}
