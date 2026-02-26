use teloxide::{
    RequestError,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::capabilities::Capabilities;
use crate::commands::helpers::as_html_card;

fn has_any_system_capability(capabilities: &Capabilities) -> bool {
    capabilities.has_free
        || capabilities.has_top
        || capabilities.has_sensors
        || capabilities.has_ip
        || capabilities.has_ss
        || capabilities.has_uptime
        || capabilities.is_systemd
}

pub(crate) fn main_menu_keyboard(capabilities: &Capabilities) -> InlineKeyboardMarkup {
    let mut rows = vec![vec![
        InlineKeyboardButton::callback("📊 Status", "cmd:status"),
        InlineKeyboardButton::callback("💓 Health", "cmd:health"),
        InlineKeyboardButton::callback("🚨 Alerts", "cmd:alerts"),
    ]];

    let mut second_row = vec![
        InlineKeyboardButton::callback("📈 Monitor", "menu:monitor"),
        InlineKeyboardButton::callback("📦 Data", "menu:data"),
    ];
    if has_any_system_capability(capabilities) {
        second_row.insert(
            0,
            InlineKeyboardButton::callback("🖥️ System", "menu:system"),
        );
    }
    rows.push(second_row);
    rows.push(vec![InlineKeyboardButton::callback("❓ Help", "menu:help")]);

    InlineKeyboardMarkup::new(rows)
}

fn system_menu_keyboard(capabilities: &Capabilities) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut resource_row: Vec<InlineKeyboardButton> = Vec::new();
    if capabilities.has_free {
        resource_row.push(InlineKeyboardButton::callback(
            "📦 Sys Snapshot",
            "cmd:sysstatus",
        ));
    }
    if capabilities.has_top {
        resource_row.push(InlineKeyboardButton::callback("🧠 CPU", "cmd:cpu"));
    }
    if capabilities.has_sensors {
        resource_row.push(InlineKeyboardButton::callback("🌡️ Temp", "cmd:temp"));
    }
    if !resource_row.is_empty() {
        rows.push(resource_row);
    }

    let mut network_row: Vec<InlineKeyboardButton> = Vec::new();
    if capabilities.has_ip {
        network_row.push(InlineKeyboardButton::callback("🌐 Network", "cmd:network"));
    }
    if capabilities.has_uptime {
        network_row.push(InlineKeyboardButton::callback("⏱️ Uptime", "cmd:uptime"));
    }
    if !network_row.is_empty() {
        rows.push(network_row);
    }

    let mut services_row: Vec<InlineKeyboardButton> = Vec::new();
    if capabilities.has_ss {
        services_row.push(InlineKeyboardButton::callback("🔌 Ports", "cmd:ports"));
    }
    if capabilities.is_systemd {
        services_row.push(InlineKeyboardButton::callback(
            "🧩 Services",
            "cmd:services",
        ));
    }
    if !services_row.is_empty() {
        rows.push(services_row);
    }

    rows.push(vec![InlineKeyboardButton::callback(
        "⬅️ Main Menu",
        "menu:main",
    )]);

    InlineKeyboardMarkup::new(rows)
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

fn menu_keyboard(menu_name: &str, capabilities: &Capabilities) -> Option<InlineKeyboardMarkup> {
    match menu_name {
        "main" => Some(main_menu_keyboard(capabilities)),
        "system" => Some(system_menu_keyboard(capabilities)),
        "monitor" => Some(monitor_menu_keyboard()),
        "data" => Some(data_menu_keyboard()),
        "help" => Some(main_menu_keyboard(capabilities)),
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
    capabilities: &Capabilities,
) -> ResponseResult<()> {
    let Some((title, description)) = menu_screen(menu_name) else {
        return Ok(());
    };
    let Some(keyboard) = menu_keyboard(menu_name, capabilities) else {
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
    capabilities: &Capabilities,
) -> ResponseResult<()> {
    let Some(keyboard) = menu_keyboard(menu_name, capabilities) else {
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

pub(crate) async fn send_navigation_hint(
    bot: &Bot,
    chat_id: ChatId,
    capabilities: &Capabilities,
) -> ResponseResult<()> {
    bot.send_message(
        chat_id,
        as_html_card(
            "Next",
            "Continue from menu buttons below or use /help for full command list.",
        ),
    )
    .reply_markup(main_menu_keyboard(capabilities))
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
