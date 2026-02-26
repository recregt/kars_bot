use super::command_def::MyCommands;
use super::features::menu::handle_menu_navigation;
use super::helpers::is_authorized;
use super::router::route_command;
use crate::app_context::AppContext;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: MyCommands,
    app_context: &AppContext,
) -> ResponseResult<()> {
    let config = &app_context.config;
    if !is_authorized(&msg, config) {
        let owner_user_id = config
            .owner_user_id()
            .map_or_else(|_| "invalid_owner_id".to_string(), |id| id.0.to_string());
        let owner_chat_id = config
            .owner_chat_id()
            .map_or_else(|_| "invalid_owner_id".to_string(), |id| id.0.to_string());
        let user_id = msg
            .from()
            .map_or_else(|| "unknown".to_string(), |user| user.id.0.to_string());
        log::warn!(
            "SECURITY: Unauthorized access attempt. mode=owner_dm_only expected_user_id={} expected_chat_id={} user_id={} chat_id={} command_text={:?}",
            owner_user_id,
            owner_chat_id,
            user_id,
            msg.chat.id.0,
            msg.text()
        );
        return Ok(());
    }
    route_command(bot, msg, cmd, app_context).await
}

pub async fn answer_callback(
    bot: Bot,
    q: CallbackQuery,
    app_context: Arc<AppContext>,
) -> ResponseResult<()> {
    bot.answer_callback_query(&q.id).await?;

    let msg = match q.message {
        Some(msg) => msg,
        None => return Ok(()),
    };

    let data = match q.data {
        Some(data) => data,
        None => return Ok(()),
    };

    let config = &app_context.config;
    let authorized = config
        .owner_user_id()
        .map(|id| id == q.from.id)
        .unwrap_or(false);

    if !authorized {
        return Ok(());
    }

    if let Some(menu_name) = data.strip_prefix("menu:") {
        return handle_menu_navigation(&bot, &msg, menu_name).await;
    }

    // "cmd:graph:cpu 1h" → "/graph cpu 1h"
    // "cmd:status"       → "/status"
    let parts: Vec<&str> = data.splitn(3, ':').collect();
    if parts.first() != Some(&"cmd") || parts.len() < 2 {
        return Ok(());
    }

    let command_str = if parts.len() == 3 {
        format!("/{} {}", parts[1], parts[2])
    } else {
        format!("/{}", parts[1])
    };

    let cmd = match MyCommands::parse(&command_str, "kars_bot") {
        Ok(cmd) => cmd,
        Err(_) => return Ok(()),
    };

    route_command(bot, msg, cmd, &app_context).await
}
