use std::cmp::Ordering;

use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;

use super::super::{
    command_def::MyCommands,
    helpers::{as_html_block, command_error_html, timeout_for},
};

mod orchestrator;
mod version;

const CHANGELOG_PATH: &str = "CHANGELOG.md";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) async fn handle_update(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
    args: &str,
) -> ResponseResult<()> {
    let mode = args.trim().to_lowercase();
    let Some(latest_version) = version::latest_changelog_version(CHANGELOG_PATH) else {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                "Update",
                "Could not read latest version from CHANGELOG.md.\nTry /update check after release metadata is available.",
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    };

    let compare = version::compare_versions(CURRENT_VERSION, &latest_version);
    let runtime_config = app_context.runtime_config.read().await.clone();
    let check_timeout = timeout_for(
        &MyCommands::Update("check".to_string()),
        runtime_config.command_timeout_secs,
    );
    let readiness = orchestrator::run_update_check(check_timeout).await;

    if mode.is_empty() || mode == "check" {
        send_update_check(bot, msg, compare, &latest_version, readiness).await?;
        return Ok(());
    }

    if mode != "apply" {
        bot.send_message(
            msg.chat.id,
            as_html_block("Update Usage", "Usage:\n/update check\n/update apply"),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    if compare != Ordering::Less {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                "Update",
                &format!(
                    "No newer release found. Current v{} is up to date.",
                    CURRENT_VERSION
                ),
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    if !app_context.capabilities.is_systemd {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                "Update",
                "Controlled restart is unavailable on this host (systemd not detected).",
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some((ready, detail)) = orchestrator::extract_readiness(bot, msg, readiness).await? else {
        return Ok(());
    };

    if !ready {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                "Update",
                &format!("Update apply is blocked by pre-checks.\n{}", detail),
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        as_html_block(
            "Update",
            &format!(
                "Starting update to v{}. Service may restart during apply.",
                latest_version
            ),
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    let result = orchestrator::run_update_apply(runtime_config.command_timeout_secs).await;
    match result {
        Ok(message) => {
            bot.send_message(msg.chat.id, as_html_block("Update", &message))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(error) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}

async fn send_update_check(
    bot: &Bot,
    msg: &Message,
    compare: Ordering,
    latest_version: &str,
    readiness: Result<(bool, String), String>,
) -> ResponseResult<()> {
    let readiness_line = match readiness {
        Ok((true, details)) => format!("Apply readiness: ready\n{}", details),
        Ok((false, details)) => format!("Apply readiness: blocked\n{}", details),
        Err(error) => format!("Apply readiness: check failed ({})", error),
    };

    let body = if compare == Ordering::Less {
        format!(
            "Current: v{}\nLatest: v{}\n\nUpdate available.\nRun /update apply to trigger controlled restart.\n\n{}",
            CURRENT_VERSION, latest_version, readiness_line
        )
    } else {
        format!(
            "Current: v{}\nLatest: v{}\n\nYou are up to date.\n\n{}",
            CURRENT_VERSION, latest_version, readiness_line
        )
    };

    bot.send_message(msg.chat.id, as_html_block("Update Check", &body))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
