set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

install-hooks:
  scripts/install_hooks.sh

sync:
  @echo ">>> Syncing with main branch (remote)..."
  git fetch --all --prune
  git checkout main
  git reset --hard origin/main
  git clean -fd
  @echo ">>> Sync complete!"
  
fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --locked --all-targets --all-features -- -D warnings

test:
  cargo test --locked

quality:
  just fmt-check
  just clippy
  just test

release-pr:
  gh workflow run release-plz.yml
