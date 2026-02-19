# Release Runbook

- Owner: Release Engineering
- Last validated: 2026-02-20
- Validation cadence: each production release

## Standard Release

1. Ensure `main` is healthy and required checks are green.
2. Run pre-release chaos drills:
   - `just chaos-pre-release`
3. Trigger release PR automation (or let push-to-main trigger it):
   - `just release-pr`
4. Review and merge the generated `chore(release): prepare release` PR.
5. Push/create `vX.Y.Z` tag matching merged Cargo version.
6. Verify the `Release` workflow uploads artifacts and release notes.

## Dry-Run Verification

- `just release-plz-preview`
- `just dist-preview`

## Post-Release Checks

- Tag points to release commit.
- GitHub Release contains expected artifacts (`musl` archive, installer, checksums, source archive).
- Release notes are coherent with merged features and generated changelog entry.
- Runtime matrix report exists when portability changes are included.
