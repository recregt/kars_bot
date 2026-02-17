# kars_bot

A Telegram server monitoring bot built with Rust + Teloxide.

## What It Does

- Monitors CPU, RAM, and Disk with `sysinfo`
- Sends threshold-based alerts with cooldown + hysteresis
- Supports alert muting (`/mute 30m`, `/unmute`)
- Provides health/status and system snapshot commands
- Stores anomalies in a local JSONL-based anomaly DB (`/recent` smart queries)
- Produces structured JSON logs for filtering and automation

## Quick Start

### 1) Build and run locally

```bash
cargo build --release
./target/release/kars_bot
```

### 2) Minimal `config.toml`

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
```

`[anomaly_journal]` is also accepted as a backward-compatible alias.

### 3) Set BotFather commands

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
```

## Operations

### Versioning Guard + Release Tag Flow

- This repo blocks accidental `Cargo.toml` version edits in normal commits via `.githooks/pre-commit`.
- A pre-push guard (`.githooks/pre-push`) also validates tag/version consistency.
- Install hooks once per clone:

```bash
scripts/install_hooks.sh
```

- Create a release tag with version sync:

```bash
scripts/release_tag.sh v0.8.0
```

- Run release checks without mutations:

```bash
scripts/release_tag.sh --dry-run v0.8.0
```

Notes:
- The script runs `cargo test` before any release mutation.
- The script generates an English `CHANGELOG.md` section via `git-cliff`.
- The script logs binary size to `docs/releases/binary-size.csv`.
- The script bumps `Cargo.toml` only when needed.
- Version bump commit uses `ALLOW_VERSION_BUMP=1` to pass the guard.
- If no tag/release is planned, `Cargo.toml` version must stay unchanged.

Prerequisite:

```bash
cargo install git-cliff
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

### Docker build (optional)

```bash
docker run --rm \
  -v "$PWD":/app \
  -w /app \
  rust:1.93 \
  bash -lc "cargo build --release"
```

### Portable Linux binary (musl, optional)

Build with static musl target:

```bash
scripts/build_musl.sh
```

Artifact path:

```text
target/x86_64-unknown-linux-musl/release/kars_bot
```

Manual equivalent:

```bash
rustup target add x86_64-unknown-linux-musl
sudo apt-get update && sudo apt-get install -y musl-tools
cargo build --release --target x86_64-unknown-linux-musl
```

Portability notes:
- `musl` binaries are usually more portable across Linux distributions than default `glibc` builds.
- Host tooling still affects command behavior (`systemctl`, `sensors`, `ss`, etc.); unsupported features degrade gracefully.
- Some environments can still differ in kernel/cgroup visibility, so validate `/status`, `/health`, `/sysstatus`, and `/graph` on target host.
- Runtime validation checklist: `docs/releases/runtime-validation-checklist.md`

## Logging

- Logging output is JSON by default and can be filtered with `RUST_LOG`.
- Monitor loop emits structured fields: `cpu`, `ram`, `disk`, `cpu_over`, `ram_over`, `disk_over`.

```bash
RUST_LOG=info ./target/release/kars_bot
```

```bash
RUST_LOG=info ./target/release/kars_bot | jq 'select(.target == "monitor" and .fields.cpu > 80)'
```

## Runtime Notes

- Authorization is single-owner only: only direct messages from `owner_id` are accepted.
- `/health` returns `Warming up` until the first monitor tick arrives.
- Daily summary runs once per day in UTC (`daily_summary.hour_utc`, `daily_summary.minute_utc`).
- Startup preflight checks currently validate `systemctl` and `sensors`.
- Owner identity changes currently require restart (`systemctl restart kars-bot`).
- Anomaly DB layout under `dir`: `events/`, `index/`, `meta/`.
- Event files rotate by size; hourly maintenance prunes `events` and matching `index` day files.

## Project Structure

- `src/main.rs`: startup, config validation, preflight checks, task wiring
- `src/config.rs`: config schema + validation
- `src/system.rs`: command execution with timeout/error model
- `src/anomaly_db/`: anomaly model, write/read, retention maintenance
- `src/commands/`: command definitions, router, helpers, feature handlers
- `src/monitor/`: metrics provider, evaluation logic, monitor service

## Code Modularity Policy

- Any Rust source file crossing `200` lines must be split into a folder module (`feature/mod.rs` + focused submodules).
- Any file containing 3 distinct responsibilities (for example: data collection, processing, exporting) must be split similarly.
- The pre-commit hook blocks commits that stage `.rs` files above `200` lines.

## Roadmap

Detailed roadmap: [docs/ROADMAP.md](docs/ROADMAP.md)