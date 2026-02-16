use teloxide::{prelude::*, utils::command::BotCommands};

use crate::config::Config;
use crate::system::run_cmd;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum MyCommands {
    #[command(description = "Show help menu.")]
    Help,
    #[command(description = "Check RAM and Disk usage.")]
    Status,
    #[command(description = "List open ports.")]
    Ports,
    #[command(description = "List running services.")]
    Services,
    #[command(description = "Show CPU usage.")]
    Cpu,
    #[command(description = "Show network statistics.")]
    Network,
    #[command(description = "Show system uptime.")]
    Uptime,
    #[command(description = "Show temperature sensors.")]
    Temp,
}

pub async fn answer(bot: Bot, msg: Message, cmd: MyCommands, config: &Config) -> ResponseResult<()> {
    if !config.authorized_users.contains(&(msg.chat.id.0 as u64)) {
        return Ok(());
    }

    match cmd {
        MyCommands::Help => {
            bot.send_message(msg.chat.id, MyCommands::descriptions().to_string())
                .await?;
        }
        MyCommands::Status => {
            let ram = run_cmd("free", &["-h"]).await;
            let disk = run_cmd("df", &["-h", "/"]).await;
            bot.send_message(
                msg.chat.id,
                format!(
                    "📊 **System Status**\n\n🧠 **RAM:**\n{}\n💾 **Disk:**\n{}",
                    ram, disk
                ),
            )
            .await?;
        }
        MyCommands::Ports => {
            let ports = run_cmd("ss", &["-tuln"]).await;
            bot.send_message(msg.chat.id, format!("🔌 **Open Ports:**\n```\n{}\n```", ports))
                .await?;
        }
        MyCommands::Services => {
            let services = run_cmd(
                "systemctl",
                &[
                    "list-units",
                    "--type=service",
                    "--state=running",
                    "--no-pager",
                ],
            )
            .await;
            let short = services
                .lines()
                .filter(|line| line.contains(".service"))
                .take(10)
                .collect::<Vec<_>>()
                .join("\n");
            bot.send_message(
                msg.chat.id,
                format!("⚙️ **Active Services:**\n```\n{}\n```", short),
            )
            .await?;
        }
        MyCommands::Cpu => {
            let cpu = run_cmd("top", &["-bn1"]).await;
            let short = cpu
                .lines()
                .filter(|line| line.contains("Cpu(s)"))
                .collect::<Vec<_>>()
                .join("\n");
            bot.send_message(msg.chat.id, format!("🖥️ **CPU Usage:**\n```\n{}\n```", short))
                .await?;
        }
        MyCommands::Network => {
            let net = run_cmd("ip", &["-s", "link"]).await;
            bot.send_message(
                msg.chat.id,
                format!("🌐 **Network Statistics:**\n```\n{}\n```", net),
            )
            .await?;
        }
        MyCommands::Uptime => {
            let uptime = run_cmd("uptime", &[]).await;
            bot.send_message(msg.chat.id, format!("⏱️ **System Uptime:**\n```\n{}\n```", uptime))
                .await?;
        }
        MyCommands::Temp => {
            let temp = run_cmd("sensors", &[]).await;
            bot.send_message(
                msg.chat.id,
                format!("🌡️ **Temperature Sensors:**\n```\n{}\n```", temp),
            )
            .await?;
        }
    }

    Ok(())
}