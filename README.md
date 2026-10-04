# argv-todo

A small terminal todo app with Vim-style keys, nested tasks, and local SQLite storage. Built with Rust, Ratatui, and Crossterm.

- Keep parents and children together in one task tree.
- Create and edit tasks inline, with three priority levels and saved sibling order.
- Search titles, complete subtrees, and undo deletions.
- Use Unicode text, bracketed paste, and your terminal's color palette.

[Contributing](CONTRIBUTING.md) · [Support](SUPPORT.md) · [Changelog](CHANGELOG.md) · [MIT license](LICENSE)

## Install from source

A current stable Rust toolchain and a C compiler are required to build. SQLite is bundled; no SQLite server or system SQLite installation is required.

```sh
git clone https://github.com/argv-tech/argv-todo.git
cd argv-todo
cargo install --path . --locked
argv-todo
```

Ensure Cargo's binary directory is on your `PATH` (`~/.cargo/bin` on Linux and macOS, `%USERPROFILE%\\.cargo\\bin` on Windows). The clone command will work after the GitHub repository is created. No crates.io release is available from this setup yet.

## Run

```sh
cargo run --locked
```

An interactive terminal is required. The app uses the full terminal and shows task rows at 35 columns × 12 rows or larger. Press `?` for help and `q` to quit.

## Storage

Changes are saved immediately. The app creates its config directory, `config.toml`, and database on first launch:

| Platform | Default database |
| --- | --- |
| Linux | `~/.config/argv-todo/db.sql` |
| macOS | `~/Library/Application Support/argv-todo/db.sql` |
| Windows | `%APPDATA%\argv-todo\db.sql` |

Linux honors `XDG_CONFIG_HOME`. Despite its `.sql` extension, `db.sql` is a binary SQLite database. SQLite may create `db.sql-wal` and `db.sql-shm` beside it while running.

For an existing database, pass its current path with `--db` or move it to the new default location before launching.

Close the app before moving or backing up a database. Keep any remaining `db.sql-wal` and `db.sql-shm` files together with `db.sql`. Task data is stored without application-level encryption.

Override the location when needed:

```sh
argv-todo --db /path/to/tasks.sql
argv-todo --help
argv-todo --version
```

## Configuration

`config.toml` lives beside the default `db.sql` in the platform config directory. With `--db`, the app reads or creates `config.toml` beside the specified database instead, keeping development settings separate.

The generated config contains:

```toml
database_path = "db.sql"
```

Set `database_path` to an absolute path or a path relative to the config's directory. `--db` takes precedence over this setting. Changing the setting selects a database; it does not move existing tasks. The config stays in its original directory even when the database path points elsewhere.

The app reads settings on startup and preserves existing config files, including comments. An empty config uses `db.sql`. Invalid TOML, unknown settings, and invalid database paths produce an error before the terminal interface opens. `--help` and `--version` do not create files.

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` | Next / previous row in the task tree |
| `l` | Select the first child; stay selected if there are none |
| `h` | Select the parent task |
| `gg` / `G` | First / last task |
| `3j`, `2k` | Repeat a motion |
| `i`, `a` | Add a child of the selected task; add a root task when the list is empty |
| `o`, `O` | Add a sibling below or above the selected task; add a root task when the list is empty |
| `e`, `cc` | Edit selected task |
| `t` | Cycle priority: low → mid → high → low |
| `ph`, `pm`, `pl` | Set priority to high, mid, or low |
| `Ctrl-p` | Cycle priority, including while creating or editing inline |
| `Space`, `x`, `Enter` | Toggle the selected task and all its descendants in normal mode |
| `dd`, `3dd` | Delete one / three tasks and all their children starting at selection |
| `u` | Restore the last deleted task tree(s) in the current session |
| `/` | Search all titles, ignoring case; keep ancestors visible for context |
| `?` | Show help |
| `Esc` | Cancel input, close help, or clear applied search |
| `Ctrl-r` | Reload tasks from disk |
| `q`, `Ctrl-c` | Quit |

In insert and search modes, letters are text. `Enter` saves or applies search. Arrow keys, `Home`, and `End` move the input cursor; `Ctrl-Left` / `Ctrl-Right` move by word. `Backspace`, `Delete`, `Ctrl-w`, and `Ctrl-u` edit text. Bracketed paste is supported. Text editing handles Unicode graphemes and wide characters.

Tasks can contain nested child tasks. Existing databases are upgraded automatically, keeping existing tasks at the top level. Deleting a parent also deletes its descendants; `u` restores the entire tree. Completing or reopening a parent gives all its descendants the same state, including tasks hidden by search. Toggling a child affects its own subtree without changing its ancestors or siblings. Changes are saved atomically.

Tasks have **high**, **mid**, or **low** priority; new roots and existing tasks default to **mid**. New children inherit their parent's priority. Siblings created with `o` or `O` start with the selected task's priority so they appear beside it. Roots and children within each parent are sorted high first, then mid, then low. Equal priorities keep their saved sibling order, including insertion above or below. Changing a priority keeps the task selected, and children stay with their parent. Priorities and sibling order are saved in SQLite and preserved by deletion undo.

The fullscreen interface uses your terminal's background and ANSI color palette. Tasks occupy one row each, with bold parents and indented connecting branches. A small cyan caret marks the selection; checkmarks and tree lines use normal text color and turn cyan when selected. Focus and completion preserve title and priority colors; a checkmark identifies completed tasks. A `done/total` count beside a parent shows its direct children. The header contains only the app name and completion count. Short key hints stay at the bottom; input and errors appear when needed. Layout and key hints adapt to the terminal size.

Create and edit tasks directly in the tree. A new draft appears at its sorted position among siblings or directly beneath a parent when creating its first child. `Ctrl-p` changes the draft priority and its position before saving. Starting a new task clears search to show its context. `Enter` saves the inline row; `Esc` cancels it. Search stays at the bottom.

## Check

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

CI runs these checks on Linux, macOS, and Windows. For a local test session, use `cargo run --locked -- --db ./target/dev/db.sql` to keep development tasks separate from your personal database.

Keyboard input and Unicode editing live in `src/vim_motion`, following the action-based structure of `argvcode`. See [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) for development guidance, and the [maintainer guide](docs/maintaining.md) for repository setup and releases.

## Community

Use [GitHub issues](https://github.com/argv-tech/argv-todo/issues) for reproducible bugs, focused feature proposals, and questions. Follow the [Code of Conduct](CODE_OF_CONDUCT.md), and report vulnerabilities through the process in [SECURITY.md](SECURITY.md).

## License

argv-todo is licensed under the [MIT license](LICENSE).
