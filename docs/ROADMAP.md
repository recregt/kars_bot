# Kars Bot — Development Roadmap (2026-02-17)

This roadmap is rewritten in English and structured with checkboxes so implementation progress can be tracked step by step.

## Current Baseline

- [x] Core monitoring/alert loop is implemented and tested.
- [x] `/graph`, `/export`, and `/recent` are available.
- [x] JSONL-based anomaly storage + retention maintenance exists.
- [x] Test baseline is green (`cargo test`: 19/19).
- [ ] Hot-reload currently applies only graph-related runtime config.
- [ ] Preflight is strict and can block startup on non-systemd/non-sensors hosts.
- [ ] `/status` is mostly static and not fully runtime-aware.

---

## Confirmed Decisions (Accepted)

- [x] **Environment-aware feature degrade**: introduce a `CapabilityManager` and runtime capability flags (e.g., `has_sensors`, `is_systemd`) so missing host tools do not crash the bot.
- [x] **Hybrid persistence strategy**: keep raw anomaly events in JSONL, add a lightweight reporting store (SQLite or sled) for efficient historical summaries and weekly reporting.
- [x] **Thread-safe runtime config updates**: move effective runtime config behind `Arc<RwLock<...>>` so hot-reload can safely update alert thresholds and timing-related values.

---

## Sprint 1 — Stabilization (P0)

### Goal
Improve production reliability before adding larger features.

### Tasks
- [x] Implement `CapabilityManager` at startup.
- [x] Detect host capabilities via `std::process::Command` checks.
- [x] Persist capabilities in app context (`Capabilities` struct).
- [x] Update command handlers to return “feature not supported on this system” when capability is missing.
- [x] Refactor preflight from fail-fast to degrade mode.
- [x] Make `/status` show live runtime state (graph runtime, anomaly config, last tick, mute state, capabilities).
- [x] Clean Clippy `collapsible_if` warnings by flattening nested conditionals.

### Acceptance Criteria
- [x] Bot starts on hosts without `systemctl` and/or `sensors`.
- [x] Unsupported features fail gracefully with clear user-facing messages.
- [x] `/status` reflects real runtime state.
- [x] `cargo clippy --all-targets --all-features -D warnings` passes.

---

## Sprint 2 — Runtime Config + Safety (P0/P1)

### Goal
Enable safe dynamic behavior changes without restarts.

### Tasks
- [x] Introduce shared runtime config container (`Arc<RwLock<RuntimeConfig>>`).
- [x] Define which values are hot-reloadable (`alerts`, `monitor_interval`, `command_timeout_secs`, graph settings).
- [x] Update monitor loop to read latest runtime config each tick.
- [x] Update command timeout logic to read live timeout values.
- [x] Add clear logs for hot-reload apply/reject decisions.
- [x] Add tests for hot-reload race safety and value visibility.

### Acceptance Criteria
- [x] Runtime config changes are reflected without restart.
- [ ] No data races or lock-related regressions in async tasks.
- [x] Invalid config updates are rejected safely with clear logs.

---

## Sprint 3 — Data & Reporting Depth (P1)

### Goal
Make weekly and historical reporting accurate and efficient.

### Tasks
- [x] Add lightweight reporting store (SQLite or sled) for aggregated windows.
- [ ] Keep JSONL as source of raw event truth (append-only).
- [ ] Write/update summary records for rolling 7-day analytics.
- [x] Make weekly report resilient to process restarts.
- [x] Extend `/recent` query grammar for combined filters (example: `cpu>85 ram>80 6h`).
- [x] Improve parse error guidance with actionable examples.

### Acceptance Criteria
- [ ] Weekly report works after restart with consistent historical context.
- [x] Complex report queries are served without scanning full raw history each time.
- [x] Parser test coverage for combined filters is added.

---

## Sprint 4 — Simulation & UX Improvements (P1/P2)

### Goal
Improve validation workflows and mobile usability.

### Tasks
- [x] Add simulation mode for synthetic metrics (sin wave + random spikes).
- [x] Support a config toggle and/or command-level simulation switch.
- [x] Verify `/graph` and anomaly detection behavior using simulated data.
- [x] Add Telegram inline keyboard shortcuts for common actions.
- [x] Define safe, non-destructive button actions first (status, graph, mute/unmute).

### Acceptance Criteria
- [x] Simulation mode can be enabled without changing production logic paths.
- [ ] Inline actions reduce manual command typing for routine operations.

---

## Sprint 5 — Portability & Release Operations (P2)

### Goal
Make deployment simpler across Linux environments.

### Tasks
- [x] Add optional static build target (`x86_64-unknown-linux-musl`).
- [x] Document binary portability trade-offs and feature constraints.
- [x] Add CI artifact build for portable release binaries.
- [x] Add glibc/musl runtime validation checklist template.
- [ ] Validate runtime behavior on glibc and musl environments.

### Acceptance Criteria
- [x] Portable binary build is reproducible and documented.
- [ ] Runtime checks and degraded features still behave predictably.

---

## Bonus Backlog

- [ ] Add self-update flow (`/update`) with release check + controlled restart.
- [ ] Add output-as-file fallback for oversized Telegram command outputs.
- [ ] Add optional redaction for sensitive command outputs (`services`, `ports`, `network`).

---

## Risks During Implementation

- [ ] **Capability drift risk**: command availability can differ by distro and container image.
- [ ] **Scheduler drift risk**: daily/weekly jobs may shift after long pauses/restarts.
- [ ] **Concurrency risk**: expanded hot-reload surface may introduce lock-contention patterns.
- [ ] **Data model risk**: hybrid storage adds synchronization/consistency responsibilities.
- [ ] **Telegram limit risk**: long outputs and dense status payloads can exceed message limits.

---

## Open Decision Items (Still Pending)

- [x] Choose reporting store: SQLite vs sled. (Selected: sled)
- [ ] Decide simulation UX: config-only, command-only, or both.
- [ ] Scope inline actions: read-only first vs controlled admin actions.
- [ ] Decide if multi-user authorization (roles) is in-scope for near-term sprints.
- [ ] Define security posture for potentially sensitive system command outputs.

---

## Next Implementation Batch

- [x] Build `Capabilities` + `CapabilityManager` and wire into app context.
- [x] Convert preflight to degrade-mode startup behavior.
- [x] Upgrade `/status` into runtime-aware diagnostics.
- [x] Apply first Clippy cleanup pass (nested-if flattening).
