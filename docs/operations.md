# Operations

## Versioning Guard and Release Flow

* Local hooks are intentionally minimal: only pre-commit Rust formatting.
* No local pre-push/post-merge gate. Branch safety is enforced in GitHub.

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

* Branch model is strict main-only: feature branch -> PR to `main` -> merge.
* `main` requires PR + required check (`CI / check`).
* Direct push to `main` is blocked by branch protection.
* Remote feature branch is auto-deleted after merge.

### release-plz + cargo-dist lifecycle

1. You merge a feature PR into `main`.
2. `Release Plz` workflow runs and opens/updates `chore(release): prepare release` PR.
3. You merge the release PR.
4. Workflow auto-creates missing `vX.Y.Z` tag (unless `RELEASE_PLZ_AUTO_TAG=false`).
5. `Release` workflow runs on tag and builds distributables with `cargo-dist` (`--artifacts=all`).
6. GitHub Release is published with generated notes and assets.

This keeps changelog/version generation in `release-plz` and binary packaging in `cargo-dist`.

## Self-Update & Security Constraints

The bot features a direct self-update mechanism that downloads release assets from GitHub over HTTPS, without relying on third-party installer frameworks. Since the service runs in a hardened environment, the following logic applies:

* **No Profile Mutation**: The updater never reads or writes shell profile files (`.profile`, `.bashrc`). No `INSTALLER_NO_MODIFY_PATH` workaround is needed.
* **SHA256 Verification**: Downloaded archives are verified against the `sha256.sum` asset published alongside each release.
* **Staging**: Temp directory (via `tempfile`) is used for downloading and extracting. `PrivateTmp=yes` ensures isolation.
* **Sanity Check**: The extracted binary is executed with `--version` before installation.
* **Atomic Swap**: The new binary is written as `kars_bot.new` then renamed into place (`rename` is atomic on the same filesystem). The previous binary is backed up as `kars_bot.bak`.
* **Two-Phase Restart**: The update runs in two phases:
  1. **Prepare**: download, verify, extract, atomic-swap the binary on disk.
  2. **Restart**: send final Telegram message, then `systemctl restart kars-bot`. The current process is killed by SIGTERM and the new binary starts.

### Directory Layout

```text
/opt/kars_bot/
├── bin/
│   ├── kars_bot          # active binary (owner: bot:bot, mode: 0755)
│   ├── kars_bot.bak      # previous version backup (auto-created)
│   └── kars_bot.new      # transient staging file (removed after rename)
└── data/
    ├── config.toml        # bot configuration
    ├── anomaly_db/        # event storage
    └── reporting_store/   # reporting data
```

### Initial Setup

```bash
# Create directory structure
sudo install -d -o bot -g bot -m 0755 /opt/kars_bot/bin
sudo install -d -o bot -g bot -m 0755 /opt/kars_bot/data

# Install initial binary
sudo install -o bot -g bot -m 0755 kars_bot /opt/kars_bot/bin/kars_bot
```

### Polkit Rule (Service Restart Permission)

The `bot` user needs permission to restart `kars-bot.service` without a password.
Create `/etc/polkit-1/rules.d/50-kars-bot-restart.rules`:

```javascript
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.systemd1.manage-units" &&
        action.lookup("unit") == "kars-bot.service" &&
        action.lookup("verb") == "restart" &&
        subject.user == "bot") {
        return polkit.Result.YES;
    }
});
```

Then reload polkit:

```bash
sudo systemctl restart polkit
```

To verify the rule works:

```bash
sudo -u bot systemctl restart kars-bot
```

## systemd Service (Hardened)

Create `/etc/systemd/system/kars-bot.service` with strict sandboxing:

```ini
[Unit]
Description=Kars Telegram Monitoring Bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/opt/kars_bot/data
ExecStart=/opt/kars_bot/bin/kars_bot
Restart=always
RestartSec=5
User=bot
Group=bot

# Security Hardening
NoNewPrivileges=yes
ProtectHome=true
ProtectSystem=strict
PrivateTmp=yes

# Sandboxed Write Access
# bin/  -> binary self-update (atomic swap)
# data/ -> config, anomaly_db, reporting_store
ReadWritePaths=/opt/kars_bot/bin /opt/kars_bot/data

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

* `musl` binaries are usually more portable across Linux distributions than default `glibc` builds.
* Host tooling still affects command behavior (`systemctl`, `sensors`, `ss`, etc.); unsupported features degrade gracefully.
* Some environments can still differ in kernel/cgroup visibility, so validate `/status`, `/health`, `/sysstatus`, and `/graph` on target host.
* Runtime validation checklist: `docs/releases/runtime-validation-checklist.md`

## Logging

* Logging output is JSON by default and can be filtered with `RUST_LOG`.
* Monitor loop emits structured fields: `cpu`, `ram`, `disk`, `cpu_over`, `ram_over`, `disk_over`.

```bash
RUST_LOG=info ./target/release/kars_bot

```

```bash
RUST_LOG=info ./target/release/kars_bot | jq 'select(.target == "monitor" and .fields.cpu > 80)'

```