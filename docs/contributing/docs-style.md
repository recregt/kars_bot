# Docs Style Guide

## Principles

- Keep docs close to code ownership and update in same change.
- Prefer short sections and actionable checklists.
- Separate generated reference docs from manually curated docs.

## Best Practices

- Use deterministic generation for reference docs.
- Avoid duplicate release instructions across files; link to runbooks.
- Keep markdown links relative and resolvable in-repo.
- Keep release metadata (`CHANGELOG.md`, version, tags) synchronized.

## Automation Rules

- Run `scripts/generate_docs_reference.sh` after command/config schema edits.
- Run `scripts/validate_docs.sh` before push.
- Do not manually edit generated reference files.
