#!/usr/bin/env bash
set -euo pipefail

mode="${1:-commit}"

current_branch() {
  git symbolic-ref --short HEAD 2>/dev/null || true
}

merge_head_sha() {
  git rev-parse -q --verify MERGE_HEAD 2>/dev/null || true
}

is_merge_commit_context() {
  [[ -n "$(merge_head_sha)" ]]
}

assert_commit_policy() {
  local branch
  branch="$(current_branch)"

  if [[ "$branch" != "main" && "$branch" != "develop" ]]; then
    return 0
  fi

  if ! is_merge_commit_context; then
    echo "[git-flow] Blocked: direct commits to '$branch' are forbidden."
    echo "Allowed flow: commit on feature/*, merge into develop, then merge develop into main."
    exit 1
  fi
}

assert_protected_branch_push_is_merge_only() {
  local branch_ref="$1"
  local local_sha="$2"
  local remote_sha="$3"

  local range
  if [[ "$remote_sha" == "0000000000000000000000000000000000000000" ]]; then
    local empty_tree
    empty_tree=$(git hash-object -t tree /dev/null)
    range="$empty_tree..$local_sha"
  else
    range="$remote_sha..$local_sha"
  fi

  while IFS= read -r commit_sha; do
    [[ -z "$commit_sha" ]] && continue
    parents_line=$(git rev-list --parents -n 1 "$commit_sha")
    parent_count=$(awk '{print NF-1}' <<<"$parents_line")
    if (( parent_count < 2 )); then
      echo "[git-flow] Blocked: non-merge commit $commit_sha detected in push to '${branch_ref#refs/heads/}'."
      echo "Only merge commits are allowed on protected branches."
      exit 1
    fi
  done < <(git rev-list --first-parent "$range")
}

assert_push_policy() {
  if [[ ! -f Cargo.toml ]]; then
    return 0
  fi

  readarray -t push_lines
  for line in "${push_lines[@]}"; do
    local_ref=$(awk '{print $1}' <<<"$line")
    local_sha=$(awk '{print $2}' <<<"$line")
    remote_ref=$(awk '{print $3}' <<<"$line")
    remote_sha=$(awk '{print $4}' <<<"$line")

    if [[ "$local_sha" == "0000000000000000000000000000000000000000" ]]; then
      case "$remote_ref" in
        refs/heads/main|refs/heads/develop)
          echo "[git-flow] Blocked: deleting protected remote branch '${remote_ref#refs/heads/}' is forbidden."
          exit 1
          ;;
      esac
      continue
    fi

    case "$local_ref" in
      refs/heads/develop)
        assert_protected_branch_push_is_merge_only "$local_ref" "$local_sha" "$remote_sha"
        ;;
      refs/heads/main)
        assert_protected_branch_push_is_merge_only "$local_ref" "$local_sha" "$remote_sha"
        if ! git rev-parse --verify --quiet refs/remotes/origin/develop >/dev/null 2>&1; then
          echo "[git-flow] Blocked: cannot validate develop ancestry because origin/develop is unavailable."
          exit 1
        fi
        if ! git merge-base --is-ancestor refs/remotes/origin/develop "$local_sha"; then
          echo "[git-flow] Blocked: main push must include current origin/develop tip."
          echo "Merge develop into main first to avoid hash drift."
          exit 1
        fi
        ;;
    esac
  done
}

cleanup_merged_feature_branch() {
  local is_squash_merge="${1:-0}"

  if [[ "$(current_branch)" != "develop" ]]; then
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
    else
      echo "[git-flow] Note: could not auto-delete '$feature_branch' with -d."
      echo "Run manually if needed: git branch -d $feature_branch"
    fi
  done
}

case "$mode" in
  commit)
    assert_commit_policy
    ;;
  push)
    assert_push_policy
    ;;
  post-merge)
    cleanup_merged_feature_branch "${2:-0}"
    ;;
  *)
    echo "Usage: scripts/enforce_git_flow.sh [commit|push|post-merge]"
    exit 1
    ;;
esac
