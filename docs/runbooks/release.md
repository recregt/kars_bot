# Release Runbook

## Standard Release

1. Ensure `main` is healthy and required checks are green.
2. Trigger release PR automation (or let push-to-main trigger it):
   - `just release-pr`
3. Review and merge the generated `chore(release): prepare release` PR.
4. Confirm auto-tag created (`vX.Y.Z`) unless `RELEASE_PLZ_AUTO_TAG=false`.
5. Verify the `Release` workflow uploads artifacts and release notes.

## Dry-Run Verification

- `gh workflow run release-plz.yml`

## Post-Release Checks

- Tag points to release commit.
- GitHub Release contains expected artifacts (`musl` archive, installer, checksums, source archive).
- Release notes are coherent with merged features and generated changelog entry.
- Runtime matrix report exists when portability changes are included.
