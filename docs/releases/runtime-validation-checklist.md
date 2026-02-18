# Runtime Validation Checklist (glibc + musl)

This checklist is used before promoting `develop` to `main` for `v1.2.x` and later.

## Scope

- Verify runtime behavior on both `glibc` and `musl` environments.
- Verify degraded capability handling when host tools are missing.
- Verify core operational commands remain stable after portability changes.

## Environment Matrix

| ID | Environment | Binary | Expected |
|---|---|---|---|
| A1 | Ubuntu/Debian (glibc) | `target/release/kars_bot` | Full feature set where host tools exist |
| A2 | Alpine/minimal (musl) | `target/x86_64-unknown-linux-musl/release/kars_bot` | Graceful degrade for unavailable host tools |

## Build Checks

### TLS dependency policy

```bash
scripts/check_tls_stack.sh
```

### glibc build

```bash
cargo build --release
```

### musl build

```bash
scripts/build_musl.sh
```

## Startup & Capability Checks

For each environment (A1, A2):

1. Start bot with valid `config.toml`.
2. Confirm process starts without panic.
3. Run `/status` and verify capability flags are reported.
4. Confirm unsupported system commands return clear degrade messages.

## Functional Smoke Checks

Run and verify:

- `/health`
- `/status`
- `/sysstatus`
- `/recent 6h`
- `/graph cpu 1h`
- `/export cpu 1h csv`

## Safety/Guard Checks

- Verify pre-commit blocks disallowed `Cargo.toml` version bumps outside release flow.
- Verify pre-push blocks branch pushes that include version bump without matching release tag.
- Verify quality gates pass: fmt, clippy, tests.
- Verify reliability SLO document exists and is updated (`docs/reference/reliability-slo.md`).

## Evidence Template

| Env | Build | Startup | Degrade | Commands | Notes |
|---|---|---|---|---|---|
| A1 | ☑ | ☑ | ☑ | ☑ | glibc build + smoke completed via `scripts/validate_runtime_matrix.sh`; runtime auth-stage reached with dummy token (`Api(NotFound)` expected). |
| A2 | ☑ | ☑ | ☑ | ☑ | musl build + smoke completed via `scripts/validate_runtime_matrix.sh`; runtime auth-stage reached with dummy token (`Api(NotFound)` expected). |

Latest report artifact:

- `docs/releases/runtime-validation-report.txt`

## Exit Criteria

- Both A1 and A2 rows are fully checked.
- No panic/crash during startup or command smoke run.
- Degraded capabilities are user-visible and non-fatal.
