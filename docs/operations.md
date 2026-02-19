# Operations

## Versioning Guard and Release Flow

- This repository blocks accidental `Cargo.toml` version edits in normal commits via Lefthook pre-commit checks.
- A Lefthook pre-push guard validates tag/version consistency.
- Strict branch policy (`scripts/enforce_git_flow.sh`) enforces local flow: `feature/* -> develop -> main`.

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
just release-plz-preview
just dist-preview
just release-pr
```

Notes:
- Production release is fully automated with `release-plz` + `cargo-dist`.
- `Release Plz` workflow runs on `main` and creates/updates release PRs.
- `Release Plz` must use `RELEASE_PLZ_TOKEN` repository secret (PAT or GitHub App token); default `GITHUB_TOKEN` does not reliably trigger required `pull_request` checks.
- `release-plz.toml` supports changelog grouping/filtering and PR metadata (labels/title/body) customization.
- Release PR merge produces version/changelog updates in repo history.
- `Release` workflow runs on pushed `v*` tags and builds distributables via `cargo-dist`.
- Release assets include musl archive, checksums, source archive, installer outputs, and `dist-plan.json`.
- PR quality is consolidated into a single required check (`quality / quality`) with internal scope-aware stages.
- Pre-push guard still enforces tag/version consistency for direct `main`/`develop` pushes.
- Version changes on feature branches are allowed for release-plz-managed release PR flow.
- Staged migration workflows remain available for explicit preview and diagnostics.
- Local bypass toggles are disabled; manual version bumps and local auto-tag shortcuts are not permitted.
- `main` push requires current `origin/develop` to be an ancestor (prevents hash drift).
- Quality CI is scope-aware:
  - single quality workflow runs policy + rust + version guard stages.
  - heavy rust checks run only for Rust-relevant changes.
- `release-plz` and `cargo-dist` preview workflows download official Linux binaries directly from upstream releases (no source compile in preview jobs).
- Local `just release-plz-preview` runs in a temporary worktree and does not mutate your active working tree.
- `cargo-dist` preview can now produce full artifacts and manifest for release-shape validation.

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