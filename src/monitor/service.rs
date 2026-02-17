use std::{sync::Arc, time::Instant};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::config::Config;

use super::{
    evaluator::evaluate_alerts_at,
    provider::MetricsProvider,
    state::{AlertSnapshot, AlertState, DailySummaryReport},
};

pub async fn check_alerts<P: MetricsProvider>(
    bot: &Bot,
    config: &Config,
    state: &Arc<Mutex<AlertState>>,
    provider: &mut P,
) {
    let metrics = match provider.collect_metrics().await {
        Ok(metrics) => metrics,
        Err(error) => {
            log::warn!("monitoring provider error: {}", error);
            return;
        }
    };

    let notifications = evaluate_alerts_at(config, state, metrics, Instant::now()).await;

    {
        let mut state = state.lock().await;
        state.record_metrics(metrics);
        state.record_alerts(notifications.len() as u64);
    }

    let owner_chat_id = match config.owner_chat_id() {
        Ok(chat_id) => chat_id,
        Err(error) => {
            log::error!("CRITICAL: invalid owner chat id in config: {}", error);
            return;
        }
    };

    let muted_until = {
        let state = state.lock().await;
        state.muted_until
    };
    if let Some(until) = muted_until {
        if Utc::now() < until {
            return;
        }
    }

    for notification in notifications {
        if let Err(error) = bot.send_message(owner_chat_id, notification).await {
            log::error!(
                "CRITICAL: Failed to send alert to {}: {}",
                owner_chat_id.0,
                error
            );
        }
    }
}

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

pub async fn mute_alerts_for(
    state: &Arc<Mutex<AlertState>>,
    duration: ChronoDuration,
) -> DateTime<Utc> {
    let until = Utc::now() + duration;
    let mut state = state.lock().await;
    state.muted_until = Some(until);
    until
}

pub async fn unmute_alerts(state: &Arc<Mutex<AlertState>>) {
    let mut state = state.lock().await;
    state.muted_until = None;
}