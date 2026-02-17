use teloxide::utils::command::BotCommands;

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
    #[command(description = "Show bot health and monitor liveness.")]
    Health,
    #[command(description = "Show alert thresholds and current alert states.")]
    Alerts,
    #[command(description = "Show recent anomaly records from JSONL journal.")]
    Recentanomalies,
    #[command(description = "Mute alerts for a duration, e.g. /mute 30m")]
    Mute(String),
    #[command(description = "Unmute alerts immediately.")]
    Unmute,
}