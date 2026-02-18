#!/usr/bin/env bash

git_scope_fetch_main() {
  git fetch --no-tags --depth=1 origin main >/dev/null 2>&1 || true
}

git_scope_base_ref() {
  git_scope_fetch_main

  if git rev-parse --verify origin/main >/dev/null 2>&1; then
    git merge-base HEAD origin/main || git rev-parse HEAD~1 2>/dev/null || git rev-parse HEAD
    return
  fi

  git rev-parse HEAD~1 2>/dev/null || git rev-parse HEAD
}

git_scope_changed_files() {
  local base_ref="$1"
  shift || true
  git diff --name-only "$base_ref...HEAD" -- "$@"
}

git_scope_changed_files_space() {
  local base_ref="$1"
  shift || true
  git_scope_changed_files "$base_ref" "$@" | tr '\n' ' '
}

git_scope_changed_rust_files_space() {
  local base_ref="$1"
  git_scope_changed_files_space "$base_ref" '*.rs'
}

git_scope_is_rust_related() {
  local changed_all="$1"
  if grep -Eq '(^| )(Cargo.toml|Cargo.lock|build.rs|rust-toolchain(\.toml)?|\.cargo/|scripts/check_tls_stack\.sh|.*\.rs)( |$)' <<<"$changed_all"; then
    return 0
  fi
  return 1
}

git_scope_push_range() {
  local local_sha="$1"
  local remote_sha="$2"

  if [[ "$remote_sha" == "0000000000000000000000000000000000000000" ]]; then
    if git rev-parse --verify refs/remotes/origin/main >/dev/null 2>&1; then
      local base_sha
      base_sha="$(git merge-base "$local_sha" refs/remotes/origin/main)"
      printf '%s..%s\n' "$base_sha" "$local_sha"
      return
    fi

    local empty_tree
    empty_tree="$(git hash-object -t tree /dev/null)"
    printf '%s..%s\n' "$empty_tree" "$local_sha"
    return
  fi

  printf '%s..%s\n' "$remote_sha" "$local_sha"
}
