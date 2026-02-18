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
    if [[ "$branch" == "main" ]]; then
      echo "Allowed flow: feature/* -> develop -> (merge) main"
    else
      echo "Allowed flow: commit on feature/*, then merge into develop"
    fi
    exit 1
  fi

  local merge_head
  merge_head="$(merge_head_sha)"

  if [[ "$branch" == "main" ]]; then
    if ! git show-ref --verify --quiet refs/heads/develop; then
      echo "[git-flow] Blocked: develop branch not found for main merge validation."
      exit 1
    fi

    local develop_sha
    develop_sha="$(git rev-parse refs/heads/develop)"
    if [[ "$merge_head" != "$develop_sha" ]]; then
      echo "[git-flow] Blocked: main can only merge from current develop HEAD."
      echo "MERGE_HEAD=$merge_head develop=$develop_sha"
      exit 1
    fi
    return 0
  fi

  local found_feature_ref=0
  while IFS= read -r ref_name; do
    [[ -z "$ref_name" ]] && continue
    if [[ "$(git rev-parse "$ref_name")" == "$merge_head" ]]; then
      found_feature_ref=1
      break
    fi
  done < <(git for-each-ref --format='%(refname:short)' refs/heads/feature)

  if [[ "$found_feature_ref" -ne 1 ]]; then
    echo "[git-flow] Blocked: develop merges must come from feature/* branch heads."
    echo "MERGE_HEAD=$merge_head does not match any refs/heads/feature/*"
    exit 1
  fi
}

assert_push_policy() {
  if [[ ! -f Cargo.toml ]]; then
    return 0
  fi

  readarray -t push_lines
  for line in "${push_lines[@]}"; do
    local_ref=$(awk '{print $1}' <<<"$line")
    local_sha=$(awk '{print $2}' <<<"$line")
    remote_sha=$(awk '{print $4}' <<<"$line")

    [[ "$local_sha" == "0000000000000000000000000000000000000000" ]] && continue

    case "$local_ref" in
      refs/heads/main|refs/heads/develop)
        if [[ "$remote_sha" == "0000000000000000000000000000000000000000" ]]; then
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
            target_branch="${local_ref#refs/heads/}"
            echo "[git-flow] Blocked: non-merge commit $commit_sha detected in push to '$target_branch'."
            echo "Only merge commits are allowed on protected branches."
            exit 1
          fi
        done < <(git rev-list "$range")
        ;;
    esac
  done
}

case "$mode" in
  commit)
    assert_commit_policy
    ;;
  push)
    assert_push_policy
    ;;
  *)
    echo "Usage: scripts/enforce_git_flow.sh [commit|push]"
    exit 1
    ;;
esac
