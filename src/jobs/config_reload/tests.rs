use std::fs;

use tempfile::tempdir;

use crate::{
    app_context::AppContext,
    capabilities::Capabilities,
    config::{RuntimeConfig, load_config},
};

use super::apply_runtime_reload_from_path;

fn config_toml(monitor_interval: u64, timeout_secs: u64, cpu_threshold: f32) -> String {
    format!(
        r#"bot_token = "123456:abc"
owner_id = 123456789
monitor_interval = {monitor_interval}
command_timeout_secs = {timeout_secs}

[alerts]
cpu = {cpu_threshold}
ram = 90.0
disk = 90.0
cooldown_secs = 300
hysteresis = 5.0

[daily_summary]
enabled = false
hour_utc = 9
minute_utc = 0

[weekly_report]
enabled = false
weekday_utc = 1
hour_utc = 9
minute_utc = 0

[graph]
enabled = true
default_window_minutes = 60
max_window_hours = 24
max_points = 1200

[anomaly_db]
enabled = true
dir = "logs"
max_file_size_bytes = 10485760
retention_days = 7

[simulation]
enabled = false
profile = "sin_spike"

[reporting_store]
enabled = false
path = "data/reporting"
retention_days = 30

[release_notifier]
enabled = false
changelog_path = "CHANGELOG.md"
state_path = "logs/release_notifier.state"
"#
    )
}

#[tokio::test]
async fn hot_reload_applies_valid_runtime_changes_without_restart() {
    let temp = tempdir().expect("tempdir should be created");
    let config_path = temp.path().join("config.toml");
    fs::write(&config_path, config_toml(30, 30, 85.0)).expect("initial config should be written");

    let initial = load_config(&config_path).expect("initial config should load");
    let app = AppContext::new(
        initial,
        2,
        config_path.to_string_lossy().to_string(),
        Capabilities::detect(),
    );

    fs::write(&config_path, config_toml(12, 45, 72.5)).expect("updated config should be written");

    let applied = apply_runtime_reload_from_path(&app, &config_path.to_string_lossy())
        .await
        .expect("valid hot-reload should apply");

    let current = app.runtime_config.read().await.clone();
    assert_eq!(applied.monitor_interval, 12);
    assert_eq!(applied.command_timeout_secs, 45);
    assert!((applied.alerts.cpu - 72.5).abs() < f32::EPSILON);
    assert_eq!(current.monitor_interval, 12);
    assert_eq!(current.command_timeout_secs, 45);
    assert!((current.alerts.cpu - 72.5).abs() < f32::EPSILON);
}

#[tokio::test]
async fn hot_reload_rejects_invalid_config_and_preserves_last_runtime() {
    let temp = tempdir().expect("tempdir should be created");
    let config_path = temp.path().join("config.toml");
    fs::write(&config_path, config_toml(30, 30, 85.0)).expect("initial config should be written");

    let initial = load_config(&config_path).expect("initial config should load");
    let expected_runtime = RuntimeConfig::from_config(&initial);
    let app = AppContext::new(
        initial,
        2,
        config_path.to_string_lossy().to_string(),
        Capabilities::detect(),
    );

    fs::write(&config_path, config_toml(0, 45, 72.5)).expect("invalid config should be written");

    let error = apply_runtime_reload_from_path(&app, &config_path.to_string_lossy())
        .await
        .expect_err("invalid config should be rejected");
    assert!(error.contains("monitor_interval must be greater than 0"));

    let current = app.runtime_config.read().await.clone();
    assert_eq!(current.monitor_interval, expected_runtime.monitor_interval);
    assert_eq!(
        current.command_timeout_secs,
        expected_runtime.command_timeout_secs
    );
    assert!((current.alerts.cpu - expected_runtime.alerts.cpu).abs() < f32::EPSILON);
}
