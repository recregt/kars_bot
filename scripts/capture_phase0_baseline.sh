#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p docs/releases
report_file="docs/releases/v1.2.0-baseline.md"

{
  echo "# v1.2.0 Phase 0 Baseline"
  echo
  echo "- generated_at_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- git_commit: $(git rev-parse --short HEAD)"
  echo "- git_branch: $(git branch --show-current)"
  echo
  echo "## Dependency Baseline"
  echo
  echo '```bash'
  echo '$ cargo tree -i rustls'
  cargo tree -i rustls || true
  echo '```'
  echo
  echo '```bash'
  echo '$ cargo tree -i openssl'
  cargo tree -i openssl || true
  echo '```'
  echo
  echo "## Binary Introspection"
  echo
  if [[ -x target/release/kars_bot ]]; then
    echo '```bash'
    echo '$ file target/release/kars_bot'
    file target/release/kars_bot
    echo '$ ldd target/release/kars_bot'
    ldd target/release/kars_bot || true
    echo '```'
  else
    echo "- glibc binary not present: run cargo build --release"
  fi
  echo
  if [[ -x target/x86_64-unknown-linux-musl/release/kars_bot ]]; then
    echo '```bash'
    echo '$ file target/x86_64-unknown-linux-musl/release/kars_bot'
    file target/x86_64-unknown-linux-musl/release/kars_bot
    echo '$ ldd target/x86_64-unknown-linux-musl/release/kars_bot'
    ldd target/x86_64-unknown-linux-musl/release/kars_bot || true
    echo '```'
  else
    echo "- musl binary not present: run scripts/build_musl.sh"
  fi
} > "$report_file"

echo "[phase0] baseline written: $report_file"
