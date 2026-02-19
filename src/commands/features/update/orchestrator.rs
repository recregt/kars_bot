use std::sync::OnceLock;

use teloxide::{prelude::*, types::ParseMode};
use tokio::{
    sync::Semaphore,
    time::{Duration, timeout},
};

use crate::app_context::AppContext;
use crate::system::{CommandError, CommandOutput, run_cmd};

use super::super::super::helpers::{as_html_block, command_body};

const UPDATE_SCRIPT_PATH: &str = "scripts/server_update.sh";
const UPDATE_LOCK_TIMEOUT_SECS: u64 = 2;
const APPLY_TIMEOUT_SECS: u64 = 120;
const COMMAND_DRAIN_TIMEOUT_SECS: u64 = 8;

static UPDATE_APPLY_LOCK: OnceLock<Semaphore> = OnceLock::new();

pub(super) async fn run_update_check(timeout_secs: u64) -> Result<(bool, String), String> {
    let output = run_cmd("bash", &[UPDATE_SCRIPT_PATH, "--check-only"], timeout_secs)
        .await
        .map_err(|error| error.to_string())?;

    Ok((output.status == 0, summarize_output(&output)))
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
    command_timeout_secs: u64,
    app_context: &AppContext,
) -> Result<String, CommandError> {
    let lock = update_apply_lock();
    let permit = timeout(
        Duration::from_secs(UPDATE_LOCK_TIMEOUT_SECS),
        lock.acquire(),
    )
    .await
    .map_err(|_| CommandError::Timeout {
        cmd: "update apply lock".to_string(),
        timeout_secs: UPDATE_LOCK_TIMEOUT_SECS,
    })?
    .map_err(|source| CommandError::Io {
        cmd: "update apply lock".to_string(),
        source: std::io::Error::other(source.to_string()),
    })?;

    let drain_permit = timeout(
        Duration::from_secs(COMMAND_DRAIN_TIMEOUT_SECS),
        app_context
            .command_slots
            .acquire_many(app_context.command_concurrency),
    )
    .await
    .map_err(|_| CommandError::Timeout {
        cmd: "command drain before update".to_string(),
        timeout_secs: COMMAND_DRAIN_TIMEOUT_SECS,
    })?
    .map_err(|source| CommandError::Io {
        cmd: "command drain before update".to_string(),
        source: std::io::Error::other(source.to_string()),
    })?;

    let flushed = app_context
        .flush_reporting_store_barrier()
        .map_err(|detail| CommandError::Io {
            cmd: "reporting store flush barrier".to_string(),
            source: std::io::Error::other(detail),
        })?;

    let output = run_cmd(
        "bash",
        &[UPDATE_SCRIPT_PATH],
        command_timeout_secs.max(APPLY_TIMEOUT_SECS),
    )
    .await;

    drop(drain_permit);
    drop(permit);

    output.map(|out| {
        let flush_line = if flushed {
            "Storage flush barrier: ok"
        } else {
            "Storage flush barrier: skipped (reporting store disabled)"
        };

        if out.status == 0 {
            format!(
                "Update succeeded.\n{}\n{}",
                flush_line,
                summarize_output(&out)
            )
        } else {
            format!(
                "Update failed (status {}).\n{}\n{}",
                out.status,
                flush_line,
                summarize_output(&out)
            )
        }
    })
}

fn update_apply_lock() -> &'static Semaphore {
    UPDATE_APPLY_LOCK.get_or_init(|| Semaphore::new(1))
}

fn summarize_output(output: &CommandOutput) -> String {
    command_body(output)
        .lines()
        .take(12)
        .collect::<Vec<_>>()
        .join("\n")
}
