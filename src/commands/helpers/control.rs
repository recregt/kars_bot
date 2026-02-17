use std::sync::Arc;

use chrono::Duration as ChronoDuration;
use teloxide::{prelude::*, types::ParseMode};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use super::formatting::as_html_block;
use crate::commands::command_def::MyCommands;

const FAST_TIMEOUT_SECS: u64 = 5;

pub(crate) fn timeout_for(cmd: &MyCommands, command_timeout_secs: u64) -> u64 {
    match cmd {
        MyCommands::Status
        | MyCommands::Sysstatus
        | MyCommands::Ports
        | MyCommands::Cpu
        | MyCommands::Network
        | MyCommands::Uptime
        | MyCommands::Health
        | MyCommands::Alerts
        | MyCommands::Graph(_)
        | MyCommands::Export(_)
        | MyCommands::Recent(_)
        | MyCommands::Mute(_)
        | MyCommands::Unmute
        | MyCommands::Help => FAST_TIMEOUT_SECS,
        MyCommands::Services | MyCommands::Temp => command_timeout_secs,
    }
}

pub(crate) fn parse_mute_duration(input: &str) -> Option<ChronoDuration> {
    let normalized = input.trim().to_lowercase();
    if normalized.len() < 2 {
        return None;
    }

    let (value_part, unit_part) = normalized.split_at(normalized.len() - 1);
    let value = value_part.parse::<i64>().ok()?;
    if value <= 0 {
        return None;
    }

    match unit_part {
        "s" => Some(ChronoDuration::seconds(value)),
        "m" => Some(ChronoDuration::minutes(value)),
        "h" => Some(ChronoDuration::hours(value)),
        "d" => Some(ChronoDuration::days(value)),
        _ => None,
    }
}

pub(crate) async fn acquire_command_slot(
    command_slots: &Arc<Semaphore>,
    msg: &Message,
    bot: &Bot,
) -> ResponseResult<Option<OwnedSemaphorePermit>> {
    match command_slots.clone().acquire_owned().await {
        Ok(permit) => Ok(Some(permit)),
        Err(error) => {
            log::error!("failed to acquire command semaphore: {}", error);
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Command queue error",
                    "Could not acquire command slot. Please try again.",
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            Ok(None)
        }
    }
}
