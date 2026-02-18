# Operations

## Versioning Guard and Release Tag Flow

- This repository blocks accidental `Cargo.toml` version edits in normal commits via Lefthook pre-commit checks.
- A Lefthook pre-push guard validates tag/version consistency.
- Strict branch policy (`scripts/enforce_git_flow.sh`) blocks direct commit/push flows to `main` and `develop`.

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
just release-preflight v1.3.3-pre
just release-safe v1.3.3
```

Create a release tag with version sync:

```bash
scripts/release_tag.sh v0.8.0
```

Run release checks without mutations:

```bash
scripts/release_tag.sh --dry-run v0.8.0
```

Notes:
- The script runs `just ci` parity checks before any release mutation.
- Use `just doctor` first if local environment drift is suspected.
- The script generates an English `CHANGELOG.md` section via `git-cliff`.
- The script logs binary size to `docs/releases/binary-size.csv`.
- The script bumps `Cargo.toml` only when needed.
- Version bump commit uses `ALLOW_VERSION_BUMP=1` to pass the guard.
- If no tag/release is planned, `Cargo.toml` version must stay unchanged.
- Pre-push guard can auto-create missing local release tag (opt-in):

```bash
AUTO_CREATE_MISSING_TAG=1 git push --follow-tags
```

  The hook creates `vX.Y.Z` locally (at pushed commit) and intentionally stops once,
  so a second push includes the newly created tag.

Prerequisite:

```bash
cargo install git-cliff
```

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