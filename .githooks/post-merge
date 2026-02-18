#!/usr/bin/env bash
set -euo pipefail

if [[ ! -x scripts/enforce_git_flow.sh ]]; then
  echo "[post-merge] Note: scripts/enforce_git_flow.sh is missing or not executable."
  exit 0
fi

scripts/enforce_git_flow.sh post-merge "${1:-0}"
