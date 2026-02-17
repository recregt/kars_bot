use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum MyCommands {
    #[command(description = "Show help and usage examples.")]
    Help,
    #[command(description = "Show bot mode/capabilities (auth, storage, maintenance, retention).")]
    Status,
    #[command(description = "Show monitor liveness and loop delay.")]
    Health,

    #[command(description = "Check RAM and Disk usage snapshot.")]
    Sysstatus,
    #[command(description = "Show CPU usage.")]
    Cpu,
    #[command(description = "Show temperature sensors.")]
    Temp,
    #[command(description = "Show network statistics.")]
    Network,
    #[command(description = "Show system uptime.")]
    Uptime,

    #[command(description = "List running services.")]
    Services,
    #[command(description = "List open ports.")]
    Ports,

    #[command(description = "Smart recent query. Examples: /recent, /recent 5, /recent 6h, /recent cpu>85")]
    Recent(String),
    #[command(description = "Render metric graph. Usage: /graph cpu|ram|disk [30m|1h|6h|24h]")]
    Graph(String),
    #[command(description = "Export metric snapshot. Usage: /export cpu|ram|disk [30m|1h|6h|24h] [csv|json]")]
    Export(String),
    #[command(description = "Show alert thresholds and current alert states.")]
    Alerts,

    #[command(description = "Mute alerts for a duration, e.g. /mute 30m")]
    Mute(String),
    #[command(description = "Unmute alerts immediately.")]
    Unmute,
}