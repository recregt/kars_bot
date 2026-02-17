# kars_bot

A Telegram server monitoring bot built with Rust + Teloxide.

## Features

- System monitoring with `sysinfo` (CPU, RAM, Disk)
- Hysteresis + cooldown alerting to reduce alert flapping/spam
- Time-bound alert mute controls (`/mute 30m`, `/unmute`)
- Health/liveness check command (`/health`)
- Alert state/config overview command (`/alerts`)
- Scheduled daily summary report (UTC, default 09:00)
- Safe command output handling (HTML escaping + truncation + line limiting)
- Command concurrency protection via semaphore

## Architecture

The project is organized into focused modules:

- `src/main.rs`: startup, config validation, preflight checks, task wiring
- `src/config.rs`: config schema + validation
- `src/system.rs`: command execution with timeout/error model
- `src/commands/`:
  - `command_def.rs`: Telegram command enum
  - `helpers.rs`: formatting/auth/timeout helpers
  - `handler.rs`: command dispatch and command handlers
- `src/monitor/`:
  - `provider.rs`: metrics provider trait + `sysinfo` implementation
  - `state.rs`: alert state and snapshots
  - `evaluator.rs`: threshold/cooldown/hysteresis evaluation logic
  - `service.rs`: monitor orchestration, mute/unmute, snapshots

## Installation

### Local build

```bash
cargo build --release
./target/release/kars_bot
```

### Docker (build in container)

```bash
docker run --rm \
  -v "$PWD":/app \
  -w /app \
  rust:1.93 \
  bash -lc "cargo build --release"
```

Then run the produced binary from host:

```bash
./target/release/kars_bot
```

### systemd service

Create `/etc/systemd/system/kars-bot.service`:

```ini
[Unit]
Description=Kars Telegram Monitoring Bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/opt/kars_bot
ExecStart=/opt/kars_bot/target/release/kars_bot
Restart=always
RestartSec=5
User=bot
Group=bot

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now kars-bot
sudo systemctl status kars-bot
```

## Configuration Schema (`config.toml`)

```toml
bot_token = "123456:telegram-bot-token"
owner_id = 123456789

# Allowed users (required)
allowed_user_ids = [123456789]

# Optional: allowed groups/channels (chat IDs, usually negative for groups)
allowed_chat_ids = [-1001234567890]

# Monitor loop interval (seconds, minimum 10)
monitor_interval = 30

# Slow command timeout (seconds)
command_timeout_secs = 30

[alerts]
cpu = 85.0
ram = 90.0
disk = 90.0

# Alert spam control
cooldown_secs = 300
hysteresis = 5.0

[daily_summary]
enabled = true
hour_utc = 9
minute_utc = 0
```

## Notes

- `allowed_user_ids` also supports legacy alias `authorized_users`.
- `/health` returns `Warming up` until the first monitor tick arrives.
- Daily summary runs once per day in UTC based on `daily_summary.hour_utc` and `daily_summary.minute_utc`.
- External command preflight checks currently validate `systemctl` and `sensors` at startup.
