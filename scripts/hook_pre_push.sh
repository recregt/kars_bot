#!/usr/bin/env bash
set -euo pipefail

source scripts/lib/log.sh
source scripts/lib/git_diff_scope.sh

SCRIPT_NAME="pre-push"

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
    log_warn "no stdin refs; using synthesized push line for $current_branch."
  else
    log_error "Blocked: no push refs on stdin and detached HEAD; cannot enforce policy safely."
    exit 1
  fi
fi

if [[ -x scripts/enforce_git_flow.sh ]]; then
  printf '%s\n' "${push_lines[@]}" | scripts/enforce_git_flow.sh push
else
  log_error "Blocked: scripts/enforce_git_flow.sh is missing or not executable."
  exit 1
fi

has_release_tag_for_version() {
  local version="$1"
  for line in "${push_lines[@]}"; do
    local_ref=$(awk '{print $1}' <<<"$line")
    local_sha=$(awk '{print $2}' <<<"$line")
    if [[ "$local_sha" == "0000000000000000000000000000000000000000" ]]; then
      continue
    fi
    if [[ "$local_ref" == "refs/tags/v$version" ]]; then
      return 0
    fi
  done

  if git ls-remote --tags origin "refs/tags/v$version" | grep -q "refs/tags/v$version"; then
    return 0
  fi

  return 1
}

maybe_create_missing_release_tag() {
  local version="$1"
  local target_sha="$2"

  if [[ "${AUTO_CREATE_MISSING_TAG:-0}" != "1" ]]; then
    return 1
  fi

  if git rev-parse --verify "refs/tags/v$version" >/dev/null 2>&1; then
    return 0
  fi

  git tag -a "v$version" "$target_sha" -m "Release v$version"
  log_info "Auto-created local tag v$version at $target_sha"
  log_info "Push stopped intentionally so git can recompute refs including the new tag."
  log_info "Re-run push with: git push --follow-tags"
  return 0
}

for line in "${push_lines[@]}"; do
  local_ref=$(awk '{print $1}' <<<"$line")
  local_sha=$(awk '{print $2}' <<<"$line")
  remote_sha=$(awk '{print $4}' <<<"$line")

  if [[ "$local_sha" == "0000000000000000000000000000000000000000" ]]; then
    continue
  fi

  if [[ "$local_ref" =~ ^refs/tags/v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    expected="${BASH_REMATCH[1]}"
    actual=$(git show "${local_ref#refs/tags/}":Cargo.toml | awk -F '"' '/^version = /{print $2; exit}')
    if [[ "$actual" != "$expected" ]]; then
      log_error "Blocked: tag ${local_ref#refs/tags/} points to Cargo.toml version $actual"
      log_error "Expected Cargo.toml version: $expected"
      exit 1
    fi
    continue
  fi

  if [[ "$local_ref" =~ ^refs/heads/ ]]; then
    range="$(git_scope_push_range "$local_sha" "$remote_sha")"

    if git diff "$range" -- Cargo.toml | grep -Eq '^[+-]version = "[0-9]+\.[0-9]+\.[0-9]+"'; then
      target_version=$(git show "$local_sha":Cargo.toml | awk -F '"' '/^version = /{print $2; exit}')
      if ! has_release_tag_for_version "$target_version"; then
        if maybe_create_missing_release_tag "$target_version" "$local_sha"; then
          exit 1
        fi
        log_error "Blocked: Cargo.toml version changed to $target_version but no matching tag v$target_version is in this push."
        log_info "Suggestion: run scripts/release_tag.sh v$target_version and push commit + tag together."
        log_info "Optional automation: AUTO_CREATE_MISSING_TAG=1 git push --follow-tags"
        exit 1
      fi
    fi
  fi
done

exit 0
