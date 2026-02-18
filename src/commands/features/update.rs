use std::{
    cmp::Ordering,
    fs,
};

use teloxide::{prelude::*, types::ParseMode};

use crate::{
    app_context::AppContext,
    system::run_cmd,
};

use super::super::{
    command_def::MyCommands,
    helpers::{as_html_block, command_error_html, timeout_for},
};

const CHANGELOG_PATH: &str = "CHANGELOG.md";
const SERVICE_NAME: &str = "kars-bot";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) async fn handle_update(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
    args: &str,
) -> ResponseResult<()> {
    let mode = args.trim().to_lowercase();
    let latest = latest_changelog_version(CHANGELOG_PATH);

    let Some(latest_version) = latest else {
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

    let compare = compare_versions(CURRENT_VERSION, &latest_version);

    if mode.is_empty() || mode == "check" {
        let body = if compare == Ordering::Less {
            format!(
                "Current: v{}\nLatest: v{}\n\nUpdate available.\nRun /update apply to trigger controlled restart via systemd.",
                CURRENT_VERSION, latest_version
            )
        } else {
            format!(
                "Current: v{}\nLatest: v{}\n\nYou are up to date.",
                CURRENT_VERSION, latest_version
            )
        };

        bot.send_message(msg.chat.id, as_html_block("Update Check", &body))
            .parse_mode(ParseMode::Html)
            .await?;
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

    let runtime_config = app_context.runtime_config.read().await.clone();
    match run_cmd(
        "systemctl",
        &["restart", SERVICE_NAME],
        timeout_for(&MyCommands::Update("apply".to_string()), runtime_config.command_timeout_secs),
    )
    .await
    {
        Ok(_) => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Update",
                    &format!(
                        "Applied update path to v{} and requested controlled restart for service '{}'.",
                        latest_version, SERVICE_NAME
                    ),
                ),
            )
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

fn latest_changelog_version(changelog_path: &str) -> Option<String> {
    let content = fs::read_to_string(changelog_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("## ") {
            continue;
        }

        let after_header = trimmed.trim_start_matches("## ").trim();
        let version_token = after_header
            .split_whitespace()
            .next()
            .map(|token| token.trim_start_matches('v'))?;

        if parse_version(version_token).is_some() {
            return Some(version_token.to_string());
        }
    }

    None
}

fn compare_versions(current: &str, latest: &str) -> Ordering {
    match (parse_version(current), parse_version(latest)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => Ordering::Equal,
    }
}

fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut it = version.split('.');
    let major = it.next()?.parse::<u64>().ok()?;
    let minor = it.next()?.parse::<u64>().ok()?;
    let patch = it.next()?.parse::<u64>().ok()?;
    if it.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::{compare_versions, parse_version};

    #[test]
    fn parses_semver_triplet() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("x.y.z"), None);
    }

    #[test]
    fn compares_versions_correctly() {
        assert_eq!(compare_versions("1.0.0", "1.1.0"), Ordering::Less);
        assert_eq!(compare_versions("1.1.0", "1.1.0"), Ordering::Equal);
        assert_eq!(compare_versions("1.2.0", "1.1.9"), Ordering::Greater);
    }
}
