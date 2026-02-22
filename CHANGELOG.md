# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

## [1.7.4](https://github.com/recregt/kars_bot/compare/v1.7.3...v1.7.4) - 2026-02-22

### Documentation

- document systemd hardening and update constraints, bump to v1.7.4

## [1.7.3](https://github.com/recregt/kars_bot/compare/v1.7.2...v1.7.3) - 2026-02-22

### Fixed

- *(update)* use INSTALLER_NO_MODIFY_PATH env var to bypass shell modification
- *(update)* disable path modification in axoupdater to fix permission errors

## [1.7.2](https://github.com/recregt/kars_bot/compare/v1.7.1...v1.7.2) - 2026-02-21

### CI

- *(deps)* bump the actions-all group with 2 updates

### Other

- *(deps)* enable major version updates for github-actions

## [1.7.1](https://github.com/recregt/kars_bot/compare/v1.7.0...v1.7.1) - 2026-02-21

### CI

- add main branch trigger and shared cache key [#89]

### Documentation

- refresh operations page and runbooks
- revise docs/README, roadmap and architecture overview
- refresh top‑level README content
- update CHANGELOG with latest entries

### Other

- ci improvements [#91]

## [1.7.0](https://github.com/recregt/kars_bot/compare/v1.6.4...v1.7.0) - 2026-02-21

### Added

- replace switch_inline_query buttons with callback buttons [#87]

## [1.6.4](https://github.com/recregt/kars_bot/compare/v1.6.3...v1.6.4) - 2026-02-20

### Fixed

- *(update)* harden checksum and archive extraction [#85]

## [1.6.3](https://github.com/recregt/kars_bot/compare/v1.6.2...v1.6.3) - 2026-02-20

### Fixed

- *(graph)* harden pure-rust embedded font registration [#83]

## [1.6.2](https://github.com/recregt/kars_bot/compare/v1.6.1...v1.6.2) - 2026-02-20

### Fixed

- *(graph)* remove font-dependent startup render path [#82]

## [1.6.1](https://github.com/recregt/kars_bot/compare/v1.6.0...v1.6.1) - 2026-02-20

### Fixed

- *(graph)* force embedded font on mesh labels [#78]

## [1.6.0](https://github.com/recregt/kars_bot/compare/v1.5.4...v1.6.0) - 2026-02-20

### Added

- reliability and infrastructure upgrade [#77]

## [1.5.4](https://github.com/recregt/kars_bot/compare/v1.5.3...v1.5.4) - 2026-02-19

### Other

- *(release)* prepare release

## [1.5.3](https://github.com/recregt/kars_bot/compare/v1.5.2...v1.5.3) - 2026-02-19

### Added

- *(sync)* add labels, summary, and stronger auto-merge retry
- *(ci)* skip heavy checks for docs-only and reuse release assets
- *(release)* auto-fill GitHub release notes from changelog
- *(hooks)* optionally auto-create missing release tags on push
- *(phase3)* harden update orchestration and signal handling
- *(graph)* phase2 deterministic render and error integrity
- *(runtime)* phase1 rustls-only tls and dns probe
- *(reliability)* add phase0 baselines and quality gates
- *(release)* add tag-driven musl release automation and server updater
- *(update)* add release check and controlled restart command
- *(reporting)* add persistent 7-day rollup summaries
- *(security)* add optional sensitive output redaction
- *(commands)* fallback oversized outputs as file attachments
- *(help)* add inline quick-action keyboard

### Fixed

- *(graph)* embed render font into binary to avoid host missing-font outages
- *(ci)* run rustfmt check with edition 2024
- allow dry-run preflight for existing tags
- harden lockfile and push/release guard flows
- prevent pre-push hook stdin deadlock
- allow controlled main release commit flow

## [1.5.2](https://github.com/recregt/kars_bot/compare/v1.5.1...v1.5.2) - 2026-02-19

### CI

- dispatch release workflow after auto-tag creation

## [1.5.1](https://github.com/recregt/kars_bot/compare/v1.5.0...v1.5.1) - 2026-02-19

### Other

- simplify flow and stabilize release-plz tagging

## [1.5.0](https://github.com/recregt/kars_bot/compare/v1.4.1...v1.5.0) - 2026-02-19

### Added

- *(sync)* enable auto-merge for main-to-develop PR

### Fixed

- *(sync)* improve auto-merge reliability for sync PRs

## [1.4.1](https://github.com/recregt/kars_bot/compare/v1.4.0...v1.4.1) - 2026-02-19

### CI

- *(dist)* fix manifest command invocation
- *(release)* install musl toolchain for cargo-dist build

## [1.4.0](https://github.com/recregt/kars_bot/compare/v1.3.3...v1.4.0) - 2026-02-19

### Added

- *(ci)* skip heavy checks for docs-only and reuse release assets
- *(release)* auto-fill GitHub release notes from changelog

### Fixed

- *(ci)* checkout requested tag in reusable release
- *(ci)* pass explicit tag_name to release action
- *(ci)* harden release workflow invocation

## [1.3.3] - 2026-02-19

### Fixed

- repair corrupted changelog ordering and duplicate section drift

## [1.3.2] - 2026-02-18

### Fixed

- allow controlled main release commit flow

## [1.3.1] - 2026-02-18

### Fixed

- allow existing remote release tag in pre-push guard

## [1.3.0] - 2026-02-18

### Added

- harden update orchestration and signal handling

## [1.2.0] - 2026-02-18

### Added

- add phase0 baselines and quality gates
- phase1 rustls-only tls and dns probe
- phase2 deterministic render and error integrity

## [1.1.0] - 2026-02-18

### Added

- add optional musl build workflow and docs
- add inline quick-action keyboard
- fallback oversized outputs as file attachments
- add optional sensitive output redaction
- add persistent 7-day rollup summaries
- add release check and controlled restart command

## [1.0.0] - 2026-02-17

### Added

- initial release
