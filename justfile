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

build-release:
  cargo build --release --locked

build-musl:
  scripts/build_musl.sh

runtime-validate:
  scripts/validate_runtime_matrix.sh

baseline:
  scripts/capture_phase0_baseline.sh

release-dry tag:
  just ci
  scripts/release_tag.sh --dry-run {{tag}}

release-preflight candidate:
  just ci
  scripts/validate_release_flow.sh {{candidate}}

release-plz-preview:
  scripts/release_plz_preview.sh

dist-preview:
  dist plan --target x86_64-unknown-linux-musl --allow-dirty

[confirm("Continue with release tag creation?")]
release-safe tag:
  just doctor-release
  just release-preflight {{tag}}-pre
  just release {{tag}}

[confirm("Continue with direct release tagging?")]
release tag:
  just ci
  scripts/release_tag.sh {{tag}}

list-broken-fmt:
  @cargo fmt --all -- --check --color never | grep "Diff in" | cut -d' ' -f3 || true
