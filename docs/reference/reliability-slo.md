# Reliability SLO/SLI Specification (v1.2.0)

## Reliability Target

- Availability objective: **99.9%** monthly for command handling and polling runtime.
- Error budget: **0.1% monthly unavailability**.

## Service Level Indicators (SLIs)

- Bot availability SLI
  - Definition: proportion of successful polling cycles over total polling cycles.
  - Source: structured logs (`polling_ok`, `polling_error`) and restart events.
- Command success SLI
  - Definition: successful command responses / total accepted commands.
  - Scope: owner-authorized commands only.
- Graph render success SLI
  - Definition: successful graph render responses / total graph render attempts.
  - Includes `/graph` and weekly report graph generation paths.
- Update success SLI
  - Definition: successful update workflows with healthy service status / total `/update apply` attempts.

## SLO Thresholds

- Bot availability: >= 99.9%
- Command success: >= 99.5%
- Graph render success: >= 99.0%
- Update success: >= 99.5%

## Error Budget Policy

- Burn warning threshold: 30% budget consumed in first 7 days.
- Freeze threshold: >50% budget consumed in first 14 days.
- Mandatory release freeze when freeze threshold is crossed.
- Exception path requires owner approval and rollback plan.

## Release Gates

A release candidate is blocked unless all are true:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -D warnings`
- `cargo test`
- `quality / quality` check is green on PR
- release tag and Cargo version are aligned (`vX.Y.Z` == `Cargo.toml` version)

## Evidence Artifacts

- `docs/releases/runtime-validation-report.txt`
- `docs/releases/v1.2.0-baseline.md`
- `docs/releases/binary-size.csv`
