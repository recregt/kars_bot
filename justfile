set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

install-hooks:
  scripts/install_hooks.sh

sync:
  git fetch --all --prune
  git switch main
  git pull --ff-only origin main

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
