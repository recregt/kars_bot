#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f Cargo.toml ]]; then
  exit 0
fi

push_lines=()
if IFS= read -r -t 1 first_line; then
  push_lines+=("$first_line")
  while IFS= read -r line; do
    push_lines+=("$line")
  done
else
  remote_name="${1:-origin}"
  current_branch="$(git symbolic-ref --short HEAD 2>/dev/null || true)"
  if [[ -n "$current_branch" ]]; then
    local_ref="refs/heads/$current_branch"
    local_sha="$(git rev-parse HEAD)"
    remote_ref="refs/heads/$current_branch"
    remote_sha="0000000000000000000000000000000000000000"

    upstream_ref="$(git for-each-ref --format='%(upstream:short)' "refs/heads/$current_branch")"
    if [[ -n "$upstream_ref" ]] && git rev-parse --verify "$upstream_ref" >/dev/null 2>&1; then
      remote_sha="$(git rev-parse "$upstream_ref")"
      upstream_branch="${upstream_ref#${remote_name}/}"
      if [[ "$upstream_branch" != "$upstream_ref" ]]; then
        remote_ref="refs/heads/$upstream_branch"
      fi
    fi

    push_lines+=("$local_ref $local_sha $remote_ref $remote_sha")
  else
    echo "[pre-push] Blocked: no push refs on stdin and detached HEAD; cannot validate pushed tags."
    exit 1
  fi
fi

for line in "${push_lines[@]}"; do
  local_ref=$(awk '{print $1}' <<<"$line")
  local_sha=$(awk '{print $2}' <<<"$line")

  if [[ "$local_ref" =~ ^refs/tags/v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    if [[ "$local_sha" == "0000000000000000000000000000000000000000" ]]; then
      continue
    fi
    expected="${BASH_REMATCH[1]}"
    actual=$(git show "${local_ref#refs/tags/}":Cargo.toml | awk -F '"' '/^version = /{print $2; exit}')
    if [[ "$actual" != "$expected" ]]; then
      echo "[pre-push] Blocked: tag ${local_ref#refs/tags/} points to Cargo.toml version $actual"
      echo "[pre-push] Expected Cargo.toml version: $expected"
      exit 1
    fi
  fi
done

exit 0
