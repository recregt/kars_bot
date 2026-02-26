use teloxide::{prelude::*, types::ParseMode};

use crate::app_context::AppContext;
use crate::system::run_cmd;

use super::super::super::{
    command_def::MyCommands,
    helpers::{
        acquire_command_slot, as_html_block, command_body, command_error_html,
        maybe_redact_sensitive_output, send_html_or_file, timeout_for,
    },
};
use super::super::menu::send_navigation_hint;
use super::common::unsupported_feature_message;

fn summarize_interfaces(stdout: &str) -> String {
    let interfaces = stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed
                .chars()
                .next()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
            {
                return None;
            }

            let mut parts = trimmed.splitn(3, ':');
            let _idx = parts.next()?;
            let name = parts.next()?.trim();

            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .filter(|name| name != "lo")
        .collect::<Vec<_>>();

    if interfaces.is_empty() {
        return "Interfaces: no non-loopback interface detected".to_string();
    }

    format!("Interfaces: {}", interfaces.join(", "))
}

fn compact_lines(text: &str, max_lines: usize) -> String {
    text.lines().take(max_lines).collect::<Vec<_>>().join("\n")
}

pub(crate) async fn handle_cpu(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let runtime_config = config.runtime_config.read().await.clone();

    if !config.capabilities.has_top {
        bot.send_message(msg.chat.id, unsupported_feature_message("CPU Usage", "top"))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let Some(_permit) = acquire_command_slot(&config.bot_runtime.command_slots, msg, bot).await?
    else {
        return Ok(());
    };
    let message = match run_cmd(
        "top",
        &["-bn1"],
        timeout_for(cmd, runtime_config.command_timeout_secs),
    )
    .await
    {
        Ok(output) => {
            let short = output
                .stdout
                .lines()
                .filter(|line| line.contains("Cpu(s)"))
                .collect::<Vec<_>>()
                .join("\n");
            let body = if short.is_empty() {
                "No CPU output."
            } else {
                &short
            };
            as_html_block("CPU Usage", body)
        }
        Err(error) => command_error_html(&error),
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;
    send_navigation_hint(bot, msg.chat.id, &config.capabilities).await?;

    Ok(())
}

pub(crate) async fn handle_network(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let runtime_config = config.runtime_config.read().await.clone();

    if !config.capabilities.has_ip {
        bot.send_message(
            msg.chat.id,
            unsupported_feature_message("Network Statistics", "ip"),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some(_permit) = acquire_command_slot(&config.bot_runtime.command_slots, msg, bot).await?
    else {
        return Ok(());
    };
    match run_cmd(
        "ip",
        &["-s", "link"],
        timeout_for(cmd, runtime_config.command_timeout_secs),
    )
    .await
    {
        Ok(output) => {
            let raw_body = command_body(&output);
            let summary = summarize_interfaces(&output.stdout);
            let compact = compact_lines(&raw_body, 24);
            let body = maybe_redact_sensitive_output(
                &format!("{}\n\nDetails:\n{}", summary, compact),
                config.config.security.redact_sensitive_output,
            );
            send_html_or_file(bot, msg.chat.id, "Network Statistics", &body).await?;
            send_navigation_hint(bot, msg.chat.id, &config.capabilities).await?;
        }
        Err(error) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}

pub(crate) async fn handle_uptime(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let runtime_config = config.runtime_config.read().await.clone();

    if !config.capabilities.has_uptime {
        bot.send_message(
            msg.chat.id,
            unsupported_feature_message("System Uptime", "uptime"),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some(_permit) = acquire_command_slot(&config.bot_runtime.command_slots, msg, bot).await?
    else {
        return Ok(());
    };
    match run_cmd(
        "uptime",
        &[],
        timeout_for(cmd, runtime_config.command_timeout_secs),
    )
    .await
    {
        Ok(output) => {
            let body = command_body(&output);
            send_html_or_file(bot, msg.chat.id, "System Uptime", &body).await?;
            send_navigation_hint(bot, msg.chat.id, &config.capabilities).await?;
        }
        Err(error) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}

pub(crate) async fn handle_temp(
    bot: &Bot,
    msg: &Message,
    config: &AppContext,
    cmd: &MyCommands,
) -> ResponseResult<()> {
    let runtime_config = config.runtime_config.read().await.clone();

    if !config.capabilities.has_sensors {
        bot.send_message(
            msg.chat.id,
            unsupported_feature_message("Temperature Sensors", "sensors"),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let Some(_permit) = acquire_command_slot(&config.bot_runtime.command_slots, msg, bot).await?
    else {
        return Ok(());
    };
    match run_cmd(
        "sensors",
        &[],
        timeout_for(cmd, runtime_config.command_timeout_secs),
    )
    .await
    {
        Ok(output) => {
            let body = command_body(&output);
            send_html_or_file(bot, msg.chat.id, "Temperature Sensors", &body).await?;
            send_navigation_hint(bot, msg.chat.id, &config.capabilities).await?;
        }
        Err(error) => {
            bot.send_message(msg.chat.id, command_error_html(&error))
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}
