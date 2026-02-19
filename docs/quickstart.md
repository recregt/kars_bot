# Quick Start

## Build and Run Locally

```bash
just build-release
./target/release/kars_bot
```

## Install Development Hooks

```bash
scripts/install_hooks.sh
```

## Developer Command Hub

```bash
just --list
just quality
just sync
just release-pr
```

## Minimal Configuration

Create `config.toml`:

```toml
bot_token = "123456:telegram-bot-token"
owner_id = 123456789
monitor_interval = 30
command_timeout_secs = 30

[alerts]
cpu = 85.0
ram = 90.0
disk = 90.0
cooldown_secs = 300
hysteresis = 5.0

[daily_summary]
enabled = true
hour_utc = 9
minute_utc = 0

[weekly_report]
enabled = false
weekday_utc = 1
hour_utc = 9
minute_utc = 0

[graph]
enabled = true
default_window_minutes = 60
max_window_hours = 24
max_points = 1200

[anomaly_db]
enabled = true
dir = "logs"
max_file_size_bytes = 10485760
retention_days = 7

[security]
redact_sensitive_output = false
```

`[anomaly_journal]` is also accepted as a backward-compatible alias.

## BotFather Commands

```text
help - Show help and usage examples
status - Show bot mode/capabilities
health - Show monitor liveness
sysstatus - Show RAM and Disk snapshot
cpu - Show CPU usage
temp - Show temperature sensors
network - Show network statistics
uptime - Show system uptime
services - List active services
ports - List open ports
recent - Smart recent query (5 | 6h | cpu>85)
graph - Metric chart (/graph cpu|ram|disk [30m|1h|6h|24h])
export - Export metric snapshot (/export cpu|ram|disk [30m|1h|6h|24h] [csv|json])
alerts - Show alert config/state
mute - Mute alerts (/mute 30m)
unmute - Unmute alerts
update - Release check and controlled restart (/update check | /update apply)
```