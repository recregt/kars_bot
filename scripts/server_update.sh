#!/usr/bin/env bash
set -euo pipefail

REPO="${REPO:-recregt/kars_bot}"
SERVICE_NAME="${SERVICE_NAME:-kars-bot}"
INSTALL_DIR="${INSTALL_DIR:-/opt/kars_bot}"
BIN_PATH="${BIN_PATH:-$INSTALL_DIR/target/release/kars_bot}"
ASSET_PATTERN='x86_64-unknown-linux-musl.tar.gz$'
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

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

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

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

echo "[update] Installing binary to: $BIN_PATH"
install -d "$(dirname "$BIN_PATH")"
if [[ -f "$BIN_PATH" ]]; then
  cp "$BIN_PATH" "$BIN_PATH.bak"
fi
install -m 0755 "$tmpdir/kars_bot" "$BIN_PATH"

echo "[update] Restarting service: $SERVICE_NAME"
systemctl restart "$SERVICE_NAME"
systemctl --no-pager --full status "$SERVICE_NAME" | sed -n '1,20p'

echo "[update] Update complete -> $tag_name"
