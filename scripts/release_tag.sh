#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/release_tag.sh vX.Y.Z
  scripts/release_tag.sh --dry-run vX.Y.Z

Behavior:
- Runs tests before any release mutation.
- Updates Cargo.toml version only when needed.
- Generates/updates CHANGELOG.md via git-cliff (mandatory).
- Captures release binary size in docs/releases/binary-size.csv.
EOF
}

dry_run=0
if [[ $# -eq 2 && "$1" == "--dry-run" ]]; then
  dry_run=1
  shift
fi

if [[ $# -ne 1 ]]; then
  usage
  exit 1
fi

tag="$1"
if [[ ! "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid tag format: $tag (expected vX.Y.Z)"
  exit 1
fi

version="${tag#v}"

if ! command -v git-cliff >/dev/null 2>&1; then
  echo "git-cliff is required for changelog generation."
  echo "Install: cargo install git-cliff"
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree is not clean. Commit/stash changes before release."
  exit 1
fi

if git rev-parse "$tag" >/dev/null 2>&1; then
  echo "Tag already exists: $tag"
  exit 1
fi

echo "Running test gate before release..."
cargo test -q

if [[ -x scripts/validate_docs.sh ]]; then
  echo "Running docs validation gate..."
  scripts/validate_docs.sh
fi

echo "Building release binary for size snapshot..."
cargo build --release -q

binary_path="target/release/kars_bot"
if [[ ! -f "$binary_path" ]]; then
  echo "Release binary not found at $binary_path"
  exit 1
fi

size_bytes=$(stat -c%s "$binary_path")
size_human=$(numfmt --to=iec --suffix=B "$size_bytes")

mkdir -p docs/releases
if [[ ! -f docs/releases/binary-size.csv ]]; then
  echo "timestamp_utc,tag,size_bytes,size_human" > docs/releases/binary-size.csv
fi

timestamp_utc=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if [[ "$dry_run" -eq 1 ]]; then
  echo "[dry-run] Tests/build passed. Previewing changelog section for $tag"
  git-cliff --unreleased --tag "$tag" | head -n 60
  echo "[dry-run] Binary size: $size_human ($size_bytes bytes)"
  echo "[dry-run] No files changed, no commit created, no tag created."
  exit 0
fi

current_version=$(awk -F '"' '/^version = /{print $2; exit}' Cargo.toml)
if [[ "$current_version" != "$version" ]]; then
  sed -i "s/^version = \".*\"/version = \"$version\"/" Cargo.toml
fi

echo "Generating changelog with git-cliff..."
if [[ -f CHANGELOG.md ]]; then
  git-cliff --unreleased --tag "$tag" --prepend CHANGELOG.md
else
  git-cliff --unreleased --tag "$tag" > CHANGELOG.md
fi

if git diff --quiet -- CHANGELOG.md; then
  echo "CHANGELOG.md was not updated by git-cliff."
  echo "Release aborted: changelog update is mandatory in release flow."
  exit 1
fi

if [[ ! -s CHANGELOG.md ]]; then
  echo "CHANGELOG.md generation failed or resulted in empty output."
  exit 1
fi

if ! grep -q "^## $tag" CHANGELOG.md; then
  echo "CHANGELOG validation failed: missing section header '## $tag'."
  echo "Release aborted to keep changelog and tag history synchronized."
  exit 1
fi

echo "$timestamp_utc,$tag,$size_bytes,$size_human" >> docs/releases/binary-size.csv

git add Cargo.toml CHANGELOG.md docs/releases/binary-size.csv

if [[ -n "$(git diff --cached --name-only)" ]]; then
  ALLOW_VERSION_BUMP=1 git commit -m "chore(release): prepare $tag"
fi

git tag -a "$tag" -m "Release $tag"
echo "Created tag $tag"
