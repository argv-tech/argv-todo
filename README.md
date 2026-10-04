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

Press `Esc` from the task view to open configuration. At 74 columns or wider, the screen splits into two panes: the database-path setting on the left and the active database and configuration-file paths on the right. Smaller terminals stack these sections. Paths wrap in spacious panes and show an ellipsis when space is limited. The screen shows the ARGV-TODO ASCII logo when the terminal is large enough and a compact heading on smaller terminals. Press `Enter`, `e`, or `i` to edit `database_path`, then `Enter` to save it to TOML. Editing uses the same Unicode text controls and paste support as task titles. Saves preserve comments and apply on the next launch; the current database stays open. `--db` continues to take precedence.

While editing configuration, `Esc` discards the draft and returns to the configuration view. Press `Esc` again to return to tasks. In the task view, `Esc` first cancels active input, closes help, or clears an applied search; press it again to open configuration.

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
| `Esc` | Open configuration; close help or clear applied search first. In a field, return to Normal, then press again to cancel |
| `Ctrl-r` | Reload tasks from disk |
| `q`, `Ctrl-c` | Quit |

Every input field—task titles, search, and the configuration database path—supports **Insert**, **Normal**, **Visual**, **Visual Line**, and **Replace** modes. Fields open in Insert, where letters are text. `Esc` returns to Normal without closing the field; another `Esc` cancels it. In Visual or Replace, `Esc` also returns to Normal. When a command is pending, `Esc` cancels the command first. `Enter` saves or applies search from any field mode. The footer shows the mode and pending command; Visual selections use reverse video.

| Field keys | Action |
| --- | --- |
| `i`, `a`, `I`, `A` | Insert before the cursor, after it, before the first nonblank, or at the end |
| `h`, `l`, arrows, `0`, `^`, `$`, `Home`, `End` | Move left/right or to the start, first nonblank, or end |
| `w`, `b`, `e`; `W`, `B`, `E`; `ge`, `gE` | Move by word or space-separated WORD, including word ends |
| `gg`, `G`, `3\|`, `%` | Field start/end, display column, or matching bracket |
| `f`, `F`, `t`, `T` followed by a character | Find or stop just before/after a character, forward/backward |
| `;`, `,` | Repeat the last find, or reverse it |
| `d`, `c`, `y` followed by a motion or text object | Delete, change and enter Insert, or yank |
| `dd`, `cc`, `yy` | Delete, change, or yank the whole field |
| `iw`, `aw`, `iW`, `aW` after an operator or in Visual | Inner/around word or WORD; around includes adjacent whitespace |
| `i`/`a` followed by `(`, `)`, `[`, `]`, `{`, `}`, `<`, `>`, quotes, or backtick | Inside/around matching delimiters; `b` aliases parentheses and `B` braces |
| `v`, `V`, `o` | Character Visual, whole-field Visual, or swap selection ends |
| Visual `d`, `c`, `y`, `r` + character | Delete, change, yank, or replace the selection |
| `x`, `X`, `D`, `C`, `s`, `S` | Delete at/before the cursor, delete/change to the end, substitute characters, or change the whole field |
| `r2`, `3r2`, `R` | Replace one/three graphemes with `2`, or enter Replace mode |
| `p`, `P` | Put the session's yank buffer after/before the cursor, or replace a Visual selection |
| `u`, `Ctrl-r`, `.` | Undo, redo, or repeat the last field change |
| `gu`, `gU`, `g~` + motion/object; Visual `u`, `U`, `~` | Lowercase, uppercase, or swap case |

Motions and operators accept counts: `3dw` deletes three words and `2d3w` deletes six. Counts are capped at 9999. Nested bracket objects accept a count to choose an enclosing pair, such as `d2i{`. These are single-line fields: `dd` clears the current field while task-list `dd` still deletes task trees. Vertical motions stay in the field. The supported commands above cover field editing; Vim's file, window, Ex-command, macro, and block-Visual features are outside this editor.

While typing, arrow keys, `Home`, and `End` move the cursor; `Ctrl-Left` / `Ctrl-Right` move by word. `Backspace`, `Delete`, `Ctrl-w`, and `Ctrl-u` edit text. Replace-mode `Backspace` restores the overwritten grapheme. Insert/Replace sessions undo as one change. `Ctrl-p` cycles task priority in any task-field mode. Bracketed paste inserts text, puts it after the cursor in Normal, or replaces a Visual selection. Newlines and other control characters become spaces. Editing and selection respect Unicode grapheme boundaries and display widths.

Tasks can contain nested child tasks. Existing databases are upgraded automatically, keeping existing tasks at the top level. Deleting a parent also deletes its descendants; `u` restores the entire tree. Completing or reopening a parent gives all its descendants the same state, including tasks hidden by search. Toggling a child affects its own subtree without changing its ancestors or siblings. Changes are saved atomically.

Tasks have **high**, **mid**, or **low** priority; new roots and existing tasks default to **mid**. New children inherit their parent's priority. Siblings created with `o` or `O` start with the selected task's priority so they appear beside it. Roots and children within each parent are sorted high first, then mid, then low. Equal priorities keep their saved sibling order, including insertion above or below. Changing a priority keeps the task selected, and children stay with their parent. Priorities and sibling order are saved in SQLite and preserved by deletion undo.

The fullscreen interface uses your terminal's background and ANSI color palette. Tasks occupy one row each, with bold parents and indented connecting branches. A small cyan caret and an underlined title mark the selection; checkmarks and tree lines use normal text color and turn cyan when selected. Focus and completion preserve title and priority colors; a checkmark and a struck-through title identify completed tasks. A `done/total` count beside a parent shows its direct children. Long titles show an ellipsis so the count stays visible, without splitting Unicode graphemes. The header contains only the app name and completion count. Short key hints stay at the bottom and fit the available width; input and errors appear when needed. Help keeps its scroll and close instructions visible. Layout and key hints adapt to the terminal size.

Create and edit tasks directly in the tree. A new draft appears at its sorted position among siblings or directly beneath a parent when creating its first child. `Ctrl-p` changes the draft priority and its position before saving. Starting a new task clears search to show its context. `Enter` saves the inline row; `Esc` enters Normal and another `Esc` cancels it. Search stays at the bottom and remains live in every field mode.

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
