## Common Commands
- Prefer `just` over raw `cargo` commands.
- Check: `just check`
- Format: `just fmt`
- Lint: `just lint`
- Test: `just test`
- Full local sweep: `just all`
- Run the CLI: `just run -- <ARGS>`

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
- Run the smallest relevant command first, then `just all` if the change touches multiple paths.
- If snapshots change, review them with `just review` before finishing.
