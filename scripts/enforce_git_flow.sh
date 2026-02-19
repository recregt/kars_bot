#!/usr/bin/env bash
set -euo pipefail

mode="${1:-commit}"

cleanup_merged_feature_branch() {
  local is_squash_merge="${1:-0}"

  if [[ "$(git symbolic-ref --short HEAD 2>/dev/null || true)" != "develop" ]]; then
    return 0
  fi

  if [[ "$is_squash_merge" == "1" ]]; then
    return 0
  fi

  local parents_line
  parents_line="$(git rev-list --parents -n 1 HEAD)"
  local parent_count
  parent_count="$(awk '{print NF-1}' <<<"$parents_line")"
  if (( parent_count < 2 )); then
    return 0
  fi

  local merged_head
  merged_head="$(git rev-parse HEAD^2)"

  local feature_branches=()
  while IFS= read -r feature_branch; do
    [[ -z "$feature_branch" ]] && continue
    if [[ "$(git rev-parse "refs/heads/$feature_branch")" == "$merged_head" ]]; then
      feature_branches+=("$feature_branch")
    fi
  done < <(git for-each-ref --format='%(refname:short)' refs/heads/feature)

  if (( ${#feature_branches[@]} == 0 )); then
    return 0
  fi

  for feature_branch in "${feature_branches[@]}"; do
    if git branch -d "$feature_branch" >/dev/null 2>&1; then
      echo "[git-flow] Cleaned up merged local branch '$feature_branch'."
    fi
  done
}

assert_no_protected_branch_delete() {
  readarray -t push_lines

  for line in "${push_lines[@]}"; do
    local_ref=$(awk '{print $1}' <<<"$line")
    local_sha=$(awk '{print $2}' <<<"$line")
    remote_ref=$(awk '{print $3}' <<<"$line")

    if [[ "$local_sha" != "0000000000000000000000000000000000000000" ]]; then
      continue
    fi

    case "$remote_ref" in
      refs/heads/main|refs/heads/develop)
        echo "[git-flow] Blocked: deleting protected remote branch '${remote_ref#refs/heads/}' is forbidden."
        exit 1
        ;;
    esac

    case "$local_ref" in
      refs/heads/main|refs/heads/develop)
        echo "[git-flow] Blocked: deleting protected local branch '${local_ref#refs/heads/}' is forbidden."
        exit 1
        ;;
    esac
  done
}

case "$mode" in
  commit)
    exit 0
    ;;
  push)
    assert_no_protected_branch_delete
    ;;
  post-merge)
    cleanup_merged_feature_branch "${2:-0}"
    ;;
  *)
    echo "Usage: scripts/enforce_git_flow.sh [commit|push|post-merge]"
    exit 1
    ;;
esac
