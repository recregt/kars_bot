#!/usr/bin/env bash
set -euo pipefail

REPO="${REPO:-recregt/kars_bot}"
SERVICE_NAME="${SERVICE_NAME:-kars-bot}"
INSTALL_DIR="${INSTALL_DIR:-/opt/kars_bot}"
BIN_PATH="${BIN_PATH:-$INSTALL_DIR/target/release/kars_bot}"
ASSET_PATTERN='x86_64-unknown-linux-musl.tar.xz$'
CHECKSUM_ASSET_PATTERN='sha256.sum$'
API_URL="https://api.github.com/repos/${REPO}/releases/latest"
LOCK_FILE="${LOCK_FILE:-/tmp/kars_bot_update.lock}"
STARTUP_SETTLE_SECS="${STARTUP_SETTLE_SECS:-2}"
HEALTH_RETRY_COUNT="${HEALTH_RETRY_COUNT:-10}"
HEALTH_RETRY_SLEEP_SECS="${HEALTH_RETRY_SLEEP_SECS:-1}"
HEALTH_STABLE_CHECKS="${HEALTH_STABLE_CHECKS:-2}"

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
require_cmd sha256sum
require_cmd df

exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  echo "[update] Another update is already running."
  exit 1
fi

check_apply_ready() {
  if [[ ! -d "$(dirname "$BIN_PATH")" ]]; then
    if ! install -d "$(dirname "$BIN_PATH")" >/dev/null 2>&1; then
      echo "[update] Cannot create target directory: $(dirname "$BIN_PATH")"
      return 1
    fi
  fi

  if [[ -f "$BIN_PATH" && ! -w "$BIN_PATH" ]]; then
    echo "[update] Binary path is not writable: $BIN_PATH"
    return 1
  fi

  if ! systemctl status "$SERVICE_NAME" >/dev/null 2>&1; then
    echo "[update] Service not found or inaccessible: $SERVICE_NAME"
    return 1
  fi

  return 0
}

if [[ "$CHECK_ONLY" -eq 1 ]]; then
  if check_apply_ready; then
    echo "APPLY_READY=1"
    echo "SERVICE_NAME=$SERVICE_NAME"
    echo "BIN_PATH=$BIN_PATH"
    exit 0
  fi
  echo "APPLY_READY=0"
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

ensure_tmp_space_for_download() {
  local required_bytes="$1"
  local safety_bytes=$((50 * 1024 * 1024))
  local total_required_bytes=$((required_bytes + safety_bytes))
  local available_kb
  local required_kb

  available_kb="$(df -Pk "$tmpdir" | awk 'NR==2 {print $4}')"
  required_kb="$(((total_required_bytes + 1023) / 1024))"

  if (( available_kb < required_kb )); then
    echo "[update] Not enough free space in temp filesystem."
    echo "[update] Required: ${required_kb}KB (asset + safety), available: ${available_kb}KB"
    return 1
  fi

  return 0
}

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
asset_size_bytes="$(jq -r --arg pattern "$ASSET_PATTERN" '.assets[] | select(.name | test($pattern)) | .size' "$release_json" | head -n1)"

if [[ -z "$asset_url" || -z "$asset_name" || -z "$asset_size_bytes" ]]; then
  echo "[update] Could not find musl release asset matching pattern: $ASSET_PATTERN"
  exit 1
fi

if ! [[ "$asset_size_bytes" =~ ^[0-9]+$ ]]; then
  echo "[update] Invalid asset size in release metadata: $asset_size_bytes"
  exit 1
fi

if ! ensure_tmp_space_for_download "$asset_size_bytes"; then
  exit 1
fi

echo "[update] Latest tag: $tag_name"
echo "[update] Downloading asset: $asset_name"
asset_path="$tmpdir/$asset_name"
curl -fL "$asset_url" -o "$asset_path"

checksum_url="$(jq -r --arg pattern "$CHECKSUM_ASSET_PATTERN" '.assets[] | select(.name | test($pattern)) | .browser_download_url' "$release_json" | head -n1)"
checksum_name="$(jq -r --arg pattern "$CHECKSUM_ASSET_PATTERN" '.assets[] | select(.name | test($pattern)) | .name' "$release_json" | head -n1)"

if [[ -n "$checksum_url" && -n "$checksum_name" ]]; then
  checksum_path="$tmpdir/$checksum_name"
  echo "[update] Downloading checksums: $checksum_name"
  curl -fL "$checksum_url" -o "$checksum_path"

  expected_hash="$(awk -v name="$asset_name" 'index($0, name) { print $1; exit }' "$checksum_path")"
  if [[ -z "$expected_hash" ]]; then
    echo "[update] Checksum entry not found for $asset_name in $checksum_name"
    exit 1
  fi

  echo "[update] Verifying SHA256 checksum..."
  echo "$expected_hash  $asset_path" | sha256sum -c -
else
  echo "[update] WARNING: No checksum asset found; skipping SHA256 verification."
fi

echo "[update] Extracting binary..."
if ! tar -xJf "$asset_path" -C "$tmpdir" --strip-components=1; then
  tar -xJf "$asset_path" -C "$tmpdir"
fi

if [[ ! -f "$tmpdir/kars_bot" ]]; then
  extracted_binary="$(find "$tmpdir" -type f -name kars_bot -print -quit)"
  if [[ -z "$extracted_binary" ]]; then
    echo "[update] Extracted archive does not contain kars_bot binary."
    exit 1
  fi

  install -m 0755 "$extracted_binary" "$tmpdir/kars_bot"
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

sleep "$STARTUP_SETTLE_SECS"

active_stable_count=0
for ((i = 1; i <= HEALTH_RETRY_COUNT; i++)); do
  if systemctl --quiet is-active "$SERVICE_NAME"; then
    active_stable_count=$((active_stable_count + 1))
    if (( active_stable_count >= HEALTH_STABLE_CHECKS )); then
      break
    fi
  else
    active_stable_count=0
  fi

  sleep "$HEALTH_RETRY_SLEEP_SECS"
done

if (( active_stable_count < HEALTH_STABLE_CHECKS )); then
  echo "[update] Service health check failed after restart."
  rollback_if_needed
  exit 1
fi

systemctl --no-pager --full status "$SERVICE_NAME" | sed -n '1,20p'

echo "[update] Update complete -> $tag_name"
