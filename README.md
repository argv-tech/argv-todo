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
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
