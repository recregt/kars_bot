# Command Reference

This file is generated from src/commands/command_def.rs.
Do not edit manually. Run scripts/generate_docs_reference.sh.

| Command | Has Args | Description |
|---|---|---|
| `/help` | No | Show help and usage examples. |
| `/status` | No | Show bot mode/capabilities (auth, storage, maintenance, retention). |
| `/health` | No | Show monitor liveness and loop delay. |
| `/sysstatus` | No | Check RAM and Disk usage snapshot. |
| `/cpu` | No | Show CPU usage. |
| `/temp` | No | Show temperature sensors. |
| `/network` | No | Show network statistics. |
| `/uptime` | No | Show system uptime. |
| `/services` | No | List running services. |
| `/ports` | No | List open ports. |
| `/recent` | Yes |  |
| `/graph` | Yes | Render metric graph. Usage: /graph cpu\|ram\|disk [30m\|1h\|6h\|24h] |
| `/export` | Yes |  |
| `/alerts` | No | Show alert thresholds and current alert states. |
| `/mute` | Yes | Mute alerts for a duration, e.g. /mute 30m |
| `/unmute` | No | Unmute alerts immediately. |
| `/update` | Yes | Release check and controlled restart. Usage: /update [check\|apply] |
