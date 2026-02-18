# Release Runbook

## Standard Release

1. Ensure clean tree and up-to-date branch.
2. Run release flow:
   - `scripts/release_tag.sh vX.Y.Z`
3. Verify:
   - `CHANGELOG.md` contains `## vX.Y.Z`
   - `Cargo.toml` and `Cargo.lock` package version match `X.Y.Z`
   - `docs/releases/binary-size.csv` has a new row

## Dry-Run Verification

- `scripts/release_tag.sh --dry-run vX.Y.Z`
- `scripts/validate_release_flow.sh vX.Y.Z-pre`

## Post-Release Checks

- Tag points to release commit.
- Release notes are coherent with merged features.
- Runtime matrix report exists when portability changes are included.
