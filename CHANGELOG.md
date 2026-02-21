# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

## [1.7.0](https://github.com/recregt/kars_bot/compare/v1.6.4...v1.7.0) - 2026-02-21

### Added

- replace switch_inline_query buttons with callback buttons [#87]

## [1.6.4](https://github.com/recregt/kars_bot/compare/v1.6.3...v1.6.4) - 2026-02-20

### Fixed

- *(update)* harden checksum and archive extraction [#85] ([#85](https://github.com/recregt/kars_bot/pull/85) [#85](https://github.com/recregt/kars_bot/pull/85) )

## [1.6.3](https://github.com/recregt/kars_bot/compare/v1.6.2...v1.6.3) - 2026-02-20

### Fixed

- *(graph)* harden pure-rust embedded font registration [#83] ([#83](https://github.com/recregt/kars_bot/pull/83) [#83](https://github.com/recregt/kars_bot/pull/83) )

## [1.6.2](https://github.com/recregt/kars_bot/compare/v1.6.1...v1.6.2) - 2026-02-20

### Fixed

- *(graph)* remove font-dependent startup render path [#82] ([#82](https://github.com/recregt/kars_bot/pull/82) [#82](https://github.com/recregt/kars_bot/pull/82) )

### Other

- implement robust hard-sync logic for release-plz compatibility [#80]

## [1.6.1](https://github.com/recregt/kars_bot/compare/v1.6.0...v1.6.1) - 2026-02-20

### Fixed

- *(graph)* force embedded font on mesh labels [#78] ([#78](https://github.com/recregt/kars_bot/pull/78) [#78](https://github.com/recregt/kars_bot/pull/78) )

## [1.6.0](https://github.com/recregt/kars_bot/compare/v1.5.4...v1.6.0) - 2026-02-20

### Added

- reliability and infrastructure upgrade [#77]

### Other

- *(flow)* strict main-only PR workflow and release alignment [#75] ([#75](https://github.com/recregt/kars_bot/pull/75) [#75](https://github.com/recregt/kars_bot/pull/75) )

## [1.5.4](https://github.com/recregt/kars_bot/compare/v1.5.3...v1.5.4) - 2026-02-19

### Other

- *(release)* prepare release

## [1.5.3](https://github.com/recregt/kars_bot/compare/v1.5.2...v1.5.3) - 2026-02-19

### Added

- *(sync)* add labels, summary, and stronger auto-merge retry
- *(sync)* enable auto-merge for main-to-develop PR
- *(ci)* skip heavy checks for docs-only and reuse release assets
- *(release)* auto-fill GitHub release notes from changelog
- *(hooks)* optionally auto-create missing release tags on push
- *(phase3)* harden update orchestration and signal handling
- *(graph)* phase2 deterministic render and error integrity
- *(runtime)* phase1 rustls-only tls and dns probe
- *(reliability)* add phase0 baselines and quality gates
- *(release)* add tag-driven musl release automation and server updater
- *(git-flow)* auto-clean merged feature branches via post-merge hook
- *(update)* add release check and controlled restart command
- *(reporting)* add persistent 7-day rollup summaries
- *(security)* add optional sensitive output redaction
- *(commands)* fallback oversized outputs as file attachments
- *(help)* add inline quick-action keyboard
- *(portability)* add optional musl build workflow and docs
- hierarchical anomaly journal and maintenance
- observability and owner-auth hardening
- modular architecture and daily summary reporting
- reliability, alert controls, and concurrency hardening
- production hardening and health monitoring

### CI

- dispatch release workflow after auto-tag creation
- add concurrency guards to workflows
- *(dist)* fix manifest command invocation
- *(release)* streamline checks and extend release-plz/cargo-dist
- *(release)* install musl toolchain for cargo-dist build
- *(release-plz)* require dedicated token for PR-triggered checks
- *(release)* harden cargo-dist build invocation for tags
- fix release-plz and cargo-dist preview flows
- restore required aggregate check context name
- fix actionlint shellcheck redirects in reusable release
- modularize quality flows and add guarded release reuse
- fix quality gate fmt scope and auto-sync branches

### Documentation

- *(automation)* modularize docs and add fail-safe validation pipeline
- *(runtime)* add glibc-musl validation checklist

### Fixed

- *(ci)* exempt sync main->develop PRs from release label policy
- *(sync)* queue auto-merge on unstable state
- *(sync)* add scheduled retries for unstable PR auto-merge
- *(sync)* improve auto-merge reliability for sync PRs
- *(sync)* handle unstable state before enabling auto-merge
- *(sync)* run for bot merges and harden permissions
- *(hooks)* avoid false tag block on new feature branch push
- *(changelog)* regenerate history and harden release generation
- *(ci)* run rustfmt check with edition 2024
- *(ci)* use rustfmt for changed-file check
- *(ci)* pin actionlint to valid version
- allow dry-run preflight for existing tags
- harden lockfile and push/release guard flows
- *(ci)* checkout requested tag in reusable release
- *(ci)* pass explicit tag_name to release action
- *(ci)* harden release workflow invocation
- prevent pre-push hook stdin deadlock
- allow controlled main release commit flow
- *(hooks)* allow existing remote release tag in pre-push guard

### Other

- *(release)* bump version to 1.5.3 for graph font hotfix
- *(graph)* embed font for deterministic /graph rendering
- *(release)* prepare release
- *(release)* prepare release
- simplify flow and stabilize release-plz tagging
- *(ci)* reduce to minimal automation set
- *(sync)* simplify triggers and concurrency
- add just sync command
- harden flow checks for shallow repos and sync conflicts
- enforce strict local git flow and remove bypass paths
- extend release-plz and cargo-dist capabilities
- *(release)* prepare release
- *(release)* remove legacy flow and extend release-plz/cargo-dist
- *(release)* prepare release
- *(dist)* define cargo profile for release artifacts
- migrate production flow to release-plz and cargo-dist
- force fresh tag builds and add staged release tooling migration
- *(release)* bump version to v1.3.3 and refresh roadmap
- *(fmt)* format repository and auto-format staged Rust on commit
- *(ci)* optimize quality checks and stabilize branch sync
- *(git-flow)* block protected branch deletion pushes
- *(ci)* reduce dependabot and guard noise
- *(ci)* add dependabot and merge-queue-ready quality tooling
- *(release)* prepare v1.3.2
- add doctor diagnostics and release-safe automation
- *(ci)* add just ci and release preflight gate
- *(release)* prepare v1.3.1
- *(platform)* consolidate automation with just, lefthook, and reusable workflows
- *(release)* prepare v1.3.0
- *(release)* prepare v1.2.0
- *(release)* prepare v1.1.1
- harden git flow guards and modularize README docs
- *(release)* bump version to 1.1.0
- *(runtime)* add concurrent config stress test and close roadmap
- *(runtime)* automate glibc-musl matrix validation
- *(reload)* verify runtime apply and invalid config rejection
- squash commits from #7 onward
- initial modular bot setup

### Fixed

- *(graph)* embed render font into binary to avoid host missing-font outages

## [1.5.2](https://github.com/recregt/kars_bot/compare/v1.5.1...v1.5.2) - 2026-02-19

### CI

- dispatch release workflow after auto-tag creation

## [1.5.1](https://github.com/recregt/kars_bot/compare/v1.5.0...v1.5.1) - 2026-02-19

### Other

- simplify flow and stabilize release-plz tagging
- *(ci)* reduce to minimal automation set

## [1.5.0](https://github.com/recregt/kars_bot/compare/v1.4.1...v1.5.0) - 2026-02-19

### Added

- *(sync)* add labels, summary, and stronger auto-merge retry
- *(sync)* enable auto-merge for main-to-develop PR

### CI

- add concurrency guards to workflows

### Fixed

- *(sync)* queue auto-merge on unstable state
- *(sync)* add scheduled retries for unstable PR auto-merge
- *(sync)* improve auto-merge reliability for sync PRs
- *(sync)* handle unstable state before enabling auto-merge
- *(sync)* run for bot merges and harden permissions

### Other

- add just sync command
- harden flow checks for shallow repos and sync conflicts
- enforce strict local git flow and remove bypass paths
- extend release-plz and cargo-dist capabilities

## [1.4.1](https://github.com/recregt/kars_bot/compare/v1.4.0...v1.4.1) - 2026-02-19

### CI

- *(dist)* fix manifest command invocation
- *(release)* streamline checks and extend release-plz/cargo-dist
- *(release)* install musl toolchain for cargo-dist build

### Other

- *(release)* remove legacy flow and extend release-plz/cargo-dist

## [1.4.0](https://github.com/recregt/kars_bot/compare/v1.3.3...v1.4.0) - 2026-02-19

### Added

- *(ci)* skip heavy checks for docs-only and reuse release assets
- *(release)* auto-fill GitHub release notes from changelog

### Other

- *(release-plz)* require dedicated token for PR-triggered checks
- *(dist)* define cargo profile for release artifacts
- *(release)* harden cargo-dist build invocation for tags
- migrate production flow to release-plz and cargo-dist
- fix release-plz and cargo-dist preview flows
- force fresh tag builds and add staged release tooling migration
- restore required aggregate check context name
- fix actionlint shellcheck redirects in reusable release
- modularize quality flows and add guarded release reuse



### Bug Fixes

- Prevent pre-push hook stdin deadlock

- Harden release workflow invocation

- Pass explicit tag_name to release action

- Checkout requested tag in reusable release

- Harden lockfile and push/release guard flows

- Allow dry-run preflight for existing tags

- Pin actionlint to valid version

- Use rustfmt for changed-file check

- Run rustfmt check with edition 2024


### Maintenance

- Add dependabot and merge-queue-ready quality tooling

- Reduce dependabot and guard noise

- Bump rhysd/actionlint in the actions-all group

- Block protected branch deletion pushes

- Trigger required checks

- Merge develop into main sync branch

- Optimize quality checks and stabilize branch sync

- Format repository and auto-format staged Rust on commit


### merge

- Main back into develop after v1.3.2

- Pre-push stdin deadlock fix

- Develop into main after pre-push fix

- Main back into develop after pre-push fix

- Release workflow recovery

- Promote release workflow recovery

- Sync main back after release workflow recovery

- Release dispatch tag_name fix

- Promote release dispatch tag_name fix

- Sync main back after release dispatch fix

- Release checkout tag fix

- Promote release checkout tag fix

- Sync main back after release checkout tag fix

- Lockfile and guard hardening

- Promote lockfile and guard hardening

- Release preflight existing-tag fix

- Promote release preflight existing-tag fix

- Tooling automation bundle

- Promote tooling automation bundle



## [1.3.3] - 2026-02-19



### Bug Fixes

- Repair corrupted changelog ordering and duplicate section drift


### Maintenance

- Regenerate changelog deterministically in release flow and reject duplicate version headers

- Add fingerprint-aware lockfile security audit to pre-commit hooks

- Add confirmation gates for release recipes and formatter diagnostics helper

- Update roadmap progress and release examples for v1.3.3



## [1.3.2] - 2026-02-18



### Bug Fixes

- Allow controlled main release commit flow


### Maintenance

- Add just ci and release preflight gate

- Add doctor diagnostics and release-safe automation

- Prepare v1.3.2


### merge

- Platform automation consolidation and release v1.3.1

- Promote v1.3.1 platform standardization

- Local ci preflight automation

- Local ci preflight tooling

- Release automation doctor improvements

- Develop into main for v1.3.2

- Controlled main release guard fix

- Develop into main for v1.3.2 release guard



## [1.3.1] - 2026-02-18



### Bug Fixes

- Allow existing remote release tag in pre-push guard


### Maintenance

- Consolidate automation with just, lefthook, and reusable workflows

- Prepare v1.3.1


### merge

- Release v1.3.0 from feature branch

- Pre-push release tag guard fix

- Promote v1.3.0 and hook fixes from develop



## [1.3.0] - 2026-02-18



### Features

- Harden update orchestration and signal handling


### Maintenance

- Prepare v1.3.0


### ci

- Fix quality gate fmt scope and auto-sync branches


### merge

- Release metadata v1.2.0

- Promote develop to main for v1.2.0

- Phase3 update orchestration and signal handling

- Phase3 validator and update-signal hardening

- Ci quality and branch sync automation

- Promote ci gate fixes



## [1.2.0] - 2026-02-18



### Features

- Add phase0 baselines and quality gates

- Phase1 rustls-only tls and dns probe

- Phase2 deterministic render and error integrity


### Maintenance

- Prepare v1.2.0


### merge

- Phase0 reliability baselines and gates

- Phase1 rustls-only transition

- Phase2 graph determinism and error integrity



## [1.1.1] - 2026-02-18



### Maintenance

- Prepare v1.1.1



## [1.0.1] - 2026-02-18



### Documentation

- Modularize docs and add fail-safe validation pipeline


### Features

- Auto-clean merged feature branches via post-merge hook

- Add tag-driven musl release automation and server updater



## [1.1.0] - 2026-02-18



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



## [1.0.0] - 2026-02-17



### Maintenance

- Squash commits from #7 onward



## [0.6.0] - 2026-02-17



### Features

- Hierarchical anomaly journal and maintenance



## [0.5.0] - 2026-02-17



### Features

- Observability and owner-auth hardening



## [0.4.0] - 2026-02-17



### Features

- Modular architecture and daily summary reporting



## [0.3.0] - 2026-02-17



### Features

- Reliability, alert controls, and concurrency hardening



## [0.2.0] - 2026-02-17



### Features

- Production hardening and health monitoring



## [0.1.0] - 2026-02-16



### Maintenance

- Initial modular bot setup



