# todo-rs

A terminal todo app built with Rust, Ratatui, Crossterm, and SQLite. Keyboard input and text editing live in `src/vim_motion`, following the action-based structure of `argvcode`.

## Run

```sh
cargo run
```

A Rust toolchain and a C compiler are required to build. SQLite is bundled; no SQLite server or system SQLite installation is required.

## Storage

Changes are saved immediately. The app creates its config directory and database on first launch:

| Platform | Default database |
| --- | --- |
| Linux | `~/.config/todo-rs/db.sql` |
| macOS | `~/Library/Application Support/todo-rs/db.sql` |
| Windows | `%APPDATA%\todo-rs\db.sql` |

Linux honors `XDG_CONFIG_HOME`. Despite its `.sql` extension, `db.sql` is a binary SQLite database. SQLite may create `db.sql-wal` and `db.sql-shm` beside it while running.

Override the location when needed:

```sh
cargo run -- --db /tmp/todos.sql
cargo run -- --help
```

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` | Next / previous row in the task tree |
| `l` | Select the first child; start a child if there are none |
| `h` | Select the parent task |
| `gg` / `G` | First / last task |
| `3j`, `2k` | Repeat a motion |
| `i`, `a`, `o` | Add a sibling of the selected task; add a root task when the list is empty |
| `e`, `cc` | Edit selected task |
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

The fullscreen interface uses your terminal's background and ANSI color palette. Tasks occupy one row each, with bold parents and indented connecting branches. A small caret and cyan title mark the selection; completed tasks have green checks and dimmed titles. A `done/total` count beside a parent shows its direct children. The header contains only the app name and completion count. Short key hints stay at the bottom; input and errors appear when needed. Layout and key hints adapt to the terminal size.

## Check

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
