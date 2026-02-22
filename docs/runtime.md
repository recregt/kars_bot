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
* **Self-Update Environment**: The `axoupdater` execution is wrapped in a thread-safe environment guard. The `INSTALLER_NO_MODIFY_PATH` variable must be set to `1` via an `unsafe` block before the updater run to bypass shell profile mutation attempts in hardened environments.
* **Service Sandboxing**: Runtime assumes a `ProtectSystem=strict` and `ProtectHome=true` state. Any persistent data or self-update binary swaps must occur within the explicitly allowed `ReadWritePaths` (typically `/opt/kars_bot/`).
* **Temporary Assets**: The `PrivateTmp=yes` directive in the systemd service ensures that update artifacts and temporary staging files are isolated and automatically purged on service restart/stop.
