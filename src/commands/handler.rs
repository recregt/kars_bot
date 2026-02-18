use teloxide::prelude::*;

use crate::app_context::AppContext;

use super::command_def::MyCommands;
use super::helpers::is_authorized;
use super::router::route_command;

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
            .map(|id| id.0.to_string())
            .unwrap_or_else(|_| "invalid_owner_id".to_string());
        let owner_chat_id = config
            .owner_chat_id()
            .map(|id| id.0.to_string())
            .unwrap_or_else(|_| "invalid_owner_id".to_string());
        let user_id = msg
            .from()
            .map(|user| user.id.0.to_string())
            .unwrap_or_else(|| "unknown".to_string());
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
