#!/usr/bin/env bash
set -euo pipefail

REPO="${REPO:-recregt/kars_bot}"
SERVICE_NAME="${SERVICE_NAME:-kars-bot}"
INSTALL_DIR="${INSTALL_DIR:-/opt/kars_bot}"
BIN_PATH="${BIN_PATH:-$INSTALL_DIR/target/release/kars_bot}"
ASSET_PATTERN='x86_64-unknown-linux-musl.tar.gz$'
API_URL="https://api.github.com/repos/${REPO}/releases/latest"
LOCK_FILE="${LOCK_FILE:-/tmp/kars_bot_update.lock}"

CHECK_ONLY=0
if [[ "${1:-}" == "--check-only" ]]; then
  CHECK_ONLY=1
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[update] Missing required command: $1"
    exit 1
  fi
}

require_cmd curl
require_cmd jq
require_cmd tar
require_cmd install
require_cmd systemctl
require_cmd flock

exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  echo "CHECK_LOCK=busy"
  echo "APPLY_READY=0"
  echo "DETAIL=another update is already running"
  echo "[update] Another update is already running."
  exit 1
fi

emit_check() {
  local key="$1"
  local value="$2"
  echo "${key}=${value}"
}

check_apply_ready() {
  local target_dir
  target_dir="$(dirname "$BIN_PATH")"
  local ready=1

  emit_check "CHECK_LOCK" "ok"
  emit_check "CHECK_BIN_PATH" "$BIN_PATH"
  emit_check "CHECK_SERVICE" "$SERVICE_NAME"

  if [[ ! -d "$target_dir" ]]; then
    emit_check "CHECK_TARGET_DIR" "missing:$target_dir"
    ready=0
  else
    emit_check "CHECK_TARGET_DIR" "ok:$target_dir"
    if [[ ! -w "$target_dir" ]]; then
      emit_check "CHECK_TARGET_DIR_WRITABLE" "fail"
      ready=0
    else
      emit_check "CHECK_TARGET_DIR_WRITABLE" "ok"
    fi
  fi

  if [[ -f "$BIN_PATH" ]]; then
    if [[ ! -w "$BIN_PATH" ]]; then
      emit_check "CHECK_BIN_WRITABLE" "fail"
      ready=0
    else
      emit_check "CHECK_BIN_WRITABLE" "ok"
    fi
  else
    emit_check "CHECK_BIN_WRITABLE" "n/a_missing_bin"
  fi

  if ! systemctl status "$SERVICE_NAME" >/dev/null 2>&1; then
    emit_check "CHECK_SYSTEMD_SERVICE" "fail"
    ready=0
  else
    emit_check "CHECK_SYSTEMD_SERVICE" "ok"
  fi

  if [[ "$ready" -eq 1 ]]; then
    emit_check "APPLY_READY" "1"
    return 0
  fi

  emit_check "APPLY_READY" "0"
  return 1
}

if [[ "$CHECK_ONLY" -eq 1 ]]; then
  if check_apply_ready; then
    emit_check "SERVICE_NAME" "$SERVICE_NAME"
    emit_check "BIN_PATH" "$BIN_PATH"
    exit 0
  fi
  echo "[update] Apply readiness checks failed."
  exit 1
fi

if ! check_apply_ready; then
  exit 1
fi

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

rollback_if_needed() {
  if [[ -f "$BIN_PATH.bak" ]]; then
    echo "[update] Rolling back to backup binary..."
    install -m 0755 "$BIN_PATH.bak" "$BIN_PATH"
    systemctl restart "$SERVICE_NAME" || true
  fi
}

trap 'echo "[update] Interrupted by signal."; rollback_if_needed; exit 130' INT TERM

echo "[update] Fetching latest release metadata from ${REPO}..."
release_json="$tmpdir/release.json"
curl -fsSL "$API_URL" -o "$release_json"

tag_name="$(jq -r '.tag_name // empty' "$release_json")"
if [[ -z "$tag_name" ]]; then
  echo "[update] Could not read latest release tag."
  exit 1
fi

asset_url="$(jq -r --arg pattern "$ASSET_PATTERN" '.assets[] | select(.name | test($pattern)) | .browser_download_url' "$release_json" | head -n1)"
asset_name="$(jq -r --arg pattern "$ASSET_PATTERN" '.assets[] | select(.name | test($pattern)) | .name' "$release_json" | head -n1)"

if [[ -z "$asset_url" || -z "$asset_name" ]]; then
  echo "[update] Could not find musl release asset matching pattern: $ASSET_PATTERN"
  exit 1
fi

echo "[update] Latest tag: $tag_name"
echo "[update] Downloading asset: $asset_name"
asset_path="$tmpdir/$asset_name"
curl -fL "$asset_url" -o "$asset_path"

echo "[update] Extracting binary..."
tar -xzf "$asset_path" -C "$tmpdir"
if [[ ! -f "$tmpdir/kars_bot" ]]; then
  echo "[update] Extracted archive does not contain kars_bot binary."
  exit 1
fi

if [[ ! -x "$tmpdir/kars_bot" ]]; then
  chmod +x "$tmpdir/kars_bot"
fi

if ! "$tmpdir/kars_bot" --version >/dev/null 2>&1; then
  echo "[update] Downloaded binary failed basic execution check (--version)."
  exit 1
fi

echo "[update] Installing binary to: $BIN_PATH"
install -d "$(dirname "$BIN_PATH")"
if [[ -f "$BIN_PATH" ]]; then
  cp "$BIN_PATH" "$BIN_PATH.bak"
fi

tmp_install_path="$BIN_PATH.new"
install -m 0755 "$tmpdir/kars_bot" "$tmp_install_path"
mv -f "$tmp_install_path" "$BIN_PATH"

echo "[update] Restarting service: $SERVICE_NAME"
if ! systemctl restart "$SERVICE_NAME"; then
  echo "[update] Service restart failed."
  rollback_if_needed
  exit 1
fi

if ! systemctl --quiet is-active "$SERVICE_NAME"; then
  echo "[update] Service health check failed after restart."
  rollback_if_needed
  exit 1
fi

systemctl --no-pager --full status "$SERVICE_NAME" | sed -n '1,20p'

echo "[update] Update complete -> $tag_name"
