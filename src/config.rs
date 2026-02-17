use serde::Deserialize;
use std::fs;
use teloxide::types::{ChatId, UserId};
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot_token: String,
    pub owner_id: i64,
    pub alerts: Alerts,
    pub monitor_interval: u64,
    #[serde(default = "default_command_timeout_secs")]
    pub command_timeout_secs: u64,
    #[serde(default)]
    pub daily_summary: DailySummary,
    #[serde(default)]
    pub anomaly_journal: AnomalyJournal,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Alerts {
    pub cpu: f32,
    pub ram: f32,
    pub disk: f32,
    #[serde(default = "default_alert_cooldown_secs")]
    pub cooldown_secs: u64,
    #[serde(default = "default_alert_hysteresis")]
    pub hysteresis: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DailySummary {
    #[serde(default = "default_daily_summary_enabled")]
    pub enabled: bool,
    #[serde(default = "default_daily_summary_hour")]
    pub hour_utc: u8,
    #[serde(default = "default_daily_summary_minute")]
    pub minute_utc: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnomalyJournal {
    #[serde(default = "default_anomaly_journal_enabled")]
    pub enabled: bool,
    #[serde(default = "default_anomaly_journal_dir")]
    pub dir: String,
    #[serde(default = "default_anomaly_journal_max_file_size_bytes")]
    pub max_file_size_bytes: u64,
    #[serde(default = "default_anomaly_journal_retention_days")]
    pub retention_days: u16,
}

impl Default for DailySummary {
    fn default() -> Self {
        Self {
            enabled: default_daily_summary_enabled(),
            hour_utc: default_daily_summary_hour(),
            minute_utc: default_daily_summary_minute(),
        }
    }
}

impl Default for AnomalyJournal {
    fn default() -> Self {
        Self {
            enabled: default_anomaly_journal_enabled(),
            dir: default_anomaly_journal_dir(),
            max_file_size_bytes: default_anomaly_journal_max_file_size_bytes(),
            retention_days: default_anomaly_journal_retention_days(),
        }
    }
}

fn default_command_timeout_secs() -> u64 {
    30
}

fn default_alert_cooldown_secs() -> u64 {
    300
}

fn default_alert_hysteresis() -> f32 {
    3.0
}

fn default_daily_summary_enabled() -> bool {
    true
}

fn default_daily_summary_hour() -> u8 {
    9
}

fn default_daily_summary_minute() -> u8 {
    0
}

fn default_anomaly_journal_enabled() -> bool {
    true
}

fn default_anomaly_journal_dir() -> String {
    "logs".to_string()
}

fn default_anomaly_journal_max_file_size_bytes() -> u64 {
    10 * 1024 * 1024
}

fn default_anomaly_journal_retention_days() -> u16 {
    7
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("invalid config format: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid config value: {0}")]
    Validation(String),
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.monitor_interval < 10 {
            return Err(ConfigError::Validation(
                "monitor_interval must be at least 10 seconds".to_string(),
            ));
        }

        if self.owner_id <= 0 {
            return Err(ConfigError::Validation(
                "owner_id must be a positive Telegram user id".to_string(),
            ));
        }

        if self.command_timeout_secs == 0 {
            return Err(ConfigError::Validation(
                "command_timeout_secs must be greater than 0".to_string(),
            ));
        }

        if self.alerts.cooldown_secs == 0 {
            return Err(ConfigError::Validation(
                "alerts.cooldown_secs must be greater than 0".to_string(),
            ));
        }

        for (name, value) in [
            ("alerts.cpu", self.alerts.cpu),
            ("alerts.ram", self.alerts.ram),
            ("alerts.disk", self.alerts.disk),
        ] {
            if !(0.0..=100.0).contains(&value) {
                return Err(ConfigError::Validation(format!(
                    "{} must be between 0 and 100",
                    name
                )));
            }
        }

        if self.alerts.hysteresis < 0.0 || self.alerts.hysteresis > 100.0 {
            return Err(ConfigError::Validation(
                "alerts.hysteresis must be between 0 and 100".to_string(),
            ));
        }

        if self.daily_summary.hour_utc > 23 {
            return Err(ConfigError::Validation(
                "daily_summary.hour_utc must be between 0 and 23".to_string(),
            ));
        }

        if self.daily_summary.minute_utc > 59 {
            return Err(ConfigError::Validation(
                "daily_summary.minute_utc must be between 0 and 59".to_string(),
            ));
        }

        if self.anomaly_journal.enabled {
            if self.anomaly_journal.dir.trim().is_empty() {
                return Err(ConfigError::Validation(
                    "anomaly_journal.dir cannot be empty when enabled".to_string(),
                ));
            }

            if self.anomaly_journal.max_file_size_bytes == 0 {
                return Err(ConfigError::Validation(
                    "anomaly_journal.max_file_size_bytes must be greater than 0".to_string(),
                ));
            }

            if self.anomaly_journal.retention_days == 0 {
                return Err(ConfigError::Validation(
                    "anomaly_journal.retention_days must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn owner_chat_id(&self) -> Result<ChatId, ConfigError> {
        Ok(ChatId(self.owner_id))
    }

    pub fn owner_user_id(&self) -> Result<UserId, ConfigError> {
        let owner_user_id = u64::try_from(self.owner_id).map_err(|_| {
            ConfigError::Validation("owner_id must fit into Telegram UserId (u64)".to_string())
        })?;

        Ok(UserId(owner_user_id))
    }
}

pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;
    Ok(config)
}