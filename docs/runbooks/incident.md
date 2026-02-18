# Incident Runbook

## First 5 Minutes

- Capture failing command/context.
- Confirm bot process health.
- Check latest structured logs for monitor/runtime errors.
- Verify capability degrade warnings and config reload warnings.

## Triage Checklist

- Auth issues (`owner_id`, DM-only path)
- Runtime config hot-reload behavior
- System command capability availability
- Reporting store read/write errors
- Telegram output size or API failures

## Mitigation

- Use `/mute` as temporary alert-noise reduction.
- Disable optional risky features in config and hot-reload where supported.
- Roll back to previous release if user-facing behavior is degraded.

## Recovery Exit Criteria

- `/health` stable and monitor tick fresh.
- `/status` reflects expected runtime state.
- No sustained error spam in logs.
