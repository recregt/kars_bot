#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

GLIBC_BIN="target/release/kars_bot"
MUSL_BIN="target/x86_64-unknown-linux-musl/release/kars_bot"
REPORT_DIR="docs/releases"
REPORT_FILE="$REPORT_DIR/runtime-validation-report.txt"

mkdir -p "$REPORT_DIR"

echo "[runtime-validation] Building glibc binary"
cargo build --release

echo "[runtime-validation] Building musl binary"
scripts/build_musl.sh

if [[ ! -x "$GLIBC_BIN" ]]; then
  echo "[runtime-validation] missing glibc binary: $GLIBC_BIN"
  exit 1
fi

if [[ ! -x "$MUSL_BIN" ]]; then
  echo "[runtime-validation] missing musl binary: $MUSL_BIN"
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cat >"$TMP_DIR/config.toml" <<'EOF'
bot_token = "123456:telegram-bot-token"
owner_id = 123456789
monitor_interval = 30
command_timeout_secs = 30

[alerts]
cpu = 85.0
ram = 90.0
disk = 90.0
cooldown_secs = 300
hysteresis = 5.0

[daily_summary]
enabled = false
hour_utc = 9
minute_utc = 0

[weekly_report]
enabled = false
weekday_utc = 1
hour_utc = 9
minute_utc = 0

[graph]
enabled = true
default_window_minutes = 60
max_window_hours = 24
max_points = 1200

[anomaly_db]
enabled = true
dir = "logs"
max_file_size_bytes = 10485760
retention_days = 7

[simulation]
enabled = false
profile = "wave"

[reporting_store]
enabled = false
path = "data/reporting_store"
retention_days = 30

[release_notifier]
enabled = false
changelog_path = "CHANGELOG.md"
state_path = "data/release_notifier/state.json"

[security]
redact_sensitive_output = false
EOF

run_smoke() {
  local label="$1"
  local bin_path="$2"
  local log_file="$TMP_DIR/${label}.log"

  echo "[runtime-validation] Smoke run: $label"
  set +e
  (
    cd "$TMP_DIR"
    timeout 6s "$ROOT_DIR/$bin_path" >"$log_file" 2>&1
  )
  local code=$?
  set -e

  if [[ "$code" -ne 0 && "$code" -ne 124 ]]; then
    if grep -Eq "Failed to retrieve 'me': Api\(NotFound\)|Failed to get webhook info: Api\(NotFound\)" "$log_file"; then
      echo "[runtime-validation] $label smoke reached Telegram auth stage (expected with dummy token)"
      return 0
    fi

    echo "[runtime-validation] $label smoke failed with exit code $code"
    sed -n '1,80p' "$log_file"
    return 1
  fi

  return 0
}

run_smoke "glibc" "$GLIBC_BIN"
run_smoke "musl" "$MUSL_BIN"

{
  echo "Runtime Validation Report"
  echo "generated_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo ""
  echo "glibc_file=$(file "$GLIBC_BIN")"
  echo "musl_file=$(file "$MUSL_BIN")"
  echo ""
  echo "glibc_ldd=$(ldd "$GLIBC_BIN" 2>&1 | tr '\n' ' ' )"
  echo "musl_ldd=$(ldd "$MUSL_BIN" 2>&1 | tr '\n' ' ' )"
  echo ""
  echo "status=PASS"
} >"$REPORT_FILE"

echo "[runtime-validation] PASS"
echo "[runtime-validation] report: $REPORT_FILE"
