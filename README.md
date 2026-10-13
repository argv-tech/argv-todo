# argv-todo

A small terminal todo app with Vim-style keys, nested tasks, and local SQLite storage. Built with Rust, Ratatui, and Crossterm.

- Keep parents and children together in one task tree.
- Choose normal, split todo/completed, or project tab layouts in Settings.
- Give each project its own tab and move whole task trees between projects.
- Create and edit tasks inline, with three priority levels and saved sibling order.
- Keep a saved note for each task, shown in the bottom section when selected.
- Search titles, complete subtrees, and undo deletions.
- Use Unicode text, bracketed paste, and your terminal's color palette.

[Contributing](CONTRIBUTING.md) · [Support](SUPPORT.md) · [Changelog](CHANGELOG.md) · [MIT license](LICENSE)

## Install

After the first crates.io release, install with:

```sh
cargo install argv-todo --locked
argv-todo
```

To install before publication, or build a local checkout, use the source instructions below.

### From source

A current stable Rust toolchain and a C compiler are required to build. SQLite is bundled; no SQLite server or system SQLite installation is required.

On macOS, install the Xcode Command Line Tools with `xcode-select --install`. On Windows, use the Rust MSVC toolchain and Visual Studio Build Tools with the **Desktop development with C++** workload.

```sh
git clone https://github.com/argv-tech/argv-todo.git
cd argv-todo
cargo install --path . --locked
argv-todo
```

Ensure Cargo's binary directory is on your `PATH` (`~/.cargo/bin` on Linux and macOS, `%USERPROFILE%\\.cargo\\bin` on Windows).

## Run

```sh
cargo run --locked
```

An interactive terminal is required. The app uses the full terminal and shows task rows at 35 columns × 12 rows or larger. Press `?` for help and `q` to quit.

Run in Terminal or iTerm2 on macOS, or Windows Terminal with PowerShell or Command Prompt on Windows. Linux and macOS use bracketed paste; Windows uses native console text input because Crossterm 0.29 does not parse bracketed-paste events there. Paste a single line into Windows input fields; a pasted newline acts as Enter.

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
task_view = "normal"
```

Set `database_path` to an absolute path or a path relative to the config's directory. `--db` takes precedence over this setting. Changing the setting selects a database; it does not move existing tasks. The config stays in its original directory even when the database path points elsewhere.

For Windows paths in TOML, use literal strings such as `database_path = 'C:\Users\example\task storage\db.sql'`, or forward slashes such as `database_path = "C:/Users/example/task storage/db.sql"`.

The app reads settings on startup and when opening configuration, preserving existing config files and comments. An empty config uses `db.sql` and the normal task view. Invalid TOML, unknown settings, invalid layout names, and invalid database paths produce an error before the terminal interface opens. `--help` and `--version` do not create files.

Press `Esc` from the task view to open configuration. Settings are grouped under **Storage** (`database_path`), **Display** (`task_view`, `show_completed`, `show_hints`), and **Task behavior** (`default_priority`, `sort_order`, `completed_delete`). Use `j` / `k`, arrows, or `Tab` / `Shift-Tab` to select settings across categories, skipping their headings. `gg` / `G` or `Home` / `End` selects the first / last setting. Short panes scroll to the selected setting and keep its category visible when space permits. At 74 columns or wider, the screen shows settings on the left and details for the selected setting on the right. Selecting `database_path` shows database and configuration-file paths; selecting `task_view` shows the active layout, its controls, and a small layout preview when space permits. Details update as you select or change a setting. Smaller terminals stack these sections and keep the details concise. Paths wrap in spacious panes and show an ellipsis when space is limited. The screen shows the ARGV-TODO ASCII logo when space permits. On `database_path`, press `Enter`, `e`, or `i` to edit, then `Enter` to save. Editing uses the same Unicode controls and paste support as task titles. Database-path changes apply on the next launch; the current database stays open, and `--db` retains precedence.

On `task_view`, press `Enter`, `e`, `i`, `h` / `l`, or Left / Right to cycle through `normal`, `split`, and `projects`. The choice saves to TOML and applies immediately. Press `Esc` to return to tasks without restarting. Normal keeps the full task tree. Split shows unfinished todos on the left and completed tasks on the right, stacking them below 70 columns. Unfinished ancestors of completed children appear in the Completed pane as dimmed ghost rows labeled `(parent)`. These rows provide context and are skipped by selection and task commands. An obsolete `nested` setting falls back to Normal.

In Split, `Tab` or `Shift-Tab` switches pane focus. Each pane keeps its own selection and scroll position; task commands operate on the focused pane. Completing or reopening a task moves it to the corresponding pane and keeps it selected. Search filters both panes. Tab leaves focus in an active editor while typing.

In Projects, each project has its own tab and task tree. `l` expands and selects the first child when one exists; otherwise it selects the next project tab. `h` selects the parent when one exists; otherwise it selects the previous project tab. Counts such as `2l` repeat this navigation. `Tab` / `Shift-Tab` always selects the next / previous project, wrapping at either end. Left / Right arrows navigate the parent / first child within the active tree. Each tab keeps its selection and scroll position. The header count, search, and bottom Note section follow the active project. Existing unassigned tasks appear in **Inbox**. Normal and Split continue to show tasks from all projects.

Press `P` from the task view to create a project; this opens Projects and saves the layout choice. Type its name and press `Enter` to create and select its empty tab, then `i` to add its first root task. Names must be nonempty and unique ignoring ASCII case; Inbox is reserved. Children and new siblings inherit their project. Press `m` on a task to choose a destination with `j` / `k` or arrows, then `Enter` to move the task and all descendants; `Esc` cancels. A child moved to another project becomes a root there. Notes, completion, priorities, and descendant links are preserved. Projects and task assignments are saved immediately in SQLite.

Press `C` on a top-level task to create a project named after its title and open that tab. The original task is removed, including its note. Its immediate children become project roots, preserving their descendants, notes, completion state, priorities, and sibling order. A task without children becomes an empty project. Conversion saves atomically; a duplicate or reserved project name leaves the task tree intact. Child tasks cannot be converted directly.

`Alt+Enter` (also called `Alt+Return`) clears completed tasks, including those hidden by search or folding. In Projects it affects the active project; in Normal and Split it affects all projects. Unfinished descendants are retained under their nearest surviving ancestor, or promoted to roots when necessary. In Settings, `completed_delete` selects **soft** (the default, hides tasks and retains their SQLite records) or **hard** (removes records). The choice saves and applies immediately. Add `completed_delete = "hard"` or `completed_delete = "soft"` to TOML to configure it directly. Press `u` to undo either mode during the same session; undo restores task data and original parent links. Soft-deleted records stay out of the active tree after restart and can be restored from the archive.

Press `A` to browse all soft-deleted tasks, including tasks from other projects and tasks hidden by your current search. The archive labels each task's project and shows its saved note at the bottom. Use `j` / `k` or arrows to select a task, `gg` / `G` or `Home` / `End` for the first / last task, and `Ctrl-r` to reload. Press `r` to restore the selected task, its archived descendants, and any archived parents. Restoration works after restart and preserves notes, completion state, priorities, sibling order, and project assignment; completed tasks remain subject to the current visibility setting. Press `D` (Shift-d) to clear all soft-deleted tasks from all projects permanently, then `Enter` to confirm or `Esc` to cancel. Clearing preserves live descendants and cannot be undone. Other task editing and deletion shortcuts are ignored in the archive. Press `A` or `Esc` to return to your previous project, pane, search, and folds, keeping the selected live task when possible. Hard-deleted tasks do not appear here.

A database preview labeled “after restart” appears below the active database when the draft or saved setting resolves to a different path. Relative paths resolve against the config directory. The preview updates while editing; `--db` still takes precedence on launch.

While editing configuration, `Esc` discards the draft and returns to the configuration view. Press `Esc` again to return to tasks. In the task view, `Esc` first cancels active input, closes help, or clears an applied search; press it again to open configuration.

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` | Next / previous visible row in the task tree (also Up / Down) |
| `Tab`, `Shift-Tab` | Switch project tabs or Split panes; select a setting in configuration |
| `l` | Expand and select the first child; in Projects, next tab if there is no child |
| `h` | Select the parent task; in Projects, previous tab if there is no parent |
| Right / Left arrows | Expand and select the first child / select the parent; stay if there is no child |
| `gg` / `G` | First / last task |
| `3j`, `2k` | Repeat a motion |
| `i`, `a` | Add a child of the selected task; add a root task when the list is empty |
| `o`, `O` | Add a sibling below or above the selected task; add a root task when the list is empty |
| `P` | Create a project and open its tab |
| `C` | Convert a top-level task to a project, keeping its whole tree |
| `m` | Move the selected task tree to a project |
| `Alt+Enter` / `Alt+Return` | Clear completed tasks using the configured deletion mode |
| `A` | Show all soft-deleted tasks; `A` or `Esc` returns |
| `r` in archive | Restore the selected task tree and its archived parents |
| `D` in archive | Permanently clear all soft-deleted tasks; `Enter` confirms, `Esc` cancels |
| `e`, `cc` | Edit selected task |
| `n` | Add or edit the selected task's note in the bottom section |
| `t` | Cycle priority: low → mid → high → low |
| `ph`, `pm`, `pl` | Set priority to high, mid, or low |
| `Ctrl-p` | Cycle priority, including while creating or editing inline |
| `Enter` | Collapse or expand the selected task's children |
| `Space`, `x` | Toggle the selected task and all its descendants in normal mode |
| `dd`, `3dd` | Delete one / three tasks and all their children starting at selection |
| `u` | Restore the last deleted task tree(s) in the current session |
| `/` | Search titles in the current view, ignoring case; keep ancestors visible |
| `?` | Open help; press again to close |
| `Esc` | Open configuration; close help or clear applied search first. In a field, return to Normal, then press again to cancel |
| `Ctrl-r` | Reload tasks from disk |
| `q`, `Ctrl-c` | Quit |

Help has sections for task navigation, task changes and project pickers, archive restoration and permanent cleanup, settings, and search and help controls. Archive and Settings list their navigation, confirmation, and cancellation shortcuts separately. It uses the same full-screen branding and spacing as settings. On spacious terminals, choose a section in the guide with `j` / `k` or arrows; its shortcuts appear on the right. `Tab` / `Shift-Tab` cycles sections, and `gg` / `G` or `Home` / `End` selects the first / last section. Selection stays in the guide. Use `PgUp` / `PgDn` to scroll the selected section.

Section titles are bold, uppercase, and spaced apart. Narrow or short terminals show all shortcuts in one scrolling column: use `j` / `k` or arrows to scroll, `PgUp` / `PgDn` to move a page, and `gg` / `G` or `Home` / `End` to jump to the beginning or end. Keys and descriptions align in wider panes and stack in narrow ones. A scrollbar and row range show your position. `Esc` or `?` closes help; the close hint stays visible.

Every input field—task titles, notes, search, and the configuration database path—supports **Insert**, **Normal**, **Visual**, **Visual Line**, and **Replace** modes. Fields open in Insert, where letters are text. `Esc` returns to Normal without closing the field; another `Esc` cancels it. In Visual or Replace, `Esc` also returns to Normal. When a command is pending, `Esc` cancels the command first. `Enter` saves or applies search from any field mode, except notes: it adds a line in Insert or Replace and saves in Normal or selection modes. `Ctrl-s` saves notes from any mode. The footer shows the mode and pending command; Visual selections use reverse video.

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

Tasks can contain nested child tasks. `j` / `k` moves through visible rows at every level; use `h` / `l` to select the parent or first child. `Enter` hides or reveals descendants without changing task data; a `+` beside the child count marks a collapsed branch. Collapse state lasts for the current session and is shared across panes. Starting a search expands branches so hidden tasks can appear in the results. Existing databases are upgraded automatically, preserving hierarchy and keeping unassigned tasks in Inbox. Deleting a parent also deletes its descendants; `u` restores the entire tree. Completing or reopening a parent gives all its descendants the same state, including tasks hidden by search. Toggling a child affects its own subtree without changing its ancestors or siblings. Changes are saved atomically.

Tasks have **high**, **mid**, or **low** priority; new roots and existing tasks default to **mid**. New children inherit their parent's priority. Siblings created with `o` or `O` start with the selected task's priority so they appear beside it. Roots and children within each parent are sorted high first, then mid, then low. Equal priorities keep their saved sibling order, including insertion above or below. Changing a priority keeps the task selected, and children stay with their parent. Priorities and sibling order are saved in SQLite and preserved by deletion undo.

The fullscreen interface uses your terminal's background and ANSI color palette. Tasks occupy one row each, with bold parents and indented connecting branches. A small cyan caret and an underlined title mark the selection; checkmarks and tree lines use normal text color and turn cyan when selected. Focus and completion preserve title and priority colors; a checkmark and a struck-through title identify completed tasks. A `done/total` count beside a parent shows its direct children. Long titles show an ellipsis so the count stays visible, without splitting Unicode graphemes. The header shows the app name, version in red, and completion count. Split pane headings show the number of visible selectable tasks, excluding ghost parents. Short key hints stay at the bottom and fit the available width, with keys in cyan and descriptions dimmed; input and errors appear when needed. Help keeps its scroll and close instructions visible. Layout and key hints adapt to the terminal size.

Create and edit tasks directly in the tree. A new draft appears at its sorted position among siblings or directly beneath a parent when creating its first child. `Ctrl-p` changes the draft priority and its position before saving. Starting a new task clears search to show its context. `Enter` saves the inline row; `Esc` enters Normal and another `Esc` cancels it. Search stays at the bottom and remains live in every field mode.

The bottom Note section names the selected task and shows a wrapped preview of its note, following the focused pane in Split. Notes preserve line breaks and blank lines. Short notes leave more room for tasks; longer previews grow to four text rows and end with an ellipsis when clipped. On short stacked layouts, the preview yields space to task panes, search, and errors. Press `n` to add or edit a note. While typing, `Enter` adds a line and Up / Down moves between lines; `Ctrl-s` saves from any mode. The active note editor grows to six text rows when space permits, scrolls vertically to keep the cursor visible, and scrolls long lines horizontally. It has a cyan separator, a focus marker, and a placeholder when empty. Unicode paste preserves line breaks, normalizing CRLF and CR to LF. `Esc` enters Normal, where `j` / `k` moves between lines, `0` / `$` and Home / End select the current line's ends, `gg` / `G` selects the note's ends, and `Enter` saves. Line operations and line selections work on note lines. Another `Esc` discards the draft. Clear the field and save to remove a note. Turning off hints also hides the note heading's shortcuts. Notes are stored in SQLite and preserved when moving tasks, browsing archives, or undoing deletion. Existing tasks start with empty notes.

## Check

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

CI runs these checks on Linux, macOS, and Windows for pull requests targeting `main` or `dev`. Pushing a `v*` tag runs the same checks and builds release binaries before publishing platform archives named with their Cargo version, such as `argv-todo-0.1.0-linux.tar.gz`, to a GitHub release. The test and build workflows also support `workflow_call` for reuse and manual runs. The manual **Publish to crates.io** workflow validates the package by default and uploads it when dry-run is disabled; see the [maintainer guide](docs/maintaining.md#publishing-to-cratesio) for setup and local publishing commands.

For a local test session, use `cargo run --locked -- --db ./target/dev/db.sql` to keep development tasks separate from your personal database.

Keyboard input and Unicode editing live in `src/vim_motion`, following the action-based structure of `argvcode`. See [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) for development guidance, and the [maintainer guide](docs/maintaining.md) for repository setup and releases.

## Community

Use [GitHub issues](https://github.com/argv-tech/argv-todo/issues) for reproducible bugs, focused feature proposals, and questions. Follow the [Code of Conduct](CODE_OF_CONDUCT.md), and report vulnerabilities through the process in [SECURITY.md](SECURITY.md).

## License

argv-todo is licensed under the [MIT license](LICENSE).
