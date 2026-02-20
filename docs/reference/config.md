# Config Reference

This file is generated from src/config/schema.rs.
Do not edit manually.

## Config

| Field | Type | Default | Aliases |
|---|---|---|---|
| `bot_token` | `String` | No | - |
| `owner_id` | `u64` | No | - |
| `monitor_interval` | `u64` | Yes | - |
| `command_timeout_secs` | `u64` | Yes | - |
| `alerts` | `Alerts` | Yes | - |
| `daily_summary` | `DailySummary` | Yes | - |
| `weekly_report` | `WeeklyReport` | Yes | - |
| `graph` | `Graph` | Yes | - |
| `anomaly_db` | `AnomalyDb` | Yes | anomaly_journal |
| `simulation` | `Simulation` | Yes | - |
| `reporting_store` | `ReportingStoreConfig` | Yes | - |
| `release_notifier` | `ReleaseNotifierConfig` | Yes | - |
| `security` | `Security` | Yes | - |

## RuntimeConfig

| Field | Type | Default | Aliases |
|---|---|---|---|
| `alerts` | `Alerts` | No | - |
| `monitor_interval` | `u64` | No | - |
| `command_timeout_secs` | `u64` | No | - |
| `graph` | `Graph` | No | - |

## Alerts

| Field | Type | Default | Aliases |
|---|---|---|---|
| `cpu` | `f32` | Yes | - |
| `ram` | `f32` | Yes | - |
| `disk` | `f32` | Yes | - |
| `cooldown_secs` | `u64` | Yes | - |
| `hysteresis` | `f32` | Yes | - |

## DailySummary

| Field | Type | Default | Aliases |
|---|---|---|---|
| `enabled` | `bool` | Yes | - |
| `hour_utc` | `u8` | Yes | - |
| `minute_utc` | `u8` | Yes | - |

## WeeklyReport

| Field | Type | Default | Aliases |
|---|---|---|---|
| `enabled` | `bool` | Yes | - |
| `weekday_utc` | `u8` | Yes | - |
| `hour_utc` | `u8` | Yes | - |
| `minute_utc` | `u8` | Yes | - |

## Graph

| Field | Type | Default | Aliases |
|---|---|---|---|
| `enabled` | `bool` | Yes | - |
| `default_window_minutes` | `u64` | Yes | - |
| `max_window_hours` | `u64` | Yes | - |
| `max_points` | `u16` | Yes | - |

## AnomalyDb

| Field | Type | Default | Aliases |
|---|---|---|---|
| `enabled` | `bool` | Yes | - |
| `dir` | `String` | Yes | - |
| `max_file_size_bytes` | `u64` | Yes | - |
| `retention_days` | `u16` | Yes | - |

## Simulation

| Field | Type | Default | Aliases |
|---|---|---|---|
| `enabled` | `bool` | Yes | - |
| `profile` | `String` | Yes | - |

## ReportingStoreConfig

| Field | Type | Default | Aliases |
|---|---|---|---|
| `enabled` | `bool` | Yes | - |
| `path` | `String` | Yes | - |
| `retention_days` | `u16` | Yes | - |

## ReleaseNotifierConfig

| Field | Type | Default | Aliases |
|---|---|---|---|
| `enabled` | `bool` | Yes | - |
| `changelog_path` | `String` | Yes | - |
| `state_path` | `String` | Yes | - |

## Security

| Field | Type | Default | Aliases |
|---|---|---|---|
| `redact_sensitive_output` | `bool` | Yes | - |

