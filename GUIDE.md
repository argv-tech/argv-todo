# argv-todo guide

[README: installation and project links](README.md)

- [Your first task tree](#your-first-task-tree)
- [Everyday keys](#everyday-keys)
- [Layouts and projects](#layouts-and-projects)
- [Deadlines and reminders](#deadlines-and-reminders)
- [Deletion, undo, and the archive](#deletion-undo-and-the-archive)
- [Settings and data files](#settings-and-data-files)

## Your first task tree

1. Press `i`, type `Release checklist`, and press `Enter` to save.
2. With that task selected, press `i` again. Type `Run tests` and press
   `Enter`. This creates a child.
3. Press `o`, type `Update docs`, and press `Enter` to add another child
   beside it.
4. Press `Space` to complete `Update docs`. Press `h` to select the parent;
   `Enter` folds or unfolds its children.
5. With the parent selected, press `o` to add another top-level task.

**`i` adds a child; `o` adds a sibling.** In an empty list, either creates
a top-level task. Completed tasks stay visible by default.

Edits open in Insert mode, so you can type normally. `Enter` saves a title.
To discard an edit, press `Esc` to enter Normal mode, then `Esc` again to
cancel. Notes use `Ctrl-s` to save because `Enter` adds a line while typing.

## Everyday keys

These keys apply while browsing tasks. Uppercase letters mean Shift plus
the letter; `dd`, `cc`, and `gg` are two consecutive key presses.

| Key | Action |
| --- | --- |
| `j` / `k`, Up / Down | Select the next / previous visible task |
| `h` / `l` | Select the parent / expand and select the first child |
| Left / Right | Navigate parent / child without switching project tabs |
| `Enter` | Fold or unfold the selected task's children |
| `gg` / `G`, Home / End | Select the first / last task |
| `i` / `a` | Add a child, or a root in an empty list |
| `o` / `O` | Add a sibling below / above |
| `e` / `cc` | Edit the title |
| `n` | Edit the task's note |
| `Space` / `x` | Complete or reopen the task **and all its descendants** |
| `t` / `Ctrl-p` | Cycle priority: low → mid → high → low |
| `ph` / `pm` / `pl` | Set high / mid / low priority |
| `T` | Set or clear a deadline |
| `/` | Search task titles, ignoring case |
| `dd` | Delete the selected task and its descendants after confirmation |
| `u` | Undo the last deletion in this session |
| `Alt+Enter` | Clear completed tasks after confirmation |
| `A` | Open the archive |
| `Ctrl-r` | Reload tasks from disk |
| `Esc` | Clear an applied search; otherwise open Settings |
| `?` | Open or close help |
| `q` / `Ctrl-c` | Quit from the task tree / quit from anywhere |

Counts repeat commands: `3j` moves down three rows; `3dd` selects three
visible tasks for deletion, including their descendants.

Search keeps ancestors visible so matches have context. Completing or
deleting a parent also affects descendants hidden by search or folding.
Completing a child leaves its ancestors and siblings unchanged.

### Priority, deadline, and manual ordering

By default, siblings are sorted high → mid → low by each task's own priority
at every tree level. A child's priority does not change its parent's position;
descendants stay attached to their parent. Equal-priority siblings keep their
saved order. New roots use the `default_priority` setting;
children inherit their parent's priority and new siblings inherit the
selected task's priority. `Ctrl-p` also changes priority while editing a title.

Choose `sort_order = "deadline"` in Settings to sort branches by their earliest
deadline, including descendants. Overdue deadlines come first; undated branches
come last. Equal deadlines keep saved order. Deadline sorting ignores priority,
and priority sorting ignores deadlines.

Use `Shift-H` / `Shift-L` to outdent / indent the selected task tree in any
sort mode. Indenting uses the previous sibling in the current sort order.
The selected sort mode stays active and descendants move with their parent.
To move trees up or down yourself, set `sort_order` to `manual` in Settings:

| Key | Action |
| --- | --- |
| `J` / `K` | Move the selected task tree down / up (manual order only) |
| `L` | Indent under the previous sibling |
| `H` | Outdent one level |

### Notes and text editing

Press `n` to edit the selected task's note in the bottom section. `Enter`
adds a line; `Ctrl-s` saves from any mode. In Normal mode, `Enter` also saves.
Clear the note and save to remove it.

In Normal mode, press `i` to resume typing. Fields support Vim motions and
editing commands: `w` / `b`, `0` / `$`,
`dw`, `ciw`, `dd`, `v` / `V`, `y` / `p`, `r` / `R`, `u` / `Ctrl-r`, and `.`.
In a title field, `dd` clears text. In a note, it deletes the current line.
The footer shows the current mode.

Arrow keys, Home, End, Backspace, and Delete work while typing. Unicode
editing respects grapheme boundaries. On Linux/macOS, pasted line breaks
are preserved in notes and become spaces in single-line fields. On Windows,
paste one line at a time: a pasted newline acts as `Enter`.

## Layouts and projects

Press `Esc` from the task tree to open Settings, select `task_view`, and
use `Enter` or `h` / `l` to change it. Press `Esc` to return. Layout changes
apply immediately.

| Layout | What you see | `Tab` / `Shift-Tab` |
| --- | --- | --- |
| `normal` | One task tree containing all projects | No pane switch |
| `split` | Todo and Completed panes; stacked in narrow terminals | Switch panes |
| `projects` | One task tree per project, plus Inbox for unassigned tasks | Next / previous project |

In Split, dimmed `(parent)` rows give completed children context. Those rows
cannot be selected. In Projects, `h` switches to the previous tab when there
is no parent; `l` switches to the next tab when there is no child. Each pane
or tab remembers its selection and scroll position.

| Key | Project action |
| --- | --- |
| `P` | Create a project and switch to its tab |
| `R` | Rename the active project in Projects |
| `D` | Delete the active project and all its tasks in Projects; confirm with `Enter` |
| `m` | Move the selected task tree to a project; choose with `j` / `k`, confirm with `Enter` |
| `C` | Replace a top-level task with a project named after its title |

Project names must be nonempty and unique ignoring ASCII case. Inbox is
reserved and cannot be renamed or deleted. `Esc` cancels a project edit or
destination picker; from Insert mode, press it twice to cancel an edit.

Moving a child to another project makes it a root there and keeps its
descendants. Conversion with `C` removes the original task **and its note**;
its children become project roots with their descendants and task data intact.
Without children, conversion creates an empty project.

## Deadlines and reminders

Press `T` on a task to open the deadline picker. **Deadline times use UTC.**

| Picker key | Action |
| --- | --- |
| `h` / `l` | Previous / next day |
| `j` / `k` | Next / previous week |
| `H` / `L` | Previous / next month |
| `t` | Jump to today |
| `Tab` | Switch between date, hour, and minute |
| `h` / `l` in a time field | Decrease / increase the hour or minute |
| `Enter` / `x` / `Esc` | Save / clear the deadline / cancel |

For desktop reminders, run the separately installed daemon in another terminal:

```sh
argv-todo-daemon
```

It stays running until stopped; the TUI does not start it. If you use `--db`,
pass the same path to both binaries so they share the database and config.
On Linux/macOS, use `argv-todo-daemon &` to run it in the background.

The default reminders fire one day and one hour before a deadline, then at
the deadline. Set `notifications = false` to disable them, or set an
individual reminder's minutes to `0` to disable that reminder. Reminder
offsets accept 0–10080 minutes. Each notification fires at most once per
deadline per daemon run; restarting can repeat notifications already due.

## Deletion, undo, and the archive

Deletion prompts use `Enter` to confirm and `Esc` to cancel. Other task
commands are ignored until you choose.

| Operation | Can it be recovered? |
| --- | --- |
| `dd`: delete a task tree | `u` during the same session; it does **not** go to the archive |
| `Alt+Enter` with `completed_delete = "soft"` (default) | `u` now, or restore from the archive after restart |
| `Alt+Enter` with `completed_delete = "hard"` | `u` during the same session only |
| `D` in Projects: delete the project, including archived tasks | Cannot be undone |
| `D` in the archive: permanently clear all archived tasks | Cannot be undone |

Completed cleanup includes tasks hidden by search or folding. It affects
the active project in Projects, or all projects in Normal and Split.
Unfinished descendants survive and move to the nearest surviving ancestor,
or become roots.

Press `A` to browse soft-deleted tasks from **all projects**. Select with
`j` / `k`; press `r` to restore a task, its archived descendants, and any
archived parents. Restored tasks keep notes, priorities, deadlines, project
assignment, and completion state. Completed tasks may still be hidden by
your visibility setting. Press `A` or `Esc` to return to your previous view.

## Settings and data files

The first launch creates `config.toml` and `db.sql` in the platform config
directory:

| Platform | Directory |
| --- | --- |
| Linux | `~/.config/argv-todo/` (honors `XDG_CONFIG_HOME`) |
| macOS | `~/Library/Application Support/argv-todo/` |
| Windows | `%APPDATA%\argv-todo\` |

In Settings, select with `j` / `k` or `Tab` / `Shift-Tab`. Use `Enter` or
`h` / `l` to change a preference. For `database_path`, press `Enter` to edit,
then `Enter` to save. Database changes take effect on the next launch;
other preferences apply immediately. Changes preserve config comments.

You can also edit `config.toml` directly. These are the defaults:

```toml
database_path = "db.sql"
task_view = "normal"              # normal, split, projects
default_priority = "mid"          # low, mid, high
show_completed = true            # Normal and Projects; Split has its own pane
sort_order = "priority"           # priority, manual, deadline
completed_delete = "soft"         # soft, hard
show_hints = true
notifications = true
notify_reminder_1_minutes = 60
notify_reminder_2_minutes = 1440
```

The app rereads settings on launch and when opening Settings. Invalid TOML,
unknown keys, or invalid values produce an error.

To use another database:

```sh
argv-todo --db /path/to/tasks.sql
argv-todo --help
argv-todo --version
```

`--db` takes precedence over `database_path` and uses `config.toml` beside
the specified database. Otherwise, relative `database_path` values resolve
against the config directory. Changing the path selects a database; it
does not move tasks or the config file. On Windows, use TOML literal strings
such as `database_path = 'C:\Users\example\tasks.sql'` or forward slashes.

Despite the extension, `db.sql` is a binary SQLite database. **Close the TUI
and daemon before copying or moving it.** Keep any remaining `db.sql-wal`
and `db.sql-shm` files with it. Back up `config.toml` too if you want to
preserve settings. Task data is not encrypted by the app.
