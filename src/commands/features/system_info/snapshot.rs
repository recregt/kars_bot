use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::system::run_cmd;

use super::super::super::{
    command_def::MyCommands,
    helpers::{
        acquire_command_slot, command_body, command_error_html, maybe_redact_sensitive_output,
        send_html_or_file, timeout_for,
    },
};
use super::super::menu::send_navigation_hint;
use super::common::unsupported_feature_message;

fn summarize_memory(stdout: &str) -> Option<String> {
    let mem_line = stdout
        .lines()
        .find(|line| line.trim_start().starts_with("Mem:"))?;
    let cols: Vec<&str> = mem_line.split_whitespace().collect();
    if cols.len() < 4 {
        return None;
    }

    Some(format!(
        "Memory: used {} / total {} (free {})",
        cols[2], cols[1], cols[3]
    ))
}

fn summarize_root_disk(stdout: &str) -> Option<String> {
    let root_line = stdout
        .lines()
        .find(|line| line.split_whitespace().last() == Some("/"))?;
    let cols: Vec<&str> = root_line.split_whitespace().collect();
    if cols.len() < 6 {
        return None;
    }

    Some(format!(
        "Disk (/): used {} / size {} ({} used, avail {})",
        cols[2], cols[1], cols[4], cols[3]
    ))
}

fn compact_lines(text: &str, max_lines: usize) -> String {
    text.lines().take(max_lines).collect::<Vec<_>>().join("\n")
}

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

    let Some(_permit) = acquire_command_slot(&config.bot_runtime.command_slots, msg, bot).await?
    else {
        return Ok(());
    };

    let timeout = timeout_for(cmd, runtime_config.command_timeout_secs);
    let ram = run_cmd("free", &["-h"], timeout).await;
    let disk = run_cmd("df", &["-h", "/"], timeout).await;

    match (ram, disk) {
        (Ok(ram_out), Ok(disk_out)) => {
            let memory_summary = summarize_memory(&ram_out.stdout)
                .unwrap_or_else(|| "Memory summary unavailable".to_string());
            let disk_summary = summarize_root_disk(&disk_out.stdout)
                .unwrap_or_else(|| "Disk summary unavailable".to_string());

            let body = format!(
                "Summary:\n- {}\n- {}\n\nRAM (details):\n{}\n\nDisk (details):\n{}",
                memory_summary,
                disk_summary,
                compact_lines(&command_body(&ram_out), 12),
                compact_lines(&command_body(&disk_out), 12)
            );
            send_html_or_file(bot, msg.chat.id, "System Snapshot", &body).await?;
            send_navigation_hint(bot, msg.chat.id).await?;
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

    let Some(_permit) = acquire_command_slot(&config.bot_runtime.command_slots, msg, bot).await?
    else {
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
            let raw_body = command_body(&output);
            let body = maybe_redact_sensitive_output(
                &raw_body,
                config.config.security.redact_sensitive_output,
            );
            send_html_or_file(bot, msg.chat.id, "Open Ports", &body).await?;
            send_navigation_hint(bot, msg.chat.id).await?;
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

    let Some(_permit) = acquire_command_slot(&config.bot_runtime.command_slots, msg, bot).await?
    else {
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
            let redacted =
                maybe_redact_sensitive_output(body, config.config.security.redact_sensitive_output);
            send_html_or_file(bot, msg.chat.id, "Active Services", &redacted).await?;
            send_navigation_hint(bot, msg.chat.id).await?;
        }
        Err(error) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}
