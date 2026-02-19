# Operations

## Versioning Guard and Release Flow

- Local hooks are intentionally lightweight for fast iteration.
- Pre-commit hook focuses on Rust auto-formatting for staged `.rs` files.
- Pre-push hook validates pushed `v*` tags against `Cargo.toml` version.

Install hooks once per clone:

```bash
scripts/install_hooks.sh
```

Common local operations via `just`:

```bash
just bootstrap
just --list
just ci
just doctor
just doctor-release
just docs
just release-pr
```

Notes:
- Production release is fully automated with `release-plz` + `cargo-dist`.
- `Release Plz` workflow runs on `main` and creates/updates release PRs.
- `Release Plz` must use `RELEASE_PLZ_TOKEN` repository secret (PAT or GitHub App token); default `GITHUB_TOKEN` does not reliably trigger required `pull_request` checks.
- `release-plz.toml` supports changelog grouping/filtering and PR metadata (labels/title/body) customization.
- Release PR merge produces version/changelog updates in repo history.
- If a push to `main` changes `Cargo.toml` version and matching `v<version>` tag is missing, `Release Plz` workflow auto-creates and pushes the tag.
- `Release` workflow runs on pushed `v*` tags and builds distributables via `cargo-dist`.
- Release assets include musl archive, checksums, source archive, installer outputs, and `dist-plan.json`.
- PR quality remains consolidated into a single required check (`quality / quality`) and now runs a minimal fixed Rust pipeline (`fmt`, `clippy`, `nextest`, TLS graph check).
- Automation is intentionally minimal: only `Quality Gates`, `Release Plz`, and `Release` workflows are retained.
- Branch sync from `main` to `develop` is manual via local flow (`just sync` + merge discipline).
- Repository rulesets are minimal (`deletion` + `non_fast_forward`) with admin-role bypass enabled.

## systemd Service

Create `/etc/systemd/system/kars-bot.service`:

```ini
[Unit]
Description=Kars Telegram Monitoring Bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/opt/kars_bot
ExecStart=/opt/kars_bot/target/release/kars_bot
Restart=always
RestartSec=5
User=bot
Group=bot

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now kars-bot
sudo systemctl status kars-bot
```

### Minimum-Privilege sudoers for Update Flow

`/update apply` executes `scripts/server_update.sh`, which requires controlled `systemctl` operations and binary replacement under `/opt/kars_bot`.

Example sudoers policy (edit with `visudo`):

```text
Cmnd_Alias KARS_BOT_UPDATE = /usr/bin/systemctl status kars-bot, /usr/bin/systemctl restart kars-bot, /usr/bin/systemctl is-active kars-bot
bot ALL=(root) NOPASSWD: KARS_BOT_UPDATE
```

Notes:
- Restrict commands to the exact service unit and required verbs only.
- Keep file ownership on `/opt/kars_bot` minimal and explicit.
- Validate with `/update check` before allowing `/update apply` in production.

### Non-systemd Degraded Behavior

When systemd is not available, `/update apply` is intentionally blocked by capability checks.

- `/update check` still reports readiness details and failure reasons.
- `/update apply` returns a degraded-mode message instead of attempting restart logic.
- In non-systemd environments, perform manual binary rollout and process supervision.

## Docker Build (Optional)

```bash
docker run --rm \
  -v "$PWD":/app \
  -w /app \
  rust:1.93 \
  bash -lc "cargo build --release"
```

## Portable Linux Binary (musl, Optional)

Build with static musl target:

```bash
scripts/build_musl.sh
```

Artifact path:

```text
target/x86_64-unknown-linux-musl/release/kars_bot
```

Manual equivalent:

```bash
rustup target add x86_64-unknown-linux-musl
sudo apt-get update && sudo apt-get install -y musl-tools
cargo build --release --target x86_64-unknown-linux-musl
```

Portability notes:
- `musl` binaries are usually more portable across Linux distributions than default `glibc` builds.
- Host tooling still affects command behavior (`systemctl`, `sensors`, `ss`, etc.); unsupported features degrade gracefully.
- Some environments can still differ in kernel/cgroup visibility, so validate `/status`, `/health`, `/sysstatus`, and `/graph` on target host.
- Runtime validation checklist: `docs/releases/runtime-validation-checklist.md`
- Automated matrix validation runner: `scripts/validate_runtime_matrix.sh`

## Logging

- Logging output is JSON by default and can be filtered with `RUST_LOG`.
- Monitor loop emits structured fields: `cpu`, `ram`, `disk`, `cpu_over`, `ram_over`, `disk_over`.

```bash
RUST_LOG=info ./target/release/kars_bot
```

```bash
RUST_LOG=info ./target/release/kars_bot | jq 'select(.target == "monitor" and .fields.cpu > 80)'
```