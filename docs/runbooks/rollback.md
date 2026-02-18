# Rollback Runbook

## When to Roll Back

- Critical runtime regressions in production.
- Release automation mismatch (tag/version/changelog divergence).
- Broken command safety behavior (authorization, redaction, output fallback).

## Procedure

1. Identify last known-good tag:
   - `git tag --list 'v*' | sort -V`
2. Prepare hotfix/rollback branch from the known-good tag.
3. Redeploy binary from known-good release artifact.
4. Verify bot health commands:
   - `/health`, `/status`, `/alerts`
5. Record rollback reason and recovery plan in release notes/runbook update.

## Rollback Guardrails

- Never rewrite shared history unless coordinated.
- Keep changelog/tag/version synchronized after rollback follow-up release.
