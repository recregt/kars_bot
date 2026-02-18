#!/usr/bin/env bash
set -euo pipefail

target="x86_64-unknown-linux-musl"
bin_name="kars_bot"

echo "[musl] Ensuring Rust target: ${target}"
rustup target add "${target}" >/dev/null

if ! command -v musl-gcc >/dev/null 2>&1; then
  echo "[musl] musl-gcc not found. Install musl tools first."
  echo "       Ubuntu/Debian: sudo apt-get update && sudo apt-get install -y musl-tools"
  exit 1
fi

echo "[musl] Building release binary"
cargo build --release --target "${target}"

artifact="target/${target}/release/${bin_name}"
if [[ ! -f "${artifact}" ]]; then
  echo "[musl] Build finished but artifact not found: ${artifact}"
  exit 1
fi

echo "[musl] Build complete: ${artifact}"
