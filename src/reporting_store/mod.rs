use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use chrono::{Duration as ChronoDuration, Utc};

use crate::{
    config::Config,
    monitor::MetricSample,
};

mod model;
pub use model::RollingMetricSummary;

use model::{DailyRollup, StoredMetricSample};

#[derive(Clone)]
pub struct ReportingStore {
    samples: sled::Tree,
    daily_rollups: sled::Tree,
    sequence: Arc<AtomicU32>,
    retention_days: u16,
}

impl ReportingStore {
    pub fn open_from_config(config: &Config) -> Result<Option<Self>, sled::Error> {
        if !config.reporting_store.enabled {
            return Ok(None);
        }

        let db = sled::open(&config.reporting_store.path)?;
        let samples = db.open_tree("samples")?;
        let daily_rollups = db.open_tree("daily_rollups")?;
        Ok(Some(Self {
            samples,
            daily_rollups,
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
            self.samples.insert(key, value)?;
        }

        self.update_daily_rollup(sample)?;

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

        self.samples
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

    pub fn rolling_summary_days(&self, days: i64) -> Option<RollingMetricSummary> {
        let days = days.max(1);
        let start_day = (Utc::now() - ChronoDuration::days(days - 1))
            .format("%Y-%m-%d")
            .to_string();

        let mut summary = RollingMetricSummary::empty();
        for item in self.daily_rollups.range(start_day.as_bytes()..) {
            let Ok((_, value)) = item else {
                continue;
            };
            let Ok(rollup) = serde_json::from_slice::<DailyRollup>(&value) else {
                continue;
            };
            summary.accumulate_rollup(&rollup);
        }

        if summary.sample_count == 0 {
            return None;
        }

        Some(summary.finalize())
    }

    fn update_daily_rollup(&self, sample: MetricSample) -> Result<(), sled::Error> {
        let day_key = sample.timestamp.format("%Y-%m-%d").to_string();
        let current = self
            .daily_rollups
            .get(day_key.as_bytes())?
            .and_then(|value| serde_json::from_slice::<DailyRollup>(&value).ok());

        let updated = if let Some(mut rollup) = current {
            rollup.update_with_sample(sample);
            rollup
        } else {
            DailyRollup::new(day_key.clone(), sample)
        };

        if let Ok(value) = serde_json::to_vec(&updated) {
            self.daily_rollups.insert(day_key.as_bytes(), value)?;
        }

        Ok(())
    }

    fn prune_old(&self) -> Result<(), sled::Error> {
        let cutoff = Utc::now() - ChronoDuration::days(self.retention_days as i64);
        let cutoff_key = cutoff.timestamp_millis().to_be_bytes();

        let keys_to_remove = self
            .samples
            .iter()
            .keys()
            .filter_map(|key| key.ok())
            .take_while(|key| key.as_ref().len() >= 8 && &key.as_ref()[0..8] < cutoff_key.as_slice())
            .collect::<Vec<_>>();

        for key in keys_to_remove {
            self.samples.remove(key)?;
        }

        let cutoff_day = cutoff.format("%Y-%m-%d").to_string();
        let rollups_to_remove = self
            .daily_rollups
            .iter()
            .keys()
            .filter_map(|key| key.ok())
            .filter_map(|key| String::from_utf8(key.to_vec()).ok())
            .take_while(|day| day < &cutoff_day)
            .collect::<Vec<_>>();

        for day in rollups_to_remove {
            self.daily_rollups.remove(day.as_bytes())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
