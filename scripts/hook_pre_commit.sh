#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f Cargo.toml ]]; then
  exit 0
fi

if [[ -x scripts/enforce_git_flow.sh ]]; then
  scripts/enforce_git_flow.sh commit
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

max_lines=200
violations=()

for file in "${staged_rust_files[@]}"; do
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
