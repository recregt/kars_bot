use std::{sync::Arc, time::Instant};

use tokio::sync::Mutex;

use crate::config::Config;

use super::{provider::Metrics, state::AlertState};

pub(super) async fn evaluate_alerts_at(
    config: &Config,
    state: &Arc<Mutex<AlertState>>,
    metrics: Metrics,
    now: Instant,
) -> Vec<String> {
    let mut notifications = Vec::new();

    {
        let mut alert_state = state.lock().await;
        if alert_state.cpu_should_alert(
            metrics.cpu,
            config.alerts.cpu,
            config.alerts.cooldown_secs,
            config.alerts.hysteresis,
            now,
        ) {
            notifications.push(format!("⚠️ ALERT: CPU usage is high ({:.1}%)", metrics.cpu));
        }

        if alert_state.ram_should_alert(
            metrics.ram,
            config.alerts.ram,
            config.alerts.cooldown_secs,
            config.alerts.hysteresis,
            now,
        ) {
            notifications.push(format!("⚠️ ALERT: RAM usage is high ({:.1}%)", metrics.ram));
        }

        if alert_state.disk_should_alert(
            metrics.disk,
            config.alerts.disk,
            config.alerts.cooldown_secs,
            config.alerts.hysteresis,
            now,
        ) {
            notifications.push(format!("⚠️ ALERT: Disk usage is high ({:.1}%)", metrics.disk));
        }
    }

    notifications
}

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    use tokio::sync::Mutex;

    use crate::config::{
        Alerts, Config, Graph, ReleaseNotifierConfig, ReportingStoreConfig, Simulation,
        WeeklyReport,
    };

    use super::{evaluate_alerts_at, AlertState, Metrics};
    use crate::monitor::provider::{MetricsProvider, MockMetricsProvider, SimulatedMetricsProvider};

    fn test_config() -> Config {
        Config {
            bot_token: "token".to_string(),
            owner_id: 1,
            alerts: Alerts {
                cpu: 80.0,
                ram: 80.0,
                disk: 80.0,
                cooldown_secs: 300,
                hysteresis: 5.0,
            },
            monitor_interval: 10,
            command_timeout_secs: 30,
            daily_summary: Default::default(),
            weekly_report: WeeklyReport::default(),
            graph: Graph::default(),
            anomaly_db: Default::default(),
            simulation: Simulation::default(),
            reporting_store: ReportingStoreConfig::default(),
            release_notifier: ReleaseNotifierConfig::default(),
        }
    }

    #[tokio::test]
    async fn mock_provider_returns_sequence() {
        let mut provider = MockMetricsProvider::new(vec![Metrics::new(81.0, 10.0, 10.0)]);
        let metrics = provider
            .collect_metrics()
            .await
            .expect("mock should return metrics");
        assert!(metrics.cpu > 80.0);
    }

    #[tokio::test]
    async fn cooldown_and_hysteresis_work() {
        let config = test_config();
        let state = Arc::new(Mutex::new(AlertState::default()));
        let start = Instant::now();

        let first = evaluate_alerts_at(&config, &state, Metrics::new(90.0, 10.0, 10.0), start).await;
        assert_eq!(first.len(), 1);

        let cooldown_block = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(92.0, 10.0, 10.0),
            start + Duration::from_secs(60),
        )
        .await;
        assert_eq!(cooldown_block.len(), 0);

        let after_cooldown = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(93.0, 10.0, 10.0),
            start + Duration::from_secs(301),
        )
        .await;
        assert_eq!(after_cooldown.len(), 1);

        let clear = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(74.0, 10.0, 10.0),
            start + Duration::from_secs(320),
        )
        .await;
        assert_eq!(clear.len(), 0);

        let retrigger = evaluate_alerts_at(
            &config,
            &state,
            Metrics::new(88.0, 10.0, 10.0),
            start + Duration::from_secs(321),
        )
        .await;
        assert_eq!(retrigger.len(), 1);
    }

    #[tokio::test]
    async fn simulation_spike_triggers_cpu_alert() {
        let mut config = test_config();
        config.alerts.cpu = 94.0;
        config.alerts.ram = 99.0;
        config.alerts.disk = 99.0;
        config.alerts.cooldown_secs = 1;

        let state = Arc::new(Mutex::new(AlertState::default()));
        let mut provider = SimulatedMetricsProvider::new();
        let start = Instant::now();
        let mut cpu_alert_observed = false;

        for tick in 0..40 {
            let metrics = provider
                .collect_metrics()
                .await
                .expect("simulated metrics should be generated");

            let notifications = evaluate_alerts_at(
                &config,
                &state,
                metrics,
                start + Duration::from_secs(tick),
            )
            .await;

            if notifications.iter().any(|n| n.contains("CPU usage is high")) {
                cpu_alert_observed = true;
                break;
            }
        }

        assert!(cpu_alert_observed, "expected simulated CPU spike to trigger alert");
    }
}