use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;

use super::super::state::{AlertSnapshot, AlertState, DailySummaryReport};

pub async fn alert_snapshot(state: &Arc<Mutex<AlertState>>) -> AlertSnapshot {
    let state = state.lock().await;
    AlertSnapshot {
        cpu_alerting: state.cpu_alerting,
        ram_alerting: state.ram_alerting,
        disk_alerting: state.disk_alerting,
        muted_until: state.muted_until,
        last_daily_summary_at: state.last_daily_summary_at(),
    }
}

pub async fn take_daily_summary_report(
    state: &Arc<Mutex<AlertState>>,
) -> Option<DailySummaryReport> {
    let mut state = state.lock().await;
    state.take_daily_summary_report(Utc::now())
}
