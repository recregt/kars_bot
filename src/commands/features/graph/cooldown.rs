use std::time::{Duration, Instant};

use crate::app_context::AppContext;

const GRAPH_COMMAND_COOLDOWN_SECS: u64 = 5;

pub(super) async fn graph_cooldown_remaining_secs(app_context: &AppContext) -> Option<u64> {
    let now = Instant::now();
    let cooldown = Duration::from_secs(GRAPH_COMMAND_COOLDOWN_SECS);

    let mut last_used = app_context.last_graph_command_at.lock().await;
    if let Some(previous) = *last_used {
        let elapsed = now.saturating_duration_since(previous);
        if elapsed < cooldown {
            let remaining = cooldown - elapsed;
            return Some(remaining.as_secs().max(1));
        }
    }

    *last_used = Some(now);
    None
}
