#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f Cargo.toml ]]; then
  exit 0
fi

current_branch="$(git symbolic-ref --short HEAD 2>/dev/null || true)"
is_release_main_bypass=0
if [[ "$current_branch" == "main" && "${ALLOW_MAIN_RELEASE_COMMIT:-}" == "1" && "${ALLOW_VERSION_BUMP:-}" == "1" ]]; then
  is_release_main_bypass=1
fi

if [[ -x scripts/enforce_git_flow.sh ]]; then
  if [[ "$is_release_main_bypass" -ne 1 ]]; then
    scripts/enforce_git_flow.sh commit
  fi
else
  echo "[pre-commit] Blocked: scripts/enforce_git_flow.sh is missing or not executable."
  exit 1
fi

if git diff --cached -- Cargo.toml | grep -Eq '^[+-]version = "[0-9]+\.[0-9]+\.[0-9]+"'; then
  if [[ "${ALLOW_VERSION_BUMP:-}" != "1" ]]; then
    echo "[pre-commit] Blocked: Cargo.toml version change detected."
    echo "Version changes are allowed only in release flow."
    echo "Use: ALLOW_VERSION_BUMP=1 git commit ... (prefer scripts/release_tag.sh)"
    exit 1
  fi

  if ! git diff --cached --name-only | grep -qx "CHANGELOG.md"; then
    echo "[pre-commit] Blocked: Cargo.toml version changed but CHANGELOG.md is not staged."
    echo "Release commits must include changelog update in the same commit."
    exit 1
  fi

  if git ls-files --error-unmatch Cargo.lock >/dev/null 2>&1; then
    if ! git diff --cached --name-only | grep -qx "Cargo.lock"; then
      echo "[pre-commit] Blocked: Cargo.toml version changed but Cargo.lock is not staged."
      echo "Run cargo generate-lockfile and include Cargo.lock in the release commit."
      exit 1
    fi
  fi

  target_version=$(git show :Cargo.toml | awk -F '"' '/^version = /{print $2; exit}')
  if [[ -z "$target_version" ]]; then
    echo "[pre-commit] Blocked: could not read staged Cargo.toml version."
    exit 1
  fi

  if ! git show :CHANGELOG.md | grep -q "^## v$target_version"; then
    echo "[pre-commit] Blocked: staged CHANGELOG.md missing section '## v$target_version'."
    echo "Run scripts/release_tag.sh v$target_version to keep release metadata synchronized."
    exit 1
  fi
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

max_lines=200
violations=()

added_rust_files=()
while IFS= read -r -d '' file; do
  [[ "$file" =~ \.rs$ ]] || continue
  added_rust_files+=("$file")
done < <(git diff --cached --name-only -z --diff-filter=A)

for file in "${added_rust_files[@]}"; do
  [[ ! -f "$file" ]] && continue
  line_count=$(wc -l < "$file")
  if (( line_count > max_lines )); then
    violations+=("$file:$line_count")
  fi
done

if (( ${#violations[@]} > 0 )); then
  echo "[pre-commit] Blocked: Rust files exceed $max_lines lines."
  echo "Please modularize to folder/mod.rs structure before commit."
  printf ' - %s\n' "${violations[@]}"
  exit 1
fi

docs_related_changed=0
while IFS= read -r changed_file; do
  case "$changed_file" in
    README.md|docs/*|docs/**/*|src/commands/command_def.rs|src/config/schema.rs|scripts/generate_docs_reference.sh|scripts/validate_docs.sh)
      docs_related_changed=1
      break
      ;;
  esac
done < <(git diff --cached --name-only)

if [[ "$docs_related_changed" -eq 1 ]]; then
  if [[ -x scripts/validate_docs.sh ]]; then
    scripts/validate_docs.sh
  else
    echo "[pre-commit] Blocked: scripts/validate_docs.sh is missing or not executable."
    exit 1
  fi
fi
