# Documentation Index

This directory is the single source of truth for technical and operational documentation.

## Structure

- [Quick Start](quickstart.md)
- [Operations](operations.md)
- [Runtime Notes](runtime.md)
- [Project Structure](project-structure.md)
- [Roadmap](ROADMAP.md)
- [Architecture](architecture/overview.md)
- [Runbooks](runbooks/release.md)
- [Reference](reference/commands.md)
- [Releases](releases/)
- [Docs Style Guide](contributing/docs-style.md)

## Automation

- Generate command/config references: `scripts/generate_docs_reference.sh`
- Validate links and generated docs: `scripts/validate_docs.sh`
- Preview release changes: `just release-plz-preview` and `just dist-preview`

## Ownership

- Code-adjacent docs must be updated in the same PR as code changes.
- Release version/changelog updates are produced by the release-plz PR flow; release docs should follow that automated output.
