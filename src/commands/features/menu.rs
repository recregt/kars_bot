use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::commands::helpers::as_html_block;

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

pub(crate) async fn handle_menu_navigation(
    bot: &Bot,
    msg: &Message,
    menu_name: &str,
) -> ResponseResult<()> {
    match menu_name {
        "main" => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Main Menu",
                    "Use buttons to run actions directly. No need to type commands manually.",
                ),
            )
            .reply_markup(main_menu_keyboard())
            .parse_mode(ParseMode::Html)
            .await?;
        }
        "system" => {
            bot.send_message(
                msg.chat.id,
                as_html_block("System Menu", "Live host diagnostics and service checks."),
            )
            .reply_markup(system_menu_keyboard())
            .parse_mode(ParseMode::Html)
            .await?;
        }
        "monitor" => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Monitor Menu",
                    "Alert controls, graphs and anomaly drill-down actions.",
                ),
            )
            .reply_markup(monitor_menu_keyboard())
            .parse_mode(ParseMode::Html)
            .await?;
        }
        "data" => {
            bot.send_message(
                msg.chat.id,
                as_html_block("Data Menu", "Recent anomalies and export shortcuts."),
            )
            .reply_markup(data_menu_keyboard())
            .parse_mode(ParseMode::Html)
            .await?;
        }
        "help" => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Quick Guide",
                    "1) Open a menu\n2) Tap an action\n3) Use ⬅️ Main Menu to continue\n\nTip: Slash commands still work, but all common flows are available as buttons.",
                ),
            )
            .reply_markup(main_menu_keyboard())
            .parse_mode(ParseMode::Html)
            .await?;
        }
        _ => {}
    }

    Ok(())
}

pub(crate) async fn send_navigation_hint(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    bot.send_message(
        chat_id,
        as_html_block(
            "Next",
            "Continue from menu buttons below or use /help for full command list.",
        ),
    )
    .reply_markup(main_menu_keyboard())
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
