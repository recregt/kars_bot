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
```bash
scripts/install_hooks.sh
just --list
```

See [docs/quickstart.md](docs/quickstart.md) for full setup.

## Git Flow

1. `git switch -c feature/<name>`
2. Commit (`feat: add xxx`)
3. Open PR to `main`
4. Merge after CI passes
5. `just sync`
