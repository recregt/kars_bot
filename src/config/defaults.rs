use super::schema::{
    Alerts, AnomalyDb, DailySummary, Graph, ReleaseNotifierConfig, ReportingStoreConfig,
    Simulation, WeeklyReport,
};

pub(super) fn default_monitor_interval() -> u64 {
    30
}

pub(super) fn default_command_timeout_secs() -> u64 {
    30
}

pub(super) fn default_alert_cpu() -> f32 {
    85.0
}

pub(super) fn default_alert_ram() -> f32 {
    90.0
}

pub(super) fn default_alert_disk() -> f32 {
    90.0
}

pub(super) fn default_cooldown_secs() -> u64 {
    300
}

pub(super) fn default_hysteresis() -> f32 {
    5.0
}

pub(super) fn default_graph_window_minutes() -> u64 {
    60
}

pub(super) fn default_graph_enabled() -> bool {
    true
}

pub(super) fn default_graph_max_window_hours() -> u64 {
    24
}

pub(super) fn default_graph_max_points() -> u16 {
    1200
}

pub(super) fn default_weekly_report_enabled() -> bool {
    false
}

pub(super) fn default_weekly_report_weekday() -> u8 {
    1
}

pub(super) fn default_weekly_report_hour() -> u8 {
    9
}

pub(super) fn default_weekly_report_minute() -> u8 {
    0
}

pub(super) fn default_daily_summary_hour() -> u8 {
    9
}

pub(super) fn default_daily_summary_minute() -> u8 {
    0
}

pub(super) fn default_daily_summary_enabled() -> bool {
    true
}

pub(super) fn default_anomaly_db_enabled() -> bool {
    true
}

pub(super) fn default_anomaly_db_dir() -> String {
    "logs".to_string()
}

pub(super) fn default_anomaly_db_max_file_size_bytes() -> u64 {
    10 * 1024 * 1024
}

pub(super) fn default_anomaly_db_retention_days() -> u16 {
    7
}

pub(super) fn default_simulation_profile() -> String {
    "wave".to_string()
}

pub(super) fn default_reporting_store_path() -> String {
    "data/reporting_store".to_string()
}

pub(super) fn default_reporting_store_retention_days() -> u16 {
    30
}

pub(super) fn default_release_notifier_changelog_path() -> String {
    "CHANGELOG.md".to_string()
}

pub(super) fn default_release_notifier_state_path() -> String {
    "data/release_notifier/state.json".to_string()
}

impl Default for Alerts {
    fn default() -> Self {
        Self {
            cpu: default_alert_cpu(),
            ram: default_alert_ram(),
            disk: default_alert_disk(),
            cooldown_secs: default_cooldown_secs(),
            hysteresis: default_hysteresis(),
        }
    }
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

impl Default for WeeklyReport {
    fn default() -> Self {
        Self {
            enabled: default_weekly_report_enabled(),
            weekday_utc: default_weekly_report_weekday(),
            hour_utc: default_weekly_report_hour(),
            minute_utc: default_weekly_report_minute(),
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            enabled: default_graph_enabled(),
            default_window_minutes: default_graph_window_minutes(),
            max_window_hours: default_graph_max_window_hours(),
            max_points: default_graph_max_points(),
        }
    }
}

impl Default for AnomalyDb {
    fn default() -> Self {
        Self {
            enabled: default_anomaly_db_enabled(),
            dir: default_anomaly_db_dir(),
            max_file_size_bytes: default_anomaly_db_max_file_size_bytes(),
            retention_days: default_anomaly_db_retention_days(),
        }
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            enabled: false,
            profile: default_simulation_profile(),
        }
    }
}

impl Default for ReportingStoreConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: default_reporting_store_path(),
            retention_days: default_reporting_store_retention_days(),
        }
    }
}

impl Default for ReleaseNotifierConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            changelog_path: default_release_notifier_changelog_path(),
            state_path: default_release_notifier_state_path(),
        }
    }
}
