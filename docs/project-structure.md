# Project Structure

- `src/main.rs`: startup, config validation, preflight checks, task wiring
- `src/config.rs`: config schema + validation
- `src/system.rs`: command execution with timeout/error model
- `src/anomaly_db/`: anomaly model, write/read, retention maintenance
- `src/commands/`: command definitions, router, helpers, feature handlers
- `src/monitor/`: metrics provider, evaluation logic, monitor service

## Code Modularity Policy

- Any Rust source file crossing `200` lines must be split into a folder module (`feature/mod.rs` + focused submodules).
- Any file containing 3 distinct responsibilities (for example: data collection, processing, exporting) must be split similarly.
- Lefthook pre-commit checks block commits that stage `.rs` files above `200` lines.