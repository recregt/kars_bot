set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

install-hooks:
  scripts/install_hooks.sh

bootstrap:
  scripts/install_hooks.sh
  just doctor

sync:
  @echo "🔄 Fetching latest refs..."
  git fetch --all --prune
  @echo "⬇️  Syncing main..."
  git switch main
  git pull origin main
  @echo "⬇️  Syncing develop..."
  git switch develop
  git pull origin develop
  @echo "✨ Local environment is up-to-date!"

doctor:
  scripts/doctor.sh

doctor-release:
  scripts/doctor.sh --release

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --locked --all-targets --all-features -- -D warnings

test:
  cargo test --locked

nextest:
  cargo nextest run --locked --all-targets

quality:
  just fmt-check
  just clippy
  just test
  scripts/check_tls_stack.sh

ci:
  just doctor
  scripts/ci_local.sh

docs:
  scripts/generate_docs_reference.sh
  scripts/validate_docs.sh

runbook-gate:
  scripts/verify_runbook_gate.sh

chaos-pre-release:
  scripts/pre_release_chaos_checks.sh

build-release:
  cargo build --release --locked

build-musl:
  scripts/build_musl.sh

runtime-validate:
  scripts/validate_runtime_matrix.sh

baseline:
  scripts/capture_phase0_baseline.sh

release-plz-preview:
  scripts/release_plz_preview.sh

dist-preview:
  dist plan --target x86_64-unknown-linux-musl --allow-dirty

release-pr:
  gh workflow run release-plz.yml

release-dispatch tag:
  gh workflow run release.yml --ref main -f tag={{tag}}

list-broken-fmt:
  @cargo fmt --all -- --check --color never | grep "Diff in" | cut -d' ' -f3 || true
