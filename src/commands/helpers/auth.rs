use teloxide::prelude::*;

use crate::config::Config;

pub(crate) fn is_authorized(msg: &Message, config: &Config) -> bool {
    let Some(from) = msg.from() else {
        return false;
    };

    let owner_user_id = match config.owner_user_id() {
        Ok(owner_user_id) => owner_user_id,
        Err(_) => return false,
    };

    let owner_chat_id = match config.owner_chat_id() {
        Ok(owner_chat_id) => owner_chat_id,
        Err(_) => return false,
    };

    if from.id != owner_user_id {
        return false;
    }

    msg.chat.id == owner_chat_id
}
