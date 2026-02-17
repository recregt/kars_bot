use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::system::run_cmd;

use super::super::{
    command_def::MyCommands,
    helpers::{acquire_command_slot, as_html_block, command_body, command_error_html, timeout_for},
};

pub(crate) async fn handle_status(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };
    let timeout = timeout_for(cmd, &config.config);
    let ram = run_cmd("free", &["-h"], timeout).await;
    let disk = run_cmd("df", &["-h", "/"], timeout).await;

    let message = match (ram, disk) {
        (Ok(ram_out), Ok(disk_out)) => {
            let body = format!(
                "RAM:\n{}\n\nDisk:\n{}",
                command_body(&ram_out),
                command_body(&disk_out)
            );
            as_html_block("System Status", &body)
        }
        (Err(error), _) | (_, Err(error)) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_ports(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };
    let message = match run_cmd("ss", &["-tuln"], timeout_for(cmd, &config.config)).await {
        Ok(output) => as_html_block("Open Ports", &command_body(&output)),
        Err(error) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_services(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
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
        timeout_for(cmd, &config.config),
    )
    .await;

    let message = match services {
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
            as_html_block("Active Services", body)
        }
        Err(error) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_cpu(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };
    let message = match run_cmd("top", &["-bn1"], timeout_for(cmd, &config.config)).await {
        Ok(output) => {
            let short = output
                .stdout
                .lines()
                .filter(|line| line.contains("Cpu(s)"))
                .collect::<Vec<_>>()
                .join("\n");
            let body = if short.is_empty() { "No CPU output." } else { &short };
            as_html_block("CPU Usage", body)
        }
        Err(error) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_network(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };
    let message = match run_cmd("ip", &["-s", "link"], timeout_for(cmd, &config.config)).await {
        Ok(output) => as_html_block("Network Statistics", &command_body(&output)),
        Err(error) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_uptime(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };
    let message = match run_cmd("uptime", &[], timeout_for(cmd, &config.config)).await {
        Ok(output) => as_html_block("System Uptime", &command_body(&output)),
        Err(error) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub(crate) async fn handle_temp(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let Some(_permit) = acquire_command_slot(&config.command_slots, msg, bot).await? else {
        return Ok(());
    };
    let message = match run_cmd("sensors", &[], timeout_for(cmd, &config.config)).await {
        Ok(output) => as_html_block("Temperature Sensors", &command_body(&output)),
        Err(error) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
