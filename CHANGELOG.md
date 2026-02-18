# Changelog

All notable changes to this project are documented in this file.

## v1.3.1 - 2026-02-18



### Bug Fixes

- Allow existing remote release tag in pre-push guard


### Maintenance

- Consolidate automation with just, lefthook, and reusable workflows


### merge

- Release v1.3.0 from feature branch

- Pre-push release tag guard fix

- Promote v1.3.0 and hook fixes from develop



## v1.3.0 - 2026-02-18



### Features

- Harden update orchestration and signal handling


### ci

- Fix quality gate fmt scope and auto-sync branches


### merge

- Release metadata v1.2.0

- Promote develop to main for v1.2.0

- Phase3 update orchestration and signal handling

- Phase3 validator and update-signal hardening

- Ci quality and branch sync automation

- Promote ci gate fixes



## v1.1.1 - 2026-02-18




## v1.0.1 - 2026-02-18




## v1.2.0 - 2026-02-18



### Features

- Add phase0 baselines and quality gates

- Phase1 rustls-only tls and dns probe

- Phase2 deterministic render and error integrity


### merge

- Phase0 reliability baselines and gates

- Phase1 rustls-only transition

- Phase2 graph determinism and error integrity



## v1.1.1 - 2026-02-18



### Documentation

- Modularize docs and add fail-safe validation pipeline


### Features

- Auto-clean merged feature branches via post-merge hook

- Add tag-driven musl release automation and server updater



## v1.1.0 - 2026-02-18



### Documentation

- Add glibc-musl validation checklist


### Features

- Add optional musl build workflow and docs

- Add inline quick-action keyboard

- Fallback oversized outputs as file attachments

- Add optional sensitive output redaction

- Add persistent 7-day rollup summaries

- Add release check and controlled restart command


### Maintenance

- Automate glibc-musl matrix validation

- Bump version to 1.1.0


### Tests

- Verify runtime apply and invalid config rejection

- Add concurrent config stress test and close roadmap


### merge

- Portable musl workflow into develop

- Runtime validation checklist and reload safety tests

- Inline quick actions for help

- Command output file fallback

- Optional sensitive output redaction

- Reporting rollups and append-only guarantees

- Runtime matrix automation and evidence

- Self-update command flow

- Runtime concurrency acceptance and roadmap closure

- Start v1.1.0 from develop

## v1.0.0 - 2026-02-17



### Refactors

- Split background jobs into focused modules (`jobs::config_reload`, `jobs::monitor`, `jobs::release_notify`, `jobs::schedules`)
- Extract recent query parser/filtering into dedicated modules for cleaner command flow

### Features

- Added release pipeline automation (git-cliff integration, test-gated tagging flow, owner release-note notification)
- Added CI release checks and musl artifact workflow for tagged builds
- Improved `/recent` query experience with combined filters and clearer query errors
- Added smart recent-query filters and mode-aware parsing
- Added command UX and anomaly DB modularization updates
- Added `/graph cpu` MVP with ring-buffer based short-window graphing
- Added hot-reloadable runtime graph settings via notify
- Added sled-backed reporting store and persisted weekly-report sample reads

### Fixes

- Fixed changelog prepend flow by using git-cliff unreleased mode

### Maintenance

- Added release/version guard automation in hooks and tagging scripts



## v0.6.0 - 2026-02-17



### Features

- Hierarchical anomaly journal and maintenance



## v0.5.0 - 2026-02-17



### Features

- Observability and owner-auth hardening



## v0.4.0 - 2026-02-17



### Features

- Modular architecture and daily summary reporting



## v0.3.0 - 2026-02-17



### Features

- Reliability, alert controls, and concurrency hardening



## v0.2.0 - 2026-02-17



### Features

- Production hardening and health monitoring



## v0.1.0 - 2026-02-16



### Maintenance

- Initial modular bot setup



