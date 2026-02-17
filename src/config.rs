use serde::Deserialize;
use std::fs;
use teloxide::types::ChatId;
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot_token: String,
    pub owner_id: u64,
    #[serde(alias = "authorized_users")]
    pub allowed_user_ids: Vec<u64>,
    #[serde(default)]
    pub allowed_chat_ids: Option<Vec<i64>>,
    pub alerts: Alerts,
    pub monitor_interval: u64,
    #[serde(default = "default_command_timeout_secs")]
    pub command_timeout_secs: u64,
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

fn default_command_timeout_secs() -> u64 {
    30
}

fn default_alert_cooldown_secs() -> u64 {
    300
}

fn default_alert_hysteresis() -> f32 {
    3.0
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

        if self.allowed_user_ids.is_empty() {
            return Err(ConfigError::Validation(
                "allowed_user_ids cannot be empty".to_string(),
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

        if self.owner_id > i64::MAX as u64 {
            return Err(ConfigError::Validation(
                "owner_id exceeds Telegram ChatId range (i64::MAX)".to_string(),
            ));
        }

        Ok(())
    }

    pub fn owner_chat_id(&self) -> Result<ChatId, ConfigError> {
        let owner_id = i64::try_from(self.owner_id).map_err(|_| {
            ConfigError::Validation(
                "owner_id exceeds Telegram ChatId range (i64::MAX)".to_string(),
            )
        })?;

        Ok(ChatId(owner_id))
    }
}

pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;
    Ok(config)
}