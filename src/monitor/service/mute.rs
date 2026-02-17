use std::sync::Arc;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use tokio::sync::Mutex;

use super::super::state::AlertState;

const MUTE_ACTION_COOLDOWN_SECS: i64 = 10;

#[derive(Debug, Clone, Copy)]
pub enum MuteActionError {
    Cooldown { retry_after_secs: i64 },
}

fn ensure_mute_action_allowed(
    state: &mut AlertState,
    now: DateTime<Utc>,
) -> Result<(), MuteActionError> {
    if let Some(last) = state.last_mute_action_at {
        let elapsed_secs = now.signed_duration_since(last).num_seconds();
        let remaining = MUTE_ACTION_COOLDOWN_SECS - elapsed_secs;
        if remaining > 0 {
            return Err(MuteActionError::Cooldown {
                retry_after_secs: remaining,
            });
        }
    }

    state.last_mute_action_at = Some(now);
    Ok(())
}

pub async fn mute_alerts_for(
    state: &Arc<Mutex<AlertState>>,
    duration: ChronoDuration,
) -> Result<DateTime<Utc>, MuteActionError> {
    let now = Utc::now();
    let until = now + duration;
    let mut state = state.lock().await;
    ensure_mute_action_allowed(&mut state, now)?;
    state.muted_until = Some(until);
    Ok(until)
}

pub async fn unmute_alerts(state: &Arc<Mutex<AlertState>>) -> Result<(), MuteActionError> {
    let now = Utc::now();
    let mut state = state.lock().await;
    ensure_mute_action_allowed(&mut state, now)?;
    state.muted_until = None;
    Ok(())
}
