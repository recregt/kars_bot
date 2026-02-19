# Release Runbook

## Standard Release

1. Ensure `main` is healthy and required checks are green.
2. Trigger release PR automation (or let push-to-main trigger it):
   - `just release-pr`
3. Review and merge the generated `chore(release): prepare release` PR.
4. Push/create `vX.Y.Z` tag matching merged Cargo version.
5. Verify the `Release` workflow uploads artifacts and release notes.

## Dry-Run Verification

- `just release-plz-preview`
- `just dist-preview`

## Post-Release Checks

- Tag points to release commit.
- GitHub Release contains expected artifacts (`musl` archive, installer, checksums, source archive).
- Release notes are coherent with merged features and generated changelog entry.
- Runtime matrix report exists when portability changes are included.
