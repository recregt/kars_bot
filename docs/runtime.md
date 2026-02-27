# Runtime Notes

* Authorization is single-owner only: only direct messages from `owner_id` are accepted.
* `/health` returns `Warming up` until the first monitor tick arrives.
* Daily summary runs once per day in UTC (`daily_summary.hour_utc`, `daily_summary.minute_utc`).
* Startup preflight checks currently validate `systemctl` and `sensors`.
* Release quality gates target `fmt + clippy + tests` and reliability artifacts.
* TLS policy for production is Rustls-only; OpenSSL should not appear in active dependency paths.
* MUSL deployments should validate DNS reachability to Telegram API in startup checks.
* Owner identity changes currently require restart (`systemctl restart kars-bot`).
* Anomaly DB layout under `dir`: `events/`, `index/`, `meta/`.
* Event files rotate by size; hourly maintenance prunes `events` and matching `index` day files.
* **Self-Update Environment**: The self-update mechanism downloads release archives directly via HTTPS from GitHub, verifies SHA256 checksums, extracts the binary, and performs an atomic swap to `/opt/kars_bot/bin/kars_bot`. No third-party installer subprocess is spawned. No shell profile files are accessed, making it fully compatible with `ProtectHome=true` and `ProtectSystem=strict` sandboxing. After swapping the binary, the bot triggers `systemctl restart kars-bot` which sends SIGTERM to the running process and starts the new binary.
* **Service Sandboxing**: Runtime assumes a `ProtectSystem=strict` and `ProtectHome=true` state. `ReadWritePaths` grants write access to `/opt/kars_bot/bin` (binary swap) and `/opt/kars_bot/data` (config, anomaly_db, reporting_store).
* **Restart Permission**: The `bot` user is authorized to restart `kars-bot.service` via a polkit rule (`/etc/polkit-1/rules.d/50-kars-bot-restart.rules`). No sudo or root escalation is needed.
* **Temporary Assets**: The `PrivateTmp=yes` directive in the systemd service ensures that update artifacts and temporary staging files are isolated and automatically purged on service restart/stop.
