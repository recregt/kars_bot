#!/usr/bin/env bash

log_prefix() {
  local script_name
  script_name="${SCRIPT_NAME:-$(basename "$0")}" 
  printf '[%s]' "$script_name"
}

log_info() {
  printf '%s %s\n' "$(log_prefix)" "$*"
}

log_warn() {
  printf '%s WARN: %s\n' "$(log_prefix)" "$*" >&2
}

log_error() {
  printf '%s ERROR: %s\n' "$(log_prefix)" "$*" >&2
}

log_die() {
  log_error "$*"
  exit 1
}
