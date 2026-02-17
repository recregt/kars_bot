use serde::Deserialize;

use super::defaults::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub owner_id: u64,
    #[serde(default = "default_monitor_interval")]
    pub monitor_interval: u64,
    #[serde(default = "default_command_timeout_secs")]
    pub command_timeout_secs: u64,
    #[serde(default)]
    pub alerts: Alerts,
    #[serde(default)]
    pub daily_summary: DailySummary,
    #[serde(default)]
    pub weekly_report: WeeklyReport,
    #[serde(default)]
    pub graph: Graph,
    #[serde(default, alias = "anomaly_journal")]
    pub anomaly_db: AnomalyDb,
    #[serde(default)]
    pub simulation: Simulation,
    #[serde(default)]
    pub reporting_store: ReportingStoreConfig,
    #[serde(default)]
    pub release_notifier: ReleaseNotifierConfig,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub alerts: Alerts,
    pub monitor_interval: u64,
    pub command_timeout_secs: u64,
    pub graph: Graph,
}

impl RuntimeConfig {
    pub fn from_config(config: &Config) -> Self {
        Self {
            alerts: config.alerts.clone(),
            monitor_interval: config.monitor_interval,
            command_timeout_secs: config.command_timeout_secs,
            graph: config.graph.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Alerts {
    #[serde(default = "default_alert_cpu")]
    pub cpu: f32,
    #[serde(default = "default_alert_ram")]
    pub ram: f32,
    #[serde(default = "default_alert_disk")]
    pub disk: f32,
    #[serde(default = "default_cooldown_secs")]
    pub cooldown_secs: u64,
    #[serde(default = "default_hysteresis")]
    pub hysteresis: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailySummary {
    #[serde(default = "default_daily_summary_enabled")]
    pub enabled: bool,
    #[serde(default = "default_daily_summary_hour")]
    pub hour_utc: u8,
    #[serde(default = "default_daily_summary_minute")]
    pub minute_utc: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeeklyReport {
    #[serde(default = "default_weekly_report_enabled")]
    pub enabled: bool,
    #[serde(default = "default_weekly_report_weekday")]
    pub weekday_utc: u8,
    #[serde(default = "default_weekly_report_hour")]
    pub hour_utc: u8,
    #[serde(default = "default_weekly_report_minute")]
    pub minute_utc: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Graph {
    #[serde(default = "default_graph_enabled")]
    pub enabled: bool,
    #[serde(default = "default_graph_window_minutes")]
    pub default_window_minutes: u64,
    #[serde(default = "default_graph_max_window_hours")]
    pub max_window_hours: u64,
    #[serde(default = "default_graph_max_points")]
    pub max_points: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnomalyDb {
    #[serde(default = "default_anomaly_db_enabled")]
    pub enabled: bool,
    #[serde(default = "default_anomaly_db_dir")]
    pub dir: String,
    #[serde(default = "default_anomaly_db_max_file_size_bytes")]
    pub max_file_size_bytes: u64,
    #[serde(default = "default_anomaly_db_retention_days")]
    pub retention_days: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Simulation {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_simulation_profile")]
    pub profile: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportingStoreConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_reporting_store_path")]
    pub path: String,
    #[serde(default = "default_reporting_store_retention_days")]
    pub retention_days: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseNotifierConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_release_notifier_changelog_path")]
    pub changelog_path: String,
    #[serde(default = "default_release_notifier_state_path")]
    pub state_path: String,
}
