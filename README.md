# kars_bot

A Telegram server monitoring bot built with Rust + Teloxide.

## What It Does

- Monitors CPU, RAM, and Disk with `sysinfo`
- Sends threshold-based alerts with cooldown + hysteresis
- Supports alert muting (`/mute 30m`, `/unmute`)
- Provides health/status and system snapshot commands
- Stores anomalies in a local JSONL-based anomaly DB (`/recent` smart queries)
- Produces structured JSON logs for filtering and automation

## Quick Access

- Quick start and minimal config: [docs/quickstart.md](docs/quickstart.md)
- Operations and release flow: [docs/operations.md](docs/operations.md)
- Runtime behavior notes: [docs/runtime.md](docs/runtime.md)
- Project structure and modularity policy: [docs/project-structure.md](docs/project-structure.md)

## Local Automation

- Install hook manager: `scripts/install_hooks.sh`
- List task commands: `just --list`
- Full local quality gate: `just quality`
- Release prep (official flow): `just release vX.Y.Z`

## Documentation

- Documentation index: [docs/README.md](docs/README.md)
- Runbooks: [docs/runbooks/release.md](docs/runbooks/release.md)
- Architecture: [docs/architecture/overview.md](docs/architecture/overview.md)
- Detailed roadmap: [docs/ROADMAP.md](docs/ROADMAP.md)
- Generated references: [docs/reference/commands.md](docs/reference/commands.md), [docs/reference/config.md](docs/reference/config.md)