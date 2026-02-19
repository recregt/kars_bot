# Kars Bot Reliability Mega-Roadmap (2026-02-19)

Target profile: 99.9% production reliability on GCP/VPS with a MUSL-target static binary.
Stack context: Teloxide, Tokio, Sled, Plotters, Sysinfo, Linux systemd service runtime.

v1.3.3 implementation status: Phase 0, Phase 1, and Phase 2 are completed. Phase 4 operator-safety controls are partially completed.

Latest update highlights (v1.3.3):
- [x] Release changelog generation is now deterministic and corruption-guarded.
- [x] Hook/CI release safety controls were tightened (fingerprint-aware lockfile audit, confirm-gated release commands).
- [x] Branch-sync and release runbook flow are aligned with current protected-branch policy.

---

## Stage 1 — Precise Technical Translation (Scope-Preserved)

### 0) Baseline and Scope Freeze (P0)

#### Goal
Establish measurable baselines and freeze non-goals before refactoring.

#### Tasks
- [ ] Capture runtime and binary baselines:
  - [ ] Dependency inversion check for crypto/TLS stack (transitive dependency walk).
  - [ ] Dynamic-link inspection on glibc target.
  - [ ] Static-link artifact inspection on MUSL target.
- [ ] Capture representative error logs for graph rendering and update flow.
- [ ] Freeze explicit out-of-scope items (new commands, non-essential UI work, feature creep).

#### Acceptance Criteria
- [ ] Baseline evidence is persisted in release validation documents.
- [ ] Each future change is traceable to one root reliability issue.

---

### 1) Remove Ghost C Dependency (OpenSSL) (P0)

#### Problem
The project includes vendored OpenSSL while the effective Telegram/TLS transport uses Rustls. This creates unnecessary C toolchain coupling, increases build time, and weakens MUSL portability.

#### Goal
Enforce a Rust-native TLS path and eliminate unnecessary OpenSSL transitive paths.

#### Tasks
- [ ] Remove direct OpenSSL dependency from the manifest.
- [ ] Keep only required Rustls feature set in Telegram client dependencies.
- [ ] Audit transitive dependency graph for native-tls/OpenSSL re-introduction.
- [ ] Validate MUSL release build path and CI assumptions.
- [ ] Update runtime/release documentation to state Rustls-only TLS policy.

#### Acceptance Criteria
- [ ] OpenSSL is absent from the active dependency graph unless explicitly justified.
- [ ] Release and MUSL builds pass without C crypto toolchain dependency.

---

### 2) Font Gate and Silent Graph Failures (P0)

#### Problem
A statically linked artifact may not access host font libraries. Plotters can panic or fail silently when no valid font path exists.

#### Goal
Introduce deterministic font lifecycle and remove silent render failure modes.

#### Tasks
- [ ] Define deterministic font strategy (embedded asset and/or shipped font bundle).
- [ ] Add explicit font resolution error classification.
- [ ] Add startup font readiness preflight.
- [ ] Add degraded behavior when font readiness fails.
- [ ] Improve graph command UX with actionable operator/user messages.

#### Acceptance Criteria
- [ ] Missing-font scenarios do not panic and do not fail silently.
- [ ] Error cause and remediation are visible to both operator and user.

---

### 3) Spawn-Blocking Error Propagation and Diagnostics (P0)

#### Problem
Graph rendering errors inside Tokio spawn_blocking are not consistently propagated with context. With panic=abort on release profile, stack traces are limited and failures can appear swallowed.

#### Goal
Map blocking-render failures into typed domain errors with structured telemetry.

#### Tasks
- [ ] Introduce typed GraphRenderError taxonomy.
- [ ] Normalize JoinError and inner render failures into one domain error model.
- [ ] Add panic boundary handling at render boundary where possible.
- [ ] Add deterministic timeout and queue/slot visibility for render jobs.
- [ ] Add test coverage for panic, timeout, join, backend, and font-failure branches.

#### Acceptance Criteria
- [ ] Every rendering failure leaves a user-visible outcome and operator-visible structured event.
- [ ] No silent error drop paths remain in graph execution.

---

### 4) End-to-End Update Flow Automation (P0/P1)

#### Problem
The Rust command path and Bash update script are not fully orchestrated; privilege checks and service-state checks remain partly manual.

#### Goal
Provide a one-command, policy-safe update flow with health validation and rollback.

#### Tasks
- [ ] Extend update check to include operational feasibility checks.
- [x] Route update apply through controlled script execution.
- [x] Harden updater with preconditions, verification, and Atomic Swap deployment.
- [x] Add service restart health gate and automatic rollback on failure.
- [x] Document minimum-privilege sudoers and non-systemd degraded behavior.

#### Acceptance Criteria
- [ ] Update apply provides deterministic success/rollback/manual-intervention states.
- [x] Service liveness is automatically validated after update.

---

## Stage 2 — Deep-Dive Gap Analysis (Architect Audit)

### A) DNS and NSS on MUSL-Static

#### Finding
- [ ] A fully static MUSL binary does not rely on glibc NSS modules, and resolver behavior can differ from glibc-based expectations.
- [ ] Telegram API reachability depends on robust DNS under container/VPS/network policy constraints.

#### Risk
- [ ] Resolver edge cases (split DNS, search domains, transient resolver outages, IPv6 preference mismatch) can cause intermittent API failures.

#### Required Additions
- [ ] Decide resolver strategy explicitly:
  - [ ] Option 1: Keep system resolver path and harden retry/backoff metrics.
  - [ ] Option 2: Integrate pure-Rust resolver path (for example Hickory/Trust-DNS resolver) with explicit upstreams and timeout policy.
- [ ] Add DNS readiness probe at startup and periodic runtime probe.
- [ ] Add resolver metrics: lookup latency, NXDOMAIN/SERVFAIL counters, fallback usage.
- [ ] Add runbook section for resolver incident triage.

### B) Concurrency and Deadlock/Starvation Paths

#### Finding
- [ ] Current graph path acquires metric history lock and render slot in sequence; lock hold windows are short but contention can still cause starvation under high command fan-in.
- [ ] Blocking render workload can increase queue wait and trigger cascading command timeout behavior.

#### Risk
- [ ] Throughput collapse (not a strict deadlock) under burst traffic, perceived as random graph failures.

#### Required Additions
- [ ] Introduce explicit lock-order policy and document it.
- [ ] Keep metric history lock strictly scoped to snapshot copy only.
- [ ] Add bounded wait telemetry for render slot acquisition.
- [ ] Add command-level circuit breaker/degrade path when queue depth exceeds threshold.

### C) Asset Lifecycle and include_bytes Footprint

#### Finding
- [ ] Embedding fonts via include_bytes moves assets into binary sections, increasing artifact size and first-touch page faults.

#### Risk
- [ ] Cold-start latency increase and potential I-cache/D-cache pressure on low-memory VPS.

#### Required Additions
- [ ] Benchmark two packaging modes: embedded font vs sidecar font package.
- [ ] Add startup timing probe (process start to first successful graph render).
- [ ] Use lazy initialization for font engine and cache resolved font handles.

### D) Signal Handling During Mid-Flight Update

#### Finding
- [ ] systemd restart sends SIGTERM then termination escalation; in-flight update tasks and Telegram polling loop can be interrupted.

#### Risk
- [ ] Partially completed update workflow, double restart attempts, or inconsistent status messaging.

#### Required Additions
- [ ] Define signal choreography:
  - [ ] Pre-stop gate: pause new update jobs.
  - [ ] Drain in-flight graph/update tasks with bounded timeout.
  - [ ] Persist update-state checkpoint before restart.
- [ ] Ensure server_update script and Rust flow are idempotent under repeated signals.

### E) Sled Database Integrity Across Atomic Binary Swap

#### Finding
- [ ] Binary Atomic Swap is safe for executable replacement, but Sled durability depends on flush semantics and clean shutdown timing.

#### Risk
- [ ] Unflushed writes or abrupt termination can cause data loss window or longer recovery path on restart.

#### Required Additions
- [ ] Add pre-restart storage barrier (flush and verify result).
- [ ] Add startup integrity check and explicit recovery log path.
- [ ] Add backup/restore hooks for reporting store before risky update operations.

---

## Stage 3 — Predictive Troubleshooting Layer (Failure Mode + Mitigation)

### Workstream: TLS and Dependency Hygiene
- [ ] Potential Failure Mode
  - [ ] Transitive crate reintroduces native-tls via default features after dependency update.
- [ ] Mitigation Strategy
  - [ ] Add CI gate that fails on OpenSSL/native-tls presence in dependency tree.
  - [ ] Pin feature sets and review dependency updates via lockfile diff policy.

### Workstream: Font and Render Stability
- [ ] Potential Failure Mode
  - [ ] Embedded font parse failure or corrupted sidecar font causes graph render outage.
- [ ] Mitigation Strategy
  - [ ] Add dual-font fallback chain and startup self-test with fail-fast diagnostics.
  - [ ] Auto-disable graph feature with explicit operator alert when readiness fails.

### Workstream: Blocking Render Error Propagation
- [ ] Potential Failure Mode
  - [ ] JoinError/timeout branches map to generic message, losing root-cause attribution.
- [ ] Mitigation Strategy
  - [ ] Enforce typed error envelope with stable error codes and event correlation ID.
  - [ ] Add synthetic fault injection tests for each branch.

### Workstream: Update and Restart Orchestration
- [ ] Potential Failure Mode
  - [ ] Service restarts with invalid binary or wrong permissions, causing boot-loop.
- [ ] Mitigation Strategy
  - [ ] Validate executable before swap, keep previous binary, and auto-rollback on health-check failure.
  - [ ] Use flock/lockfile to block concurrent update applies.

### Workstream: DNS Reliability
- [ ] Potential Failure Mode
  - [ ] Resolver outage causes Telegram API unreachability and command backlog.
- [ ] Mitigation Strategy
  - [ ] Resolver retries with jittered exponential backoff and multi-upstream fallback.
  - [ ] Degraded mode with clear status reporting and reduced command load.

### Workstream: Sled Durability
- [ ] Potential Failure Mode
  - [ ] Abrupt restart during write-heavy period leaves stale/incomplete state.
- [ ] Mitigation Strategy
  - [ ] Pre-restart flush barrier and post-restart consistency check with recovery path.
  - [ ] Scheduled snapshots for fast rollback and incident forensics.

---

## Stage 4 — Final Expanded Mega-Roadmap (Execution Plan)

## Phase 0 — Baseline, Invariants, and Reliability Gates (P0)

### Deliverables
- [x] Runtime baseline report for build, startup, graph render, update flow.
- [x] Reliability SLO map (99.9 target translated into measurable SLIs).
- [x] Reliability gates integrated into CI and pre-release checklist.

### Tasks
- [x] Define SLIs: bot availability, command success rate, graph success rate, update success rate.
- [x] Define error budget policy and release freeze criteria.
- [x] Add deterministic build metadata and artifact manifest.

### Potential Failure Mode
- [x] Baseline metrics are incomplete and cannot prove reliability gains.

### Mitigation Strategy
- [x] Reject release candidates that lack full SLI bundle and reproducibility metadata.

---

## Phase 1 — TLS, MUSL, and Dependency Hardening (P0)

### Deliverables
- [x] Rustls-only transport path.
- [x] No unintended OpenSSL/native-tls in transitive graph.
- [x] Stable MUSL static artifact validation path.

### Tasks
- [x] Remove direct OpenSSL dependency and verify lockfile impact.
- [x] Add dependency policy checks in CI.
- [x] Validate artifact properties:
  - [x] Static-PIE expectation and symbol stripping validation.
  - [x] Reproducible release profile settings review.
- [x] Validate DNS behavior under MUSL with production-like resolver settings.

### Potential Failure Mode
- [x] TLS handshake regressions appear after dependency cleanup.

### Mitigation Strategy
- [x] Add canary rollout with handshake/error-rate telemetry and immediate rollback trigger.

---

## Phase 2 — Graph Runtime Determinism and Error Integrity (P0)

### Deliverables
- [x] Deterministic font lifecycle.
- [x] Typed render error taxonomy with end-to-end propagation.
- [x] Queue/slot pressure visibility and overload protection.

### Tasks
- [x] Implement font readiness preflight and fallback policy.
- [x] Introduce GraphRenderError and stable error codes.
- [x] Bound render execution with timeout and queue depth metrics.
- [x] Ensure lock scope minimization for metric history snapshot operations.
- [x] Add test matrix for font missing, backend failure, timeout, join failure, panic boundary behavior.

### Potential Failure Mode
- [x] High-load periods exhaust render slots and trigger command timeout cascades.

### Mitigation Strategy
- [x] Apply adaptive backpressure (temporary graph cooldown, queue cap, degrade messaging).

---

## Phase 3 — Update Orchestration, Signal Safety, and Storage Integrity (P0/P1)

### Deliverables
- [x] Idempotent update workflow with prechecks, Atomic Swap, health validation, and rollback.
- [x] Signal-safe shutdown choreography compatible with systemd.
- [x] Sled durability guards for restart windows.

### Tasks
- [x] Extend update check to include permissions, service manager, path writability, and lock state.
- [ ] Harden server_update workflow:
  - [x] Verify downloaded artifact integrity and executability.
  - [x] Use Atomic Swap deployment with backup retention.
  - [x] Perform post-restart health probe and auto-rollback on failure.
- [x] Add signal-aware stop/drain logic for in-flight operations.
- [x] Add Sled flush barrier before controlled restart and startup consistency probe.
- [x] Add update lock (single-writer policy).

### Potential Failure Mode
- [x] Mid-flight restart interrupts update and leaves ambiguous service state.

### Mitigation Strategy
- [x] Persist update state machine checkpoints and resume/repair on startup.

---

## Phase 4 — Operationalization, Runbooks, and Release Control (P1)

### Deliverables
- [ ] End-to-end runbooks aligned with real failure paths.
- [x] Operator-safe commands and incident playbooks.
- [ ] Release validation matrix for glibc/MUSL and DNS variants.

### Tasks
- [x] Update release, rollback, and incident runbooks with concrete command sequences.
- [x] Add operator checklists for DNS incidents, graph subsystem degradation, and update rollback.
- [x] Add pre-release chaos checks (DNS fault injection, render failure injection, update rollback drills).

### Potential Failure Mode
- [ ] Corrective action is delayed because runbooks are incomplete or stale.

### Mitigation Strategy
- [x] Make runbook verification a mandatory release gate with ownership sign-off.

---

## Phase 5 — Performance and Observability (P2)

### Deliverables
- [ ] Full telemetry for command lifecycle, graph rendering, updates, DNS, and storage.
- [ ] Operator and user-facing UX improvements for error clarity.
- [ ] Performance baselines for cold start, steady state, and high-load behavior.

### Tasks
- [ ] Add structured telemetry dimensions:
  - [ ] trace_id, request_id, command_type, queue_wait_ms, render_ms, dns_lookup_ms, update_stage.
- [ ] Export counters/histograms for SLI dashboards and alerting.
- [ ] Add user-facing error envelope with short remediation guidance per error code.
- [ ] Benchmark and tune:
  - [ ] Cold start latency with embedded vs sidecar font assets.
  - [ ] Throughput under graph burst traffic and update contention.
- [ ] Add log sampling/rate limiting to avoid observability-induced load.

### Potential Failure Mode
- [ ] Observability overhead degrades runtime performance.

### Mitigation Strategy
- [ ] Use dynamic sampling, bounded label cardinality, and periodic overhead profiling.

---

## Global Definition of Done

- [ ] All P0 acceptance criteria are met with evidence artifacts.
- [ ] Reliability target trajectory is measurable (error budget burn visible per release).
- [ ] No silent failure path remains for graph and update critical flows.
- [ ] MUSL static artifact passes runtime validation in production-like conditions.
- [ ] DNS, update, and storage recovery runbooks are tested and operator-approved.

---

## Suggested Execution Order

- [ ] Step 1: Phase 0 baseline and reliability gates
- [ ] Step 2: Phase 1 dependency/TLS/MUSL hardening
- [ ] Step 3: Phase 2 graph determinism and error integrity
- [ ] Step 4: Phase 3 update, signal, and storage safety
- [ ] Step 5: Phase 4 runbook and release control
- [ ] Step 6: Phase 5 performance and observability
