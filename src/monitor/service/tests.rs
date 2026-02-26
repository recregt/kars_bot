use std::sync::Arc;

use chrono::{Duration as ChronoDuration, Utc};
use tokio::sync::Mutex;

use crate::monitor::{
    AlertState, CheckAlertsContext, check_alerts,
    provider::{Metrics, MockMetricsProvider},
};
use crate::test_utils::{base_test_config, test_alert_state, test_metric_history};

use super::clock::{Clock, MockClock};
use super::core::check_alerts_with_clock;
use super::mute::{mute_alerts_for_with_clock, unmute_alerts_with_clock};
use super::{
    MuteActionError, alert_snapshot, mute_alerts_for, take_daily_summary_report, unmute_alerts,
};

#[tokio::test]
async fn mute_unmute_contract_is_consistent() {
    let state = Arc::new(Mutex::new(AlertState::default()));

    let muted_until = mute_alerts_for(&state, ChronoDuration::seconds(1))
        .await
        .expect("mute should succeed");
    let snapshot = alert_snapshot(&state).await;

    assert_eq!(snapshot.muted_until, Some(muted_until));
    assert!(muted_until > Utc::now());

    // COOLDOWN is 10 seconds, so rewind the last action time deterministically.
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
    let mut config = base_test_config();
    config.owner_id = 42;
    config.monitor_interval = 1;
    config.command_timeout_secs = 1;
    config.alerts.cpu = 0.0;
    config.alerts.ram = 100.0;
    config.alerts.disk = 100.0;
    config.alerts.cooldown_secs = 1;
    config.alerts.hysteresis = 0.0;
    let mut runtime = crate::config::RuntimeConfig::from_config(&config);
    runtime.alerts.cpu = 0.0;

    let store = crate::reporting_store::NullReportingStorage;
    let state = test_alert_state();
    let history = test_metric_history(1);
    let mut provider = MockMetricsProvider::new(vec![Metrics::new(50.0, 0.0, 0.0)]);
    let notifier = crate::monitor::SpyNotifier::new();
    let anomaly_store = crate::anomaly_db::InMemoryAnomalyStorage::new();

    check_alerts(
        CheckAlertsContext {
            notifier: &notifier,
            config: &config,
            runtime_config: &runtime,
            reporting_store: &store,
            anomaly_storage: &anomaly_store,
            state: &state,
            metric_history: &history,
        },
        &mut provider,
    )
    .await;

    let sent = notifier.sent.lock().await;
    assert_eq!(sent.len(), 1);
    match &sent[0] {
        crate::monitor::SentItem::Message(_, text) => assert!(text.contains("CPU")),
        other => panic!("expected message, got {other:?}"),
    }
}

#[tokio::test]
async fn mute_and_unmute_support_time_travel_without_sleep() {
    let state = Arc::new(Mutex::new(AlertState::default()));
    let clock = MockClock::new(Utc::now());

    let muted_until = mute_alerts_for_with_clock(&state, ChronoDuration::minutes(1), &clock)
        .await
        .expect("initial mute should succeed");
    assert!(muted_until > clock.now_utc());

    let second = mute_alerts_for_with_clock(&state, ChronoDuration::minutes(1), &clock).await;
    assert!(matches!(
        second,
        Err(MuteActionError::Cooldown {
            retry_after_secs: _
        })
    ));

    clock.advance(std::time::Duration::from_secs(11));

    unmute_alerts_with_clock(&state, &clock)
        .await
        .expect("unmute after virtual cooldown should succeed");
    let snapshot = alert_snapshot(&state).await;
    assert_eq!(snapshot.muted_until, None);
}

#[tokio::test]
async fn alert_cooldown_supports_time_travel_without_waiting() {
    let mut config = base_test_config();
    config.owner_id = 42;
    config.monitor_interval = 1;
    config.command_timeout_secs = 1;
    config.alerts.cpu = 0.0;
    config.alerts.ram = 100.0;
    config.alerts.disk = 100.0;
    config.alerts.cooldown_secs = 300;
    config.alerts.hysteresis = 0.0;

    let runtime = crate::config::RuntimeConfig::from_config(&config);
    let store = crate::reporting_store::NullReportingStorage;
    let state = test_alert_state();
    let history = test_metric_history(1);
    let notifier = crate::monitor::SpyNotifier::new();
    let anomaly_store = crate::anomaly_db::InMemoryAnomalyStorage::new();
    let clock = MockClock::new(Utc::now());

    let mut provider = MockMetricsProvider::new(vec![
        Metrics::new(50.0, 0.0, 0.0),
        Metrics::new(60.0, 0.0, 0.0),
        Metrics::new(70.0, 0.0, 0.0),
    ]);

    check_alerts_with_clock(
        CheckAlertsContext {
            notifier: &notifier,
            config: &config,
            runtime_config: &runtime,
            reporting_store: &store,
            anomaly_storage: &anomaly_store,
            state: &state,
            metric_history: &history,
        },
        &mut provider,
        &clock,
    )
    .await;

    clock.advance(std::time::Duration::from_secs(60));
    check_alerts_with_clock(
        CheckAlertsContext {
            notifier: &notifier,
            config: &config,
            runtime_config: &runtime,
            reporting_store: &store,
            anomaly_storage: &anomaly_store,
            state: &state,
            metric_history: &history,
        },
        &mut provider,
        &clock,
    )
    .await;

    clock.advance(std::time::Duration::from_secs(241));
    check_alerts_with_clock(
        CheckAlertsContext {
            notifier: &notifier,
            config: &config,
            runtime_config: &runtime,
            reporting_store: &store,
            anomaly_storage: &anomaly_store,
            state: &state,
            metric_history: &history,
        },
        &mut provider,
        &clock,
    )
    .await;

    let sent = notifier.sent.lock().await;
    assert_eq!(
        sent.len(),
        2,
        "expected one immediate + one post-cooldown alert"
    );
}
