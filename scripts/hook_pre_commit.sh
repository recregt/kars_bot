#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f Cargo.toml ]]; then
  exit 0
fi

staged_rust_files=()
while IFS= read -r -d '' file; do
  [[ "$file" =~ \.rs$ ]] || continue
  staged_rust_files+=("$file")
done < <(git diff --cached --name-only -z --diff-filter=ACMR)

if (( ${#staged_rust_files[@]} == 0 )); then
  exit 0
fi

if [[ "${PRECOMMIT_AUTO_FMT:-1}" == "1" ]]; then
  if ! command -v rustfmt >/dev/null 2>&1; then
    echo "[pre-commit] Blocked: rustfmt is required for auto-format mode."
    echo "Install rustfmt component or set PRECOMMIT_AUTO_FMT=0 to bypass auto-format."
    exit 1
  fi

  rust_edition=$(awk -F '"' '/^edition = /{print $2; exit}' Cargo.toml)
  rust_edition="${rust_edition:-2024}"

  echo "[pre-commit] Auto-formatting staged Rust files with rustfmt (edition $rust_edition)."
  rustfmt --edition "$rust_edition" "${staged_rust_files[@]}"
  git add -- "${staged_rust_files[@]}"
fi
