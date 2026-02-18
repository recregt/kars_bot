#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[ci-local] starting local CI parity checks"

git fetch --no-tags --depth=1 origin main >/dev/null 2>&1 || true

base_ref=""
if git rev-parse --verify origin/main >/dev/null 2>&1; then
  base_ref="$(git merge-base HEAD origin/main || true)"
fi

if [[ -z "$base_ref" ]]; then
  base_ref="$(git rev-parse HEAD~1 2>/dev/null || git rev-parse HEAD)"
fi

changed_rs=$(git diff --name-only "$base_ref...HEAD" -- '*.rs' | tr '\n' ' ')
if [[ -n "$changed_rs" ]]; then
  echo "[ci-local] rustfmt check on changed files"
  cargo fmt --all -- --check $changed_rs
else
  echo "[ci-local] no Rust file changes detected; skipping rustfmt check"
fi

echo "[ci-local] clippy"
cargo clippy --locked --all-targets --all-features -- -D warnings

if cargo nextest --version >/dev/null 2>&1; then
  echo "[ci-local] tests (nextest)"
  cargo nextest run --locked --all-targets
else
  echo "[ci-local] tests (cargo test fallback)"
  cargo test --locked
fi

echo "[ci-local] tls graph policy"
scripts/check_tls_stack.sh

echo "[ci-local] docs validation"
scripts/validate_docs.sh

echo "[ci-local] PASS"
