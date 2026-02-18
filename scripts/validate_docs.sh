#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

required_files=(
  "README.md"
  "docs/README.md"
  "docs/ROADMAP.md"
  "docs/reference/commands.md"
  "docs/reference/config.md"
  "docs/runbooks/release.md"
  "docs/runbooks/rollback.md"
  "docs/runbooks/incident.md"
  "docs/architecture/overview.md"
  "docs/contributing/docs-style.md"
)

for path in "${required_files[@]}"; do
  if [[ ! -f "$path" ]]; then
    echo "[validate-docs] missing required file: $path"
    exit 1
  fi
done

scripts/generate_docs_reference.sh >/dev/null

if ! git diff --quiet -- docs/reference/commands.md docs/reference/config.md; then
  echo "[validate-docs] generated references are out of date."
  echo "Run: scripts/generate_docs_reference.sh"
  git --no-pager diff -- docs/reference/commands.md docs/reference/config.md | sed -n '1,200p'
  exit 1
fi

link_failures=0

check_links_in_file() {
  local file="$1"
  local source_dir
  source_dir="$(dirname "$file")"

  while IFS= read -r target; do
    [[ -z "$target" ]] && continue

    if [[ "$target" =~ ^https?:// ]] || [[ "$target" =~ ^mailto: ]] || [[ "$target" =~ ^# ]]; then
      continue
    fi

    local clean_target="${target%%#*}"
    clean_target="${clean_target%%\?*}"

    [[ -z "$clean_target" ]] && continue

    local resolved
    if [[ "$clean_target" == /* ]]; then
      resolved=".${clean_target}"
    else
      resolved="$source_dir/$clean_target"
    fi

    if [[ ! -e "$resolved" ]]; then
      echo "[validate-docs] broken local link in $file -> $target"
      link_failures=1
    fi
  done < <(grep -oE '\[[^]]+\]\(([^)]+)\)' "$file" | sed -E 's/.*\(([^)]+)\)/\1/')
}

while IFS= read -r file; do
  check_links_in_file "$file"
done < <(find docs -type f -name '*.md' | sort)

check_links_in_file "README.md"

if grep -RIn --include='*.md' '[[:blank:]]$' docs README.md >/tmp/docs_trailing_ws.txt 2>/dev/null; then
  echo "[validate-docs] trailing whitespace found in markdown files:"
  sed -n '1,120p' /tmp/docs_trailing_ws.txt
  rm -f /tmp/docs_trailing_ws.txt
  exit 1
fi
rm -f /tmp/docs_trailing_ws.txt

if [[ "$link_failures" -ne 0 ]]; then
  exit 1
fi

echo "[validate-docs] PASS"
