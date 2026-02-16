use teloxide::prelude::*;

use crate::config::Config;
use crate::system::run_cmd;

pub async fn check_alerts(bot: &Bot, config: &Config) {
    let ram = run_cmd("free", &["-m"]).await;
    let disk = run_cmd("df", &["-h", "/"]).await;
    let cpu = run_cmd("top", &["-bn1"]).await;

    if let Some(line) = cpu.lines().find(|line| line.contains("Cpu(s)")) {
        if let Some(value) = line.split_whitespace().nth(1) {
            if let Ok(usage) = value.parse::<f32>() {
                if usage > config.alerts.cpu {
                    let _ = bot
                        .send_message(
                            ChatId(config.owner_id as i64),
                            format!("⚠️ ALERT: CPU usage is high ({:.1}%)", usage),
                        )
                        .await;
                }
            }
        }
    }

    if let Some(line) = disk.lines().find(|line| line.contains('/')) {
        if let Some(value) = line.split_whitespace().nth(4) {
            let percent = value.trim_end_matches('%');
            if let Ok(usage) = percent.parse::<f32>() {
                if usage > config.alerts.disk {
                    let _ = bot
                        .send_message(
                            ChatId(config.owner_id as i64),
                            format!("⚠️ ALERT: Disk usage is high ({}%)", usage),
                        )
                        .await;
                }
            }
        }
    }

    if let Some(line) = ram.lines().find(|line| line.contains("Mem:")) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(total), Ok(used)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>()) {
                let usage = (used / total) * 100.0;
                if usage > config.alerts.ram {
                    let _ = bot
                        .send_message(
                            ChatId(config.owner_id as i64),
                            format!("⚠️ ALERT: RAM usage is high ({:.1}%)", usage),
                        )
                        .await;
                }
            }
        }
    }
}