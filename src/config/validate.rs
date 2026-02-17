use thiserror::Error;
use teloxide::types::{ChatId, UserId};

use super::schema::Config;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error("invalid config: {0}")]
    Validation(String),
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.bot_token.trim().is_empty() {
            return Err(ConfigError::Validation(
                "bot_token must not be empty".to_string(),
            ));
        }
        if self.owner_id == 0 {
            return Err(ConfigError::Validation(
                "owner_id must be a positive integer".to_string(),
            ));
        }
        if self.monitor_interval == 0 {
            return Err(ConfigError::Validation(
                "monitor_interval must be greater than 0".to_string(),
            ));
        }
        if self.command_timeout_secs == 0 {
            return Err(ConfigError::Validation(
                "command_timeout_secs must be greater than 0".to_string(),
            ));
        }
        validate_percentage("alerts.cpu", self.alerts.cpu)?;
        validate_percentage("alerts.ram", self.alerts.ram)?;
        validate_percentage("alerts.disk", self.alerts.disk)?;
        if self.alerts.cooldown_secs == 0 {
            return Err(ConfigError::Validation(
                "alerts.cooldown_secs must be greater than 0".to_string(),
            ));
        }
        if self.alerts.hysteresis.is_sign_negative() {
            return Err(ConfigError::Validation(
                "alerts.hysteresis must be non-negative".to_string(),
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
        if !(1..=7).contains(&self.weekly_report.weekday_utc) {
            return Err(ConfigError::Validation(
                "weekly_report.weekday_utc must be between 1 and 7".to_string(),
            ));
        }
        if self.weekly_report.hour_utc > 23 {
            return Err(ConfigError::Validation(
                "weekly_report.hour_utc must be between 0 and 23".to_string(),
            ));
        }
        if self.weekly_report.minute_utc > 59 {
            return Err(ConfigError::Validation(
                "weekly_report.minute_utc must be between 0 and 59".to_string(),
            ));
        }
        if self.graph.default_window_minutes == 0 {
            return Err(ConfigError::Validation(
                "graph.default_window_minutes must be greater than 0".to_string(),
            ));
        }
        if self.graph.max_window_hours == 0 {
            return Err(ConfigError::Validation(
                "graph.max_window_hours must be greater than 0".to_string(),
            ));
        }
        if self.graph.max_points < 10 {
            return Err(ConfigError::Validation(
                "graph.max_points must be at least 10".to_string(),
            ));
        }

        if self.anomaly_db.enabled && self.anomaly_db.dir.trim().is_empty() {
            return Err(ConfigError::Validation(
                "anomaly_db.dir must not be empty when anomaly_db.enabled is true".to_string(),
            ));
        }
        if self.anomaly_db.max_file_size_bytes == 0 {
            return Err(ConfigError::Validation(
                "anomaly_db.max_file_size_bytes must be greater than 0".to_string(),
            ));
        }
        if self.anomaly_db.retention_days == 0 {
            return Err(ConfigError::Validation(
                "anomaly_db.retention_days must be greater than 0".to_string(),
            ));
        }

        if self.simulation.profile.trim().is_empty() {
            return Err(ConfigError::Validation(
                "simulation.profile must not be empty".to_string(),
            ));
        }

        if self.reporting_store.enabled && self.reporting_store.path.trim().is_empty() {
            return Err(ConfigError::Validation(
                "reporting_store.path must not be empty when reporting_store.enabled is true"
                    .to_string(),
            ));
        }
        if self.reporting_store.retention_days == 0 {
            return Err(ConfigError::Validation(
                "reporting_store.retention_days must be greater than 0".to_string(),
            ));
        }
        if self.release_notifier.enabled
            && self.release_notifier.changelog_path.trim().is_empty()
        {
            return Err(ConfigError::Validation(
                "release_notifier.changelog_path must not be empty when release_notifier.enabled is true"
                    .to_string(),
            ));
        }
        if self.release_notifier.enabled && self.release_notifier.state_path.trim().is_empty() {
            return Err(ConfigError::Validation(
                "release_notifier.state_path must not be empty when release_notifier.enabled is true"
                    .to_string(),
            ));
        }
        Ok(())
    }

    pub fn owner_chat_id(&self) -> Result<ChatId, ConfigError> {
        if self.owner_id == 0 {
            return Err(ConfigError::Validation(
                "owner_id must be a positive integer".to_string(),
            ));
        }

        let chat_id = i64::try_from(self.owner_id).map_err(|_| {
            ConfigError::Validation("owner_id is too large to fit Telegram chat id".to_string())
        })?;
        Ok(ChatId(chat_id))
    }

    pub fn owner_user_id(&self) -> Result<UserId, ConfigError> {
        if self.owner_id == 0 {
            return Err(ConfigError::Validation(
                "owner_id must be a positive integer".to_string(),
            ));
        }

        Ok(UserId(self.owner_id))
    }
}

fn validate_percentage(field: &str, value: f32) -> Result<(), ConfigError> {
    if value.is_nan() || !(0.0..=100.0).contains(&value) {
        return Err(ConfigError::Validation(format!(
            "{} must be between 0 and 100",
            field
        )));
    }
    Ok(())
}
