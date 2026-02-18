use teloxide::{prelude::*, types::ParseMode};

use crate::anomaly_db::recent_anomalies;
use crate::app_context::AppContext;

use super::super::helpers::as_html_block;
use super::recent_query::{RecentQuery, apply_recent_query, parse_recent_query};

pub(crate) async fn handle_recent_anomalies(
    bot: &Bot,
    msg: &Message,
    app_context: &AppContext,
    query: Option<&str>,
) -> ResponseResult<()> {
    const DEFAULT_LIMIT: usize = 10;
    const MAX_LIMIT: usize = 100;
    const SCAN_LIMIT: usize = 500;

    let parsed_query = match parse_recent_query(query) {
        Ok(parsed) => parsed,
        Err(error) => {
            bot.send_message(
                msg.chat.id,
                as_html_block(
                    "Recent anomalies",
                    &format!(
                        "Invalid query: {}\n\nUsage:\n/recent\n/recent 5\n/recent 6h\n/recent cpu>85\n/recent cpu>85 ram>80 6h",
                        error
                    ),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    let desired_limit = match parsed_query {
        RecentQuery::Default => DEFAULT_LIMIT,
        RecentQuery::Limit(limit) => limit,
        RecentQuery::Filters(_) => MAX_LIMIT,
    };

    let mut recent = recent_anomalies(&app_context.config, SCAN_LIMIT);
    recent = apply_recent_query(recent, parsed_query);
    if recent.len() > desired_limit {
        recent.truncate(desired_limit);
    }

    if recent.is_empty() {
        bot.send_message(
            msg.chat.id,
            as_html_block("Recent anomalies", "No anomaly records found."),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let lines = recent
        .iter()
        .enumerate()
        .map(|(index, event)| {
            format!(
                "{}. {} | CPU {:.1}% (>{:.1}%: {}) | RAM {:.1}% (>{:.1}%: {}) | Disk {:.1}% (>{:.1}%: {})",
                index + 1,
                event.timestamp,
                event.cpu,
                event.cpu_threshold,
                yes_no(event.cpu_over),
                event.ram,
                event.ram_threshold,
                yes_no(event.ram_over),
                event.disk,
                event.disk_threshold,
                yes_no(event.disk_over),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    bot.send_message(msg.chat.id, as_html_block("Recent anomalies", &lines))
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
