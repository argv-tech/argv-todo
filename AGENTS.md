# Project guidance

Build a minimal Rust todo TUI with Ratatui and SQLite. Prefer small, direct
changes and few dependencies. Keep existing features unless the user asks to
change them; avoid adding features beyond the request.

## Before making changes

- Read the affected code, its tests, and `Cargo.toml` and `Cargo.lock` before
  choosing an implementation.
- Use the repository's resolved dependency versions and enabled features. Check
  backend compatibility when changing terminal code.
- Use `.agents/skills/ratatui/SKILL.md` for Rust TUI work and its references for
  the affected area. The other Ratatui guides in `.agents/skills/` provide
  supporting material; apply only advice compatible with this project.
- The knowledge patch includes APIs from newer Ratatui releases. Verify examples
  against the locked version and local crate source before using them.

## Readable Rust

- Write code a person can follow without reconstructing hidden control flow. Use
  descriptive names, short focused functions, and straightforward branches.
- Keep each module responsible for one coherent concern. Separate input mapping,
  application updates, rendering, persistence, and configuration.
- Prefer explicit types and enums for modes and actions. Avoid boolean flags
  whose meaning depends on combinations of other flags.
- Borrow data when practical. Clone when ownership requires it, rather than
  using clones to hide an unclear design.
- Return errors with useful context. Avoid `unwrap()` and `expect()` on user
  input, database operations, and terminal I/O.
- Add comments for intent, invariants, or surprising behavior. Do not narrate
  obvious code.
- Reuse existing helpers. Add abstractions, dependencies, caches, or
  asynchronous work only when the requested behavior needs them.

## Files and modules

- Preserve the directory modules in `src/app/`, `src/ui/`, `src/db/`,
  `src/config/`, and `src/vim_motion/`. Extend them with focused files; keep
  their responsibilities separate as the app grows.
- Add more modules as future work introduces distinct responsibilities. The
  current source layout is a starting point and should evolve with the app.
  Create new top-level modules when a concern does not fit an existing owner;
  otherwise add focused submodules under that owner.
- Keep `src/main.rs` focused on startup and terminal cleanup. Application event
  handling belongs in `src/app/events.rs`.
- Split substantial responsibilities into multiple files and subdirectories
  rather than growing one large file.
- A small leaf module can remain `src/name.rs`. When it needs submodules, use
  `src/name/mod.rs` with focused sibling files and further subdirectories as
  needed. Do not keep both forms for the same module.
- Keep `mod.rs` focused on module declarations, intentional re-exports, and the
  shared types or coordination belonging to that module. Put substantial
  implementations in named sibling files.
- Declare implementation details with private `mod name;`. Use `pub(super)` for
  parent access or `pub(crate)` for access elsewhere in this crate.
- Use `pub mod name;` only when the module is intentionally part of a public
  API. Public modules do not automatically make their contents public.
- Keep functions, fields, and types private by default; expose only what callers
  need. Prefer a small re-exported interface when callers should not depend on
  internal file paths.
- Split code along responsibility boundaries, not arbitrary line counts. Avoid
  one-file-per-function fragmentation and unrelated restructuring.
- Add a nested directory with its own `mod.rs` when a concern needs several
  related files. Name files and directories after their responsibility.
- Review module size and cohesion during each change. Extract a responsibility
  when new code would make a file harder to navigate or mix unrelated behavior.
  Create modules for implemented needs rather than empty future placeholders.
- Keep cross-module imports on the owning module's interface. Use explicit
  imports and intentional re-exports; avoid wildcard imports in production code
  and exposing internal modules merely to make imports compile.
- Coordinate application behavior in `app`. Keep terminal rendering out of
  persistence and configuration modules, and keep SQL out of UI and keyboard
  handling modules.

Current source layout:

| Module            | Responsibility                                                  |
| ----------------- | --------------------------------------------------------------- |
| `src/main.rs`     | Startup, terminal setup and cleanup                             |
| `src/app/`        | State, actions, editing, events, navigation, settings, tasks, pane focus and projections |
| `src/ui/`         | Frame composition, geometry, text, editors, chrome, help, theme |
| `src/ui/tasks/`   | Pane composition, draft placement, and task-tree rendering       |
| `src/ui/help/`    | Shortcut content, wrapping, and help rendering                  |
| `src/ui/config/`  | Configuration panes, setting editor, and path details           |
| `src/db/`         | Database entry point, task model, queries, migrations, trees    |
| `src/cli.rs`      | Command-line options and path overrides                         |
| `src/config/`     | Configuration paths, loading, validation, and atomic saving     |
| `src/vim_motion/` | Keyboard mapping, modes, actions, and Unicode text editing      |
| `src/vim_motion/manager/` | Task, field, and help key mapping, counts, pending command parsing |
| `src/vim_motion/editor/` | Unicode cursor and selection, edits, motions, objects, history, viewport |
| `tests/unit/`     | Unit suites and test helpers, grouped by owning module          |

Directory modules keep their shared state and interface in `mod.rs`, with
focused private implementation modules alongside it. All test suites and test
helpers live under `tests/unit/`, grouped by owning module. Source modules load
their suites with `#[cfg(test)]` and `#[path = "..."] mod tests;` so tests
retain access to private code without expanding the production API.

Route keyboard input through `vim_motion`. Callers use its re-exported editor,
manager, actions, and modes rather than internal module paths. Follow the
visibility rules above when adding or changing module boundaries.

## Workflow for modular changes

1. Identify the owning module and read its `mod.rs`, relevant implementation
   files, and tests before editing.
2. Extend an existing file when the change fits its focused responsibility. Give
   a distinct responsibility its own module from the start; extract related code
   as needed, and add a subdirectory with `mod.rs` when the new concern needs
   several files.
3. Register new files in the parent's `mod.rs`. Keep helpers private, use the
   narrowest visibility needed, and re-export only the interface callers use.
4. Put tests and test helpers under `tests/unit/`, grouped by owning module.
   Register each suite in its source module with `#[cfg(test)]` and a relative
   `#[path = "..."]` attribute. Preserve existing regression coverage when
   moving code; avoid inline tests or test suites under `src/`.
5. Review module boundaries and imports as well as behavior. Update this
   source-layout map and `CONTRIBUTING.md` when responsibilities change, then
   run the required validation below.

## Interface

- Use the full terminal with small edge margins, readable text, and restrained
  colors.
- Keep the normal view as a full task tree with indentation. Optional Split
  shows todo/completed panes, with unfinished ancestors as nonselectable ghost
  context for completed children. Tab switches pane focus; settings apply layout
  changes immediately.
  Avoid separate task pages, dashboards, and large panels.
- `j/k` selects tasks. `h` selects the parent. `l` selects the first child and
  stays selected if there are none. `i/a` creates a child, or a root when the
  list is empty.
- Keep input hints short and adapt the layout to narrow terminals.
- Keep help text and `README.md` consistent with the actual keybindings.

## State, input, and rendering

- Keep one clear owner of mutable application state and terminal input. Map
  backend events to typed actions, update state, then render.
- Store selection, scrolling, editing, and focus state outside temporary
  widgets. Rendering may update widget viewport state, but must not change task
  data or save changes.
- Compose the full visible interface in one draw call. Keep rendering free of
  database queries, filesystem or network I/O, sleeps, and blocking work.
- Derive geometry from the supplied frame or rectangle. Handle nonzero origins,
  empty areas, narrow terminals, clipping, and resize without panics or
  coordinate underflow.
- Route input to active dialogs and editors before task commands. Handle key
  press, repeat, and release deliberately; treat bracketed paste as a separate
  input event.
- Use grapheme boundaries for text editing and display width for layout and
  cursor positions. Do not equate bytes or character counts with terminal
  columns.
- Draw overlays in order and clear their area when needed. Make focus and
  completion understandable without relying only on color.
- Avoid busy loops and unnecessary redraws. Introduce workers only for work that
  would block interaction; workers return results to the main loop and never
  draw or mutate UI state directly.

## Terminal reliability

- Preserve the existing lifecycle unless the request requires changing it. Use
  version-compatible Ratatui helpers where they cover the required modes.
- Balance every enabled terminal mode with cleanup, including raw mode,
  alternate screen, bracketed paste, and cursor visibility.
- Restore the terminal on normal exit, errors, panic, and partial
  initialization. Attempt remaining cleanup steps even if one fails, and
  preserve the original error.
- Use a consistent terminal writer. Do not print ordinary logs into the active
  TUI.
- Treat backend write failures as a potentially broken terminal session; restore
  and exit rather than retrying indefinitely.

## Storage

- Use `dirs::config_dir()/argv-todo/db.sql`: Linux `~/.config`, macOS
  `~/Library/Application Support`, and Windows roaming AppData. Honor `--db`
  overrides.
- Save changes immediately with parameterized SQL.
- Migrate existing databases without losing tasks, parent links, completion
  state, priorities, or sibling order.
- Delete, restore, and toggle task trees atomically, including descendants
  hidden by search. Keep the terminal usable after errors.
- Use temporary or in-memory databases for tests; never modify the user's
  database during verification.
- Keep configuration validation and I/O outside rendering. Honor `--db`
  precedence and preserve existing configuration comments when saving.

## Documentation and scope

- Keep `README.md`, help text, and development guidance consistent with behavior
  and keybindings.
- Add a concise `CHANGELOG.md` entry for user-visible changes.
- Preserve uncommitted user changes. Avoid unrelated refactors and commit only
  when requested.

## Validation

Test changed behavior at the smallest useful layer: action/state tests,
temporary database tests, or deterministic Ratatui buffer and `TestBackend`
tests. Cover relevant error paths, tiny layouts, resize, Unicode, paste,
selection, and cursor behavior. Assert cell styles when color or modifiers
matter; text snapshots alone do not verify them. Use real-terminal tests only
for behavior that buffer tests cannot cover.

For code changes, run relevant tests and all of these checks:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

Avoid redundant tests and dependencies added only for trivial coverage.
Documentation-only changes need a diff and consistency review rather than new
tests. Report any checks that could not run and any terminal behavior that was
not exercised.

## Development servers

- Never start, restart, stop, or kill a development server unless the user
  explicitly requests it.
- Assume the user manages development servers. Use an existing server for
  verification; if unavailable, report the limitation and continue with other
  checks.
- Do not request permission to launch a development server as a routine testing
  step.
