#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[tls-check] scanning dependency graph for forbidden TLS backends"

if cargo tree | grep -Ei "native-tls|openssl" >/tmp/kars_tls_scan.txt; then
  echo "[tls-check] forbidden TLS dependency detected:"
  cat /tmp/kars_tls_scan.txt
  exit 1
fi

echo "[tls-check] PASS (rustls-only graph)"
