set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

install-hooks:
  scripts/install_hooks.sh

bootstrap:
  scripts/install_hooks.sh
  just doctor

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

build-release:
  cargo build --release --locked

build-musl:
  scripts/build_musl.sh

runtime-validate:
  scripts/validate_runtime_matrix.sh

baseline:
  scripts/capture_phase0_baseline.sh

release tag:
  just ci
  scripts/release_tag.sh {{tag}}

release-dry tag:
  just ci
  scripts/release_tag.sh --dry-run {{tag}}

release-preflight candidate:
  just ci
  scripts/validate_release_flow.sh {{candidate}}

release-safe tag:
  just doctor-release
  just release-preflight {{tag}}-pre
  just release {{tag}}
