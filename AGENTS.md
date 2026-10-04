# Project guidance

Build a minimal Rust todo TUI with Ratatui and SQLite. Prefer small, direct changes and few dependencies. Keep existing features unless the user asks to change them; avoid adding features beyond the request.

## Interface

- Use the full terminal with small edge margins, readable text, and restrained colors.
- Keep tasks in one view. Show parents and children together with indentation; avoid tabs, separate task pages, dashboards, and large panels.
- `j/k` selects tasks. `h` selects the parent. `l` selects a child or starts creating one.
- Keep input hints short and adapt the layout to narrow terminals.
- Keep help text and `README.md` consistent with the actual keybindings.

## Structure

- `src/main.rs`: startup and terminal cleanup.
- `src/app.rs`: task state and action handling.
- `src/ui.rs`: Ratatui rendering.
- `src/db.rs`: SQLite queries and schema migrations.
- `src/cli.rs`: command-line options and database location.
- `src/vim_motion/`: keyboard mapping, modes, actions, and Unicode text editing. Route keyboard input through this module.

## Storage

- Use `dirs::config_dir()/argv-todo/db.sql`: Linux `~/.config`, macOS `~/Library/Application Support`, and Windows roaming AppData. Honor `--db` overrides.
- Save changes immediately with parameterized SQL.
- Migrate existing databases without losing tasks or parent links.
- Delete and restore task trees atomically. Keep the terminal usable after errors.
- Use temporary or in-memory databases for tests; never modify the user's database during verification.

## Validation

For code changes, run the relevant tests and these checks:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build
```

Avoid redundant tests and unrelated refactors. Preserve uncommitted user changes. Commit only when requested.

## Development servers

- Never start, restart, stop, or kill a development server unless the user explicitly requests it.
- Assume the user manages development servers. Use an existing server for verification; if unavailable, report the limitation and continue with other checks.
- Do not request permission to launch a development server as a routine testing step.
