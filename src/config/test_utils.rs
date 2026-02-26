use super::Config;

pub fn base_test_config() -> Config {
    Config {
        bot_token: "token".to_string(),
        owner_id: 1,
        monitor_interval: 10,
        command_timeout_secs: 30,
        alerts: Default::default(),
        daily_summary: Default::default(),
        weekly_report: Default::default(),
        graph: Default::default(),
        anomaly_db: Default::default(),
        simulation: Default::default(),
        reporting_store: Default::default(),
        release_notifier: Default::default(),
        security: Default::default(),
    }
}
