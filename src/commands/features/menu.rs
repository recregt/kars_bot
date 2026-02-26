use teloxide::{
    RequestError,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::commands::helpers::as_html_card;

pub(crate) fn main_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📊 Status", "cmd:status"),
            InlineKeyboardButton::callback("💓 Health", "cmd:health"),
            InlineKeyboardButton::callback("🚨 Alerts", "cmd:alerts"),
        ],
        vec![
            InlineKeyboardButton::callback("🖥️ System", "menu:system"),
            InlineKeyboardButton::callback("📈 Monitor", "menu:monitor"),
            InlineKeyboardButton::callback("📦 Data", "menu:data"),
        ],
        vec![InlineKeyboardButton::callback("❓ Help", "menu:help")],
    ])
}

fn system_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📦 Sys Snapshot", "cmd:sysstatus"),
            InlineKeyboardButton::callback("🧠 CPU", "cmd:cpu"),
            InlineKeyboardButton::callback("🌡️ Temp", "cmd:temp"),
        ],
        vec![
            InlineKeyboardButton::callback("🌐 Network", "cmd:network"),
            InlineKeyboardButton::callback("⏱️ Uptime", "cmd:uptime"),
        ],
        vec![
            InlineKeyboardButton::callback("🔌 Ports", "cmd:ports"),
            InlineKeyboardButton::callback("🧩 Services", "cmd:services"),
        ],
        vec![InlineKeyboardButton::callback("⬅️ Main Menu", "menu:main")],
    ])
}

fn monitor_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📈 CPU 1h", "cmd:graph:cpu 1h"),
            InlineKeyboardButton::callback("💾 RAM 1h", "cmd:graph:ram 1h"),
            InlineKeyboardButton::callback("🧱 Disk 1h", "cmd:graph:disk 1h"),
        ],
        vec![
            InlineKeyboardButton::callback("🧾 Recent 6h", "cmd:recent:6h"),
            InlineKeyboardButton::callback("📤 Export CPU", "cmd:export:cpu 1h csv"),
        ],
        vec![
            InlineKeyboardButton::callback("🔇 Mute 1h", "cmd:mute:1h"),
            InlineKeyboardButton::callback("🔔 Unmute", "cmd:unmute"),
        ],
        vec![InlineKeyboardButton::callback("⬅️ Main Menu", "menu:main")],
    ])
}

fn data_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🧾 Recent 24h", "cmd:recent:24h"),
            InlineKeyboardButton::callback("📤 Export CPU", "cmd:export:cpu 6h csv"),
        ],
        vec![
            InlineKeyboardButton::callback("📤 Export RAM", "cmd:export:ram 6h csv"),
            InlineKeyboardButton::callback("📤 Export Disk", "cmd:export:disk 6h csv"),
        ],
        vec![InlineKeyboardButton::callback("⬅️ Main Menu", "menu:main")],
    ])
}

fn menu_screen(menu_name: &str) -> Option<(&'static str, &'static str)> {
    match menu_name {
        "main" => Some((
            "Main",
            "Use buttons to run actions directly. No need to type commands manually.",
        )),
        "system" => Some(("Main › System", "Live host diagnostics and service checks.")),
        "monitor" => Some((
            "Main › Monitor",
            "Alert controls, graphs and anomaly drill-down actions.",
        )),
        "data" => Some(("Main › Data", "Recent anomalies and export shortcuts.")),
        "help" => Some((
            "Main › Help",
            "1) Open a menu\n2) Tap an action\n3) Use ⬅️ Main Menu to continue\n\nTip: Slash commands still work, but all common flows are available as buttons.",
        )),
        _ => None,
    }
}

fn menu_keyboard(menu_name: &str) -> Option<InlineKeyboardMarkup> {
    match menu_name {
        "main" => Some(main_menu_keyboard()),
        "system" => Some(system_menu_keyboard()),
        "monitor" => Some(monitor_menu_keyboard()),
        "data" => Some(data_menu_keyboard()),
        "help" => Some(main_menu_keyboard()),
        _ => None,
    }
}

fn is_not_modified_error(error: &RequestError) -> bool {
    error
        .to_string()
        .to_lowercase()
        .contains("message is not modified")
}

pub(crate) async fn handle_menu_navigation(
    bot: &Bot,
    msg: &Message,
    menu_name: &str,
) -> ResponseResult<()> {
    let Some((title, description)) = menu_screen(menu_name) else {
        return Ok(());
    };
    let Some(keyboard) = menu_keyboard(menu_name) else {
        return Ok(());
    };

    let content = as_html_card(
        title,
        &format!("• {}", html_escape::encode_text(description)),
    );
    upsert_message_with_keyboard(bot, msg, content, keyboard).await?;

    Ok(())
}

pub(crate) async fn upsert_message_with_menu(
    bot: &Bot,
    msg: &Message,
    content: String,
    menu_name: &str,
) -> ResponseResult<()> {
    let Some(keyboard) = menu_keyboard(menu_name) else {
        return Ok(());
    };

    upsert_message_with_keyboard(bot, msg, content, keyboard).await
}

async fn upsert_message_with_keyboard(
    bot: &Bot,
    msg: &Message,
    content: String,
    keyboard: InlineKeyboardMarkup,
) -> ResponseResult<()> {
    match bot
        .edit_message_text(msg.chat.id, msg.id, content.clone())
        .reply_markup(keyboard.clone())
        .parse_mode(ParseMode::Html)
        .await
    {
        Ok(_) => {}
        Err(error) if is_not_modified_error(&error) => {}
        Err(_) => {
            bot.send_message(msg.chat.id, content)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}

pub(crate) async fn send_navigation_hint(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    bot.send_message(
        chat_id,
        as_html_card(
            "Next",
            "Continue from menu buttons below or use /help for full command list.",
        ),
    )
    .reply_markup(main_menu_keyboard())
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
