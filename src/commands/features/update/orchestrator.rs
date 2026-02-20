use std::sync::OnceLock;

use teloxide::{prelude::*, types::ParseMode};
use tokio::{
    sync::Semaphore,
    time::{Duration, timeout},
};

use super::super::super::helpers::as_html_block;
use super::self_update;

const UPDATE_LOCK_TIMEOUT_SECS: u64 = 2;

static UPDATE_APPLY_LOCK: OnceLock<Semaphore> = OnceLock::new();

pub(super) async fn run_update_check(
    _timeout_secs: u64,
    current_version: &str,
) -> Result<(bool, String), String> {
    let manifest = self_update::fetch_latest_dist_manifest().await?;
    if !self_update::update_available(current_version, &manifest.latest.version) {
        return Ok((true, "No update available for current version.".to_string()));
    }

    Ok(self_update::summarize_manifest_readiness(&manifest))
}

pub(super) async fn extract_readiness(
    bot: &Bot,
    msg: &Message,
    readiness: Result<(bool, String), String>,
) -> ResponseResult<Option<(bool, String)>> {
    match readiness {
        Ok(result) => Ok(Some(result)),
        Err(error) => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Update",
                    &format!("Update check failed before apply: {}", error),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            Ok(None)
        }
    }
}

pub(super) async fn run_update_apply(
    _command_timeout_secs: u64,
    current_version: &str,
) -> Result<String, String> {
    let lock = update_apply_lock();
    let permit = timeout(
        Duration::from_secs(UPDATE_LOCK_TIMEOUT_SECS),
        lock.acquire(),
    )
    .await
    .map_err(|_| {
        format!(
            "update apply lock timeout after {}s",
            UPDATE_LOCK_TIMEOUT_SECS
        )
    })?
    .map_err(|source| format!("update apply lock error: {source}"))?;

    let output = self_update::run_self_update(current_version).await;

    drop(permit);

    output.map(|maybe_message| {
        maybe_message.unwrap_or_else(|| "No update was applied (already up to date).".to_string())
    })
}

fn update_apply_lock() -> &'static Semaphore {
    UPDATE_APPLY_LOCK.get_or_init(|| Semaphore::new(1))
}
