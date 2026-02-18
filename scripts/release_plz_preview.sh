#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

temp_branch="tmp/release-plz-preview-$$"
temp_dir="/tmp/release-plz-preview-$$"
prev_tag_dir="/tmp/release-plz-prev-tag-$$"

cleanup() {
  git worktree remove "$temp_dir" --force >/dev/null 2>&1 || true
  git worktree remove "$prev_tag_dir" --force >/dev/null 2>&1 || true
  git branch -D "$temp_branch" >/dev/null 2>&1 || true
}

trap cleanup EXIT

git worktree add -b "$temp_branch" "$temp_dir" HEAD >/dev/null

latest_tag="$(git tag --list 'v*' --sort=-v:refname | head -n 1 || true)"

if [[ -n "$latest_tag" ]]; then
  git worktree add -d "$prev_tag_dir" "$latest_tag" >/dev/null
  (
    cd "$temp_dir"
    release-plz update \
      --config release-plz.toml \
      --allow-dirty \
      --registry-manifest-path "$prev_tag_dir/Cargo.toml"
  )
else
  (
    cd "$temp_dir"
    release-plz update --config release-plz.toml --allow-dirty
  )
fi
