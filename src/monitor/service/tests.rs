use std::sync::Arc;

use chrono::{Duration as ChronoDuration, Utc};
use tokio::sync::Mutex;

use crate::monitor::{AlertState, provider::Metrics};

use super::{
    MuteActionError, alert_snapshot, mute_alerts_for, take_daily_summary_report, unmute_alerts,
};

#[tokio::test]
async fn mute_unmute_contract_is_consistent() {
    let state = Arc::new(Mutex::new(AlertState::default()));

    let muted_until = mute_alerts_for(&state, ChronoDuration::minutes(10))
        .await
        .expect("mute should succeed");
    let snapshot = alert_snapshot(&state).await;

    assert_eq!(snapshot.muted_until, Some(muted_until));
    assert!(muted_until > Utc::now());

    tokio::time::sleep(std::time::Duration::from_secs(11)).await;

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
