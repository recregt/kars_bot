use std::sync::Arc;

use chrono::{Duration as ChronoDuration, Utc};
use tokio::sync::Mutex;

use crate::monitor::{
    AlertState, check_alerts,
    provider::{Metrics, MockMetricsProvider},
};

use super::{
    MuteActionError, alert_snapshot, mute_alerts_for, take_daily_summary_report, unmute_alerts,
};

#[tokio::test]
async fn mute_unmute_contract_is_consistent() {
    let state = Arc::new(Mutex::new(AlertState::default()));

    // use a small duration so test finishes quickly
    let muted_until = mute_alerts_for(&state, ChronoDuration::seconds(1))
        .await
        .expect("mute should succeed");
    let snapshot = alert_snapshot(&state).await;

    assert_eq!(snapshot.muted_until, Some(muted_until));
    assert!(muted_until > Utc::now());

    // wait only long enough for the mute duration to pass
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

    // COOLDOWN is 10 seconds, so artificially rewind the last action time
    {
        let mut lock = state.lock().await;
        if let Some(last) = lock.last_mute_action_at {
            lock.last_mute_action_at = Some(last - ChronoDuration::seconds(11));
        }
    }

    unmute_alerts(&state).await.expect("unmute should succeed");
    let snapshot_after = alert_snapshot(&state).await;
    assert_eq!(snapshot_after.muted_until, None);
}

#[tokio::test]
async fn mute_unmute_has_short_cooldown() {
    let state = Arc::new(Mutex::new(AlertState::default()));

    let _ = mute_alerts_for(&state, ChronoDuration::minutes(1))
        .await
        .expect("first mute should succeed");

    let second = mute_alerts_for(&state, ChronoDuration::minutes(1)).await;
    assert!(matches!(
        second,
        Err(MuteActionError::Cooldown {
            retry_after_secs: _
        })
    ));
}

#[tokio::test]
async fn daily_summary_report_contract_resets_window() {
    let state = Arc::new(Mutex::new(AlertState::default()));

    {
        let mut lock = state.lock().await;
        lock.record_metrics(Metrics::new(30.0, 40.0, 50.0));
        lock.record_metrics(Metrics::new(50.0, 60.0, 70.0));
        lock.record_alerts(3);
    }

    let report = take_daily_summary_report(&state)
        .await
        .expect("report should exist");
    assert_eq!(report.sample_count, 2);
    assert_eq!(report.alert_count, 3);
    assert_eq!(report.cpu_min, 30.0);
    assert_eq!(report.cpu_max, 50.0);

    let next_report = take_daily_summary_report(&state).await;
    assert!(next_report.is_none());
}

#[tokio::test]
async fn notifications_triggered_when_threshold_exceeded() {
    // prepare minimal config with low thresholds
    let config = crate::config::Config {
        bot_token: "tok".to_string(),
        owner_id: 42,
        monitor_interval: 1,
        command_timeout_secs: 1,
        alerts: crate::config::Alerts {
            cpu: 0.0,
            ram: 100.0,
            disk: 100.0,
            cooldown_secs: 1,
            hysteresis: 0.0,
        },
        daily_summary: Default::default(),
        weekly_report: Default::default(),
        graph: Default::default(),
        anomaly_db: Default::default(),
        simulation: Default::default(),
        reporting_store: Default::default(),
        release_notifier: Default::default(),
        security: Default::default(),
    };
    let mut runtime = crate::config::RuntimeConfig::from_config(&config);
    runtime.alerts.cpu = 0.0;

    let store = crate::reporting_store::NullReportingStorage;
    let state = Arc::new(Mutex::new(AlertState::default()));
    let history = Arc::new(Mutex::new(
        crate::monitor::MetricHistory::with_monitor_interval_secs(1),
    ));
    let mut provider = MockMetricsProvider::new(vec![Metrics::new(50.0, 0.0, 0.0)]);
    let notifier = crate::monitor::SpyNotifier::new();
    let anomaly_store = crate::anomaly_db::InMemoryAnomalyStorage::new();

    check_alerts(
        &notifier,
        &config,
        &runtime,
        &store,
        &anomaly_store,
        &state,
        &history,
        &mut provider,
    )
    .await;

    let sent = notifier.sent.lock().await;
    assert_eq!(sent.len(), 1);
    match &sent[0] {
        crate::monitor::SentItem::Message(_, text) => assert!(text.contains("CPU")),
        other => panic!("expected message, got {:?}", other),
    }
}
