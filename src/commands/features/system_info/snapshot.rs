use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::system::run_cmd;

use super::super::super::{
    command_def::MyCommands,
    helpers::{
        acquire_command_slot, command_body, command_error_html, send_html_or_file, timeout_for,
    },
};
use super::common::unsupported_feature_message;

pub(crate) async fn handle_sys_status(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let runtime_config = config.runtime_config.read().await.clone();

    if !config.capabilities.has_free {
        bot.send_message(
            msg.chat.id,
            unsupported_feature_message("System Snapshot", "free"),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };

    let timeout = timeout_for(cmd, runtime_config.command_timeout_secs);
    let ram = run_cmd("free", &["-h"], timeout).await;
    let disk = run_cmd("df", &["-h", "/"], timeout).await;

    match (ram, disk) {
        (Ok(ram_out), Ok(disk_out)) => {
            let body = format!(
                "RAM:\n{}\n\nDisk:\n{}",
                command_body(&ram_out),
                command_body(&disk_out)
            );
            send_html_or_file(bot, msg.chat.id, "System Snapshot", &body).await?;
        }
        (Err(error), _) | (_, Err(error)) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}

pub(crate) async fn handle_ports(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let runtime_config = config.runtime_config.read().await.clone();

    if !config.capabilities.has_ss {
        bot.send_message(msg.chat.id, unsupported_feature_message("Open Ports", "ss"))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };
    match run_cmd(
        "ss",
        &["-tuln"],
        timeout_for(cmd, runtime_config.command_timeout_secs),
    )
    .await
    {
        Ok(output) => {
            let body = command_body(&output);
            send_html_or_file(bot, msg.chat.id, "Open Ports", &body).await?;
        }
        Err(error) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}

pub(crate) async fn handle_services(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let runtime_config = config.runtime_config.read().await.clone();

    if !config.capabilities.is_systemd {
        bot.send_message(
            msg.chat.id,
            unsupported_feature_message("Active Services", "systemctl + systemd"),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };

    let services = run_cmd(
        "systemctl",
        &[
            "list-units",
            "--type=service",
            "--state=running",
            "--no-pager",
        ],
        timeout_for(cmd, runtime_config.command_timeout_secs),
    )
    .await;

    match services {
        Ok(output) => {
            let short = output
                .stdout
                .lines()
                .filter(|line| line.contains(".service"))
                .take(10)
                .collect::<Vec<_>>()
                .join("\n");
            let body = if short.is_empty() {
                "No service output."
            } else {
                &short
            };
            send_html_or_file(bot, msg.chat.id, "Active Services", body).await?;
        }
        Err(error) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}
