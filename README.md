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
| `Space`, `x`, `Enter` | Toggle completion in normal mode |
| `dd`, `3dd` | Delete one / three tasks and all their children starting at selection |
| `u` | Restore the last deleted task tree(s) in the current session |
| `/` | Search all titles, ignoring case; keep ancestors visible for context |
| `?` | Show help |
| `Esc` | Cancel input, close help, or clear applied search |
| `Ctrl-r` | Reload tasks from disk |
| `q`, `Ctrl-c` | Quit |

In insert and search modes, letters are text. `Enter` saves or applies search. Arrow keys, `Home`, and `End` move the input cursor; `Ctrl-Left` / `Ctrl-Right` move by word. `Backspace`, `Delete`, `Ctrl-w`, and `Ctrl-u` edit text. Bracketed paste is supported. Text editing handles Unicode graphemes and wide characters.

Tasks can contain nested child tasks. Existing databases are upgraded automatically, keeping existing tasks at the top level. Deleting a parent also deletes its descendants; `u` restores the entire tree. Completion applies to the selected task independently of its children.

The borderless interface fills the terminal with one expanded tree containing every task and subtask. Bold root tasks use square checkboxes; children use circles, wider indentation, and connecting branches. Horizontal padding keeps the tree readable, with one terminal row per task. A `done/total` count beside a parent shows its direct children. Moving between parents and children keeps the same view. Selected-task metadata and a command line sit beneath the tree. The header stays at the top and the mode and key hints stay at the bottom. Layout and key hints adapt to the terminal size.

## Check

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
