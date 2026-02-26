use std::cmp::Ordering;

use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::capabilities::Capabilities;

use super::super::{
    command_def::MyCommands,
    helpers::{as_html_block, timeout_for},
};
use super::menu::main_menu_keyboard;

mod orchestrator;
mod self_update;
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
    let latest_version = match self_update::fetch_latest_dist_manifest().await {
        Ok(manifest) => manifest.latest.version,
        Err(_) => version::latest_changelog_version(CHANGELOG_PATH)
            .unwrap_or_else(|| CURRENT_VERSION.to_string()),
    };

    let compare = version::compare_versions(CURRENT_VERSION, &latest_version);
    let runtime_config = app_context.runtime_config.read().await.clone();
    let check_timeout = timeout_for(
        &MyCommands::Update("check".to_string()),
        runtime_config.command_timeout_secs,
    );
    let readiness = orchestrator::run_update_check(check_timeout, CURRENT_VERSION).await;

    if mode.is_empty() || mode == "check" {
        send_update_check(
            bot,
            msg,
            compare,
            &latest_version,
            readiness,
            &app_context.capabilities,
        )
        .await?;
        return Ok(());
    }

    if mode != "apply" {
        bot.send_message(
            msg.chat.id,
            as_html_block("Update Usage", "Usage:\n/update check\n/update apply"),
        )
        .reply_markup(main_menu_keyboard(&app_context.capabilities))
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    if compare != Ordering::Less {
        bot.send_message(
            msg.chat.id,
            as_html_block(
                "Update",
                &format!("No newer release found. Current v{CURRENT_VERSION} is up to date."),
            ),
        )
        .reply_markup(main_menu_keyboard(&app_context.capabilities))
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
        .reply_markup(main_menu_keyboard(&app_context.capabilities))
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
                &format!("Update apply is blocked by pre-checks.\n{detail}"),
            ),
        )
        .reply_markup(main_menu_keyboard(&app_context.capabilities))
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        as_html_block(
            "Update",
            &format!("Starting update to v{latest_version}. Service may restart during apply."),
        ),
    )
    .reply_markup(main_menu_keyboard(&app_context.capabilities))
    .parse_mode(ParseMode::Html)
    .await?;

    let result =
        orchestrator::run_update_apply(runtime_config.command_timeout_secs, CURRENT_VERSION).await;
    match result {
        Ok(message) => {
            bot.send_message(msg.chat.id, as_html_block("Update", &message))
                .reply_markup(main_menu_keyboard(&app_context.capabilities))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(error) => {
            bot.send_message(
                msg.chat.id,
                as_html_block("Update", &format!("Update apply failed: {error}")),
            )
            .reply_markup(main_menu_keyboard(&app_context.capabilities))
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
    capabilities: &Capabilities,
) -> ResponseResult<()> {
    let readiness_line = match readiness {
        Ok((true, details)) => format!("Apply readiness: ready\n{details}"),
        Ok((false, details)) => format!("Apply readiness: blocked\n{details}"),
        Err(error) => format!("Apply readiness: check failed ({error})"),
    };

    let body = if compare == Ordering::Less {
        format!(
            "Current: v{CURRENT_VERSION}\nLatest: v{latest_version}\n\nUpdate available.\nRun /update apply to trigger controlled restart.\n\n{readiness_line}"
        )
    } else {
        format!(
            "Current: v{CURRENT_VERSION}\nLatest: v{latest_version}\n\nYou are up to date.\n\n{readiness_line}"
        )
    };

    bot.send_message(msg.chat.id, as_html_block("Update Check", &body))
        .reply_markup(main_menu_keyboard(capabilities))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
