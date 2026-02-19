#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[chaos] DNS fault-injection drill (expect resolver failure)"
if getent hosts does-not-exist.invalid >/dev/null 2>&1; then
  echo "[chaos] expected NXDOMAIN-style lookup failure but got success"
  exit 1
fi

echo "[chaos] Graph render failure-path drill"
cargo test -q readiness_check_runs

echo "[chaos] Update rollback readiness drill"
check_output="$(bash scripts/server_update.sh --check-only 2>&1 || true)"
if ! grep -q "APPLY_READY=" <<<"$check_output"; then
  echo "[chaos] update check output missing APPLY_READY marker"
  echo "$check_output"
  exit 1
fi

echo "[chaos] PASS"