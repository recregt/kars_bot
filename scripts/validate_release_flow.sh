#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

candidate="${1:-v1.1.1-pre}"
base_tag="$candidate"
if [[ "$candidate" =~ ^(v[0-9]+\.[0-9]+\.[0-9]+)-[0-9A-Za-z.-]+$ ]]; then
  base_tag="${BASH_REMATCH[1]}"
fi

if [[ ! "$base_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "[validate-release] invalid candidate: $candidate"
  echo "Usage: scripts/validate_release_flow.sh vX.Y.Z-pre"
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "[validate-release] working tree must be clean"
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

echo "[validate-release] candidate=$candidate (dry-run using $base_tag)"
if ! scripts/release_tag.sh --dry-run "$base_tag" | tee "$tmp_out"; then
  echo "[validate-release] release dry-run failed"
  exit 1
fi

if ! grep -q "Previewing changelog section for $base_tag" "$tmp_out"; then
  echo "[validate-release] missing changelog preview output"
  exit 1
fi

if ! grep -q "## $base_tag" "$tmp_out"; then
  echo "[validate-release] git-cliff output did not include expected header ## $base_tag"
  exit 1
fi

if ! grep -q "No files changed, no commit created, no tag created" "$tmp_out"; then
  echo "[validate-release] dry-run safety check failed"
  exit 1
fi

echo "[validate-release] PASS"
