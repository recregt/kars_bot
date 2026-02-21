# Operations

## Versioning Guard and Release Flow

- Local hooks are intentionally minimal: only pre-commit Rust formatting.
- No local pre-push/post-merge gate. Branch safety is enforced in GitHub.

Install hooks once per clone:

```bash
scripts/install_hooks.sh
```

Common local operations via `just`:

```bash
just --list
just quality
just sync
just release-pr
```

Notes:
- Branch model is strict main-only: feature branch -> PR to `main` -> merge.
- `main` requires PR + required check (`CI / check`).
- Direct push to `main` is blocked by branch protection.
- Remote feature branch is auto-deleted after merge.

### release-plz + cargo-dist lifecycle

1. You merge a feature PR into `main`.
2. `Release Plz` workflow runs and opens/updates `chore(release): prepare release` PR.
3. You merge the release PR.
4. Workflow auto-creates missing `vX.Y.Z` tag (unless `RELEASE_PLZ_AUTO_TAG=false`).
5. `Release` workflow runs on tag and builds distributables with `cargo-dist` (`--artifacts=all`).
6. GitHub Release is published with generated notes and assets.

This keeps changelog/version generation in `release-plz` and binary packaging in `cargo-dist`.

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
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
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

## Logging

- Logging output is JSON by default and can be filtered with `RUST_LOG`.
- Monitor loop emits structured fields: `cpu`, `ram`, `disk`, `cpu_over`, `ram_over`, `disk_over`.

```bash
RUST_LOG=info ./target/release/kars_bot
```

```bash
RUST_LOG=info ./target/release/kars_bot | jq 'select(.target == "monitor" and .fields.cpu > 80)'
```