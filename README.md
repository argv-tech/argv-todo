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
| `j` / `k` | Next / previous task |
| `h` / `l` | Switch All / Active / Done filters |
| `gg` / `G` | First / last task |
| `3j`, `2k` | Repeat a motion |
| `i`, `a`, `o` | Add a task |
| `e`, `cc` | Edit selected task |
| `Space`, `x`, `Enter` | Toggle completion in normal mode |
| `dd`, `3dd` | Delete one / three tasks starting at selection |
| `u` | Undo last deletion in the current session |
| `/` | Search titles, ignoring case |
| `?` | Show help |
| `Esc` | Cancel input, close help, or clear applied search |
| `Ctrl-r` | Reload tasks from disk |
| `q`, `Ctrl-c` | Quit |

In insert and search modes, letters are text. `Enter` saves or applies search. Arrow keys, `Home`, and `End` move the input cursor; `Ctrl-Left` / `Ctrl-Right` move by word. `Backspace`, `Delete`, `Ctrl-w`, and `Ctrl-u` edit text. Bracketed paste is supported. Text editing handles Unicode graphemes and wide characters.

The borderless interface fills the terminal with a task list, selected-task metadata, and a command line. The header stays at the top and the mode and key hints stay at the bottom. Layout and key hints adapt to the terminal size.

## Check

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
