set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

install-hooks:
  scripts/install_hooks.sh

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test

quality:
  just fmt-check
  just clippy
  just test
  scripts/check_tls_stack.sh

docs:
  scripts/generate_docs_reference.sh
  scripts/validate_docs.sh

build-release:
  cargo build --release

build-musl:
  scripts/build_musl.sh

runtime-validate:
  scripts/validate_runtime_matrix.sh

baseline:
  scripts/capture_phase0_baseline.sh

release tag:
  scripts/release_tag.sh {{tag}}

release-dry tag:
  scripts/release_tag.sh --dry-run {{tag}}
