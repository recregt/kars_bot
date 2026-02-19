#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

RUNBOOKS=(
  "docs/runbooks/incident.md"
  "docs/runbooks/release.md"
  "docs/runbooks/rollback.md"
)

for runbook in "${RUNBOOKS[@]}"; do
  if [[ ! -f "$runbook" ]]; then
    echo "[runbook-gate] missing file: $runbook"
    exit 1
  fi

  if ! grep -Eq '^- Owner: .+' "$runbook"; then
    echo "[runbook-gate] missing Owner metadata: $runbook"
    exit 1
  fi

  if ! grep -Eq '^- Last validated: [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$runbook"; then
    echo "[runbook-gate] missing Last validated metadata: $runbook"
    exit 1
  fi

  if ! grep -Eq '^- Validation cadence: .+' "$runbook"; then
    echo "[runbook-gate] missing Validation cadence metadata: $runbook"
    exit 1
  fi
done

echo "[runbook-gate] PASS"