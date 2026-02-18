#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

source scripts/lib/log.sh
source scripts/lib/git_diff_scope.sh

SCRIPT_NAME="ci-local"

log_info "starting local CI parity checks"

base_ref="$(git_scope_base_ref)"
changed_rs="$(git_scope_changed_rust_files_space "$base_ref")"
changed_all="$(git_scope_changed_files_space "$base_ref")"

if [[ -n "$changed_rs" ]]; then
  log_info "rustfmt check on changed files"
  rustfmt --edition 2024 --check $changed_rs
else
  log_info "no Rust file changes detected; skipping rustfmt check"
fi

if git_scope_is_rust_related "$changed_all"; then
  log_info "clippy"
  cargo clippy --locked --all-targets --all-features -- -D warnings

  if cargo nextest --version >/dev/null 2>&1; then
    log_info "tests (nextest)"
    cargo nextest run --locked --all-targets
  else
    log_info "tests (cargo test fallback)"
    cargo test --locked
  fi

  log_info "tls graph policy"
  scripts/check_tls_stack.sh
else
  log_info "no Rust/Cargo/TLS-relevant changes detected; skipping clippy/tests/tls checks"
fi

log_info "docs validation"
scripts/validate_docs.sh

log_info "PASS"
