#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="${1:-default}"

echo "[doctor] starting environment diagnostics"
echo "[doctor] mode: $MODE"

required_cmds=(git cargo rustc)
optional_cmds=(just jq curl tar install systemctl actionlint cargo-nextest)

missing_required=0
remediation=()

add_remediation() {
  remediation+=("$1")
}

for cmd in "${required_cmds[@]}"; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "[doctor] missing required command: $cmd"
    missing_required=1
    add_remediation "Install required tool '$cmd' and re-run: just doctor"
  else
    echo "[doctor] ok: $cmd"
  fi
done

if [[ "$MODE" == "--release" ]]; then
  if command -v git-cliff >/dev/null 2>&1; then
    echo "[doctor] release ok: git-cliff"
  else
    echo "[doctor] missing required release tool: git-cliff"
    missing_required=1
    add_remediation "Install release tool: cargo install git-cliff"
  fi
else
  if command -v git-cliff >/dev/null 2>&1; then
    echo "[doctor] optional ok: git-cliff"
  else
    echo "[doctor] optional missing: git-cliff"
  fi
fi

if [[ -f lefthook.yml ]]; then
  echo "[doctor] lefthook config: present"
  if command -v lefthook >/dev/null 2>&1; then
    echo "[doctor] hook tool ok: lefthook"
  else
    echo "[doctor] missing required hook tool: lefthook"
    missing_required=1
    add_remediation "Install Lefthook then run: scripts/install_hooks.sh"
  fi
else
  echo "[doctor] lefthook config: missing"
fi

for cmd in "${optional_cmds[@]}"; do
  if command -v "$cmd" >/dev/null 2>&1; then
    echo "[doctor] optional ok: $cmd"
  else
    echo "[doctor] optional missing: $cmd"
  fi
done

if [[ -f justfile ]]; then
  echo "[doctor] justfile: present"
else
  echo "[doctor] justfile: missing"
  add_remediation "Restore or create justfile to use standardized repo commands"
fi

if [[ -d .git ]]; then
  hooks_path="$(git config --local --get core.hooksPath || true)"
  if [[ -z "$hooks_path" ]]; then
    hooks_path=".git/hooks"
    echo "[doctor] local core.hooksPath: default (.git/hooks)"
  else
    echo "[doctor] local core.hooksPath: $hooks_path"
  fi

  pre_push_hook="$hooks_path/pre-push"
  if [[ -f "$pre_push_hook" ]] && grep -q "lefthook run \"pre-push\"" "$pre_push_hook"; then
    echo "[doctor] hooks installed: pre-push managed by lefthook ($pre_push_hook)"
  else
    echo "[doctor] hooks not installed or outdated"
    add_remediation "Install/refresh hooks: scripts/install_hooks.sh"
  fi
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "[doctor] working tree is dirty"
else
  echo "[doctor] working tree is clean"
fi

if (( missing_required > 0 )); then
  if (( ${#remediation[@]} > 0 )); then
    echo "[doctor] suggested fixes:"
    for item in "${remediation[@]}"; do
      echo "  - $item"
    done
  fi
  echo "[doctor] FAIL"
  exit 1
fi

if (( ${#remediation[@]} > 0 )); then
  echo "[doctor] suggestions:"
  for item in "${remediation[@]}"; do
    echo "  - $item"
  done
fi

echo "[doctor] PASS"
