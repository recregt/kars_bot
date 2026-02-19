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

## DNS Incident Checklist

- Confirm resolver state on host: `resolvectl status` or `cat /etc/resolv.conf`
- Validate outbound DNS quickly: `getent hosts api.telegram.org`
- Verify bot startup/runtime DNS probe logs for latency/error spikes
- If resolution is flaky, reduce command load and defer update/apply operations

## Graph Degradation Checklist

- Run `/graph cpu 1h` and `/health` to confirm render vs runtime isolation
- Check logs for `GRAPH_` error codes and render slot timeout events
- If graph path is degraded, confirm auto-disable behavior and communicate degraded mode

## Update Rollback Checklist

- Verify update precheck output (`/update check`) before any apply retry
- Confirm service health after apply/rollback: `systemctl is-active kars-bot`
- Ensure current binary and backup state under `/opt/kars_bot/target/release`
- Record rollback cause and corrective action in release notes

## Mitigation

- Use `/mute` as temporary alert-noise reduction.
- Disable optional risky features in config and hot-reload where supported.
- Roll back to previous release if user-facing behavior is degraded.

## Recovery Exit Criteria

- `/health` stable and monitor tick fresh.
- `/status` reflects expected runtime state.
- No sustained error spam in logs.
