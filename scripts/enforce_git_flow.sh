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
            if [[ "$target_branch" == "main" ]]; then
              subject="$(git log -1 --pretty=%s "$commit_sha")"
              if [[ "$subject" =~ ^chore\(release\):\ prepare\ v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                continue
              fi
            fi
            echo "[git-flow] Blocked: non-merge commit $commit_sha detected in push to '$target_branch'."
            echo "Only first-parent merge commits are allowed on protected branches."
            exit 1
          fi
        done < <(git rev-list --first-parent "$range")
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
