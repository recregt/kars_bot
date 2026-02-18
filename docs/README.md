# Documentation Index

This directory is the single source of truth for technical and operational documentation.

## Structure

- [Roadmap](ROADMAP.md)
- [Architecture](architecture/overview.md)
- [Runbooks](runbooks/release.md)
- [Reference](reference/commands.md)
- [Releases](releases/)
- [Docs Style Guide](contributing/docs-style.md)

## Automation

- Generate command/config references: `scripts/generate_docs_reference.sh`
- Validate links and generated docs: `scripts/validate_docs.sh`
- Validate release flow (dry-run): `scripts/validate_release_flow.sh v1.1.1-pre`

## Ownership

- Code-adjacent docs must be updated in the same PR as code changes.
- Release docs are updated via `scripts/release_tag.sh` and must not be edited manually during release tagging.
