# Architecture Overview

## Runtime Model

- Telegram bot command handling is owner-only DM based.
- Background jobs run for monitor tick, config hot-reload, maintenance, release notifications, and summaries.
- Runtime config is shared via `Arc<RwLock<RuntimeConfig>>`.

## Data Model

- Raw anomaly events are persisted as append-only JSONL in `anomaly_db` (source of truth).
- Aggregated analytics are persisted in `reporting_store` (sled trees) for fast rolling summaries.
- Weekly reporting consumes persisted samples/rollups and falls back to in-memory history when needed.

## Safety Model

- Capability detection enables graceful degradation on hosts missing system tools.
- Versioning and release consistency are enforced by git hooks.
- Sensitive output redaction is optional and controlled via config (`security.redact_sensitive_output`).
- Oversized command outputs fall back to file attachments to avoid Telegram limit failures.
