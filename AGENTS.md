## Common Commands
This project uses `mise` for tool management and tasks. Always use `mise` to execute project commands; see `mise.toml` for the available tasks.

## Repo Layout
- Entry point: `src/main.rs`
- CLI parsing: `src/args.rs`
- Top-level command dispatch: `src/app.rs`
- Command handlers: `src/cmds/`
- Config loading: `src/config.rs`
- Domain types and validation: `src/domain/`
- External process and git helpers: `src/ops/`
- Sample config: `src/assets/sample-config.yml`

## Key Conventions
- Keep the CLI shape centered on `clap` subcommands in `src/args.rs` and dispatch from `src/app.rs`.
- Keep config parsing in `src/config.rs`; keep schema and validation rules in `src/domain/config.rs`.
- Reuse `thiserror` enums for user-facing failures; bubble unexpected cases with `anyhow` only where already established.
- Preserve current naming and validation rules for source and update IDs: lowercase, digits, `_`, and `-`.
- Use snapshot tests with `insta` when changing config parsing or validation output.

## Change Checks
- Update snapshots with `mise run update-snapshots`; `mise run review-snapshots` is reserved for human review.
- Keep `cargo-insta` in `mise.toml` and the `insta` dependency in `Cargo.toml` pinned to the same version.
