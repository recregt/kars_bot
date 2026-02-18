# Runtime Notes

- Authorization is single-owner only: only direct messages from `owner_id` are accepted.
- `/health` returns `Warming up` until the first monitor tick arrives.
- Daily summary runs once per day in UTC (`daily_summary.hour_utc`, `daily_summary.minute_utc`).
- Startup preflight checks currently validate `systemctl` and `sensors`.
- Owner identity changes currently require restart (`systemctl restart kars-bot`).
- Anomaly DB layout under `dir`: `events/`, `index/`, `meta/`.
- Event files rotate by size; hourly maintenance prunes `events` and matching `index` day files.