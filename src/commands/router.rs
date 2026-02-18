use teloxide::prelude::*;

use crate::app_context::AppContext;

use super::command_def::MyCommands;
use super::features::{
    alerts::{handle_alerts, handle_mute, handle_unmute},
    export::handle_export,
    graph::handle_graph,
    health::{handle_health, handle_help},
    recent::handle_recent_anomalies,
    status::handle_status_overview,
    system_info::{
        handle_cpu, handle_network, handle_ports, handle_services, handle_sys_status, handle_temp,
        handle_uptime,
    },
    update::handle_update,
};

pub(super) async fn route_command(
    bot: Bot,
    msg: Message,
    cmd: MyCommands,
    app_context: &AppContext,
) -> ResponseResult<()> {
    match cmd {
        MyCommands::Help => handle_help(&bot, &msg).await?,
        MyCommands::Status => handle_status_overview(&bot, &msg, app_context).await?,
        MyCommands::Sysstatus => handle_sys_status(&bot, &msg, app_context, &cmd).await?,
        MyCommands::Ports => handle_ports(&bot, &msg, app_context, &cmd).await?,
        MyCommands::Services => handle_services(&bot, &msg, app_context, &cmd).await?,
        MyCommands::Cpu => handle_cpu(&bot, &msg, app_context, &cmd).await?,
        MyCommands::Network => handle_network(&bot, &msg, app_context, &cmd).await?,
        MyCommands::Uptime => handle_uptime(&bot, &msg, app_context, &cmd).await?,
        MyCommands::Temp => handle_temp(&bot, &msg, app_context, &cmd).await?,
        MyCommands::Health => handle_health(&bot, &msg, app_context).await?,
        MyCommands::Alerts => handle_alerts(&bot, &msg, app_context).await?,
        MyCommands::Graph(query) => handle_graph(&bot, &msg, app_context, &query).await?,
        MyCommands::Export(query) => handle_export(&bot, &msg, app_context, &query).await?,
        MyCommands::Recent(query) => {
            handle_recent_anomalies(&bot, &msg, app_context, normalize_recent_query(&query)).await?
        }
        MyCommands::Mute(duration_str) => {
            handle_mute(&bot, &msg, app_context, &duration_str).await?
        }
        MyCommands::Unmute => handle_unmute(&bot, &msg, app_context).await?,
        MyCommands::Update(args) => handle_update(&bot, &msg, app_context, &args).await?,
    }

    Ok(())
}

fn normalize_recent_query(query: &str) -> Option<&str> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
