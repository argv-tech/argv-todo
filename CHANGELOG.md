# Changelog

User-visible changes are recorded here. Release entries will be added when a version is published.

## Unreleased

### Fixed

- Use native Windows console paste instead of enabling bracketed-paste sequences that the Windows input backend cannot parse. Linux and macOS retain bracketed paste.

### Added

- Terminal smoke checks on Linux, macOS, and Windows exercise startup, task persistence, setting saves, and terminal cleanup with temporary storage.
- Show the Cargo package version in red beside the app name in the task header.
- Reusable Rust test and release-build workflows for Linux, macOS, and Windows, with checks on pull requests to `main` and `dev`, versioned build artifacts on `main` pushes, and GitHub releases on `v*` tags.
- Live `task_view` settings for normal and split todo/completed layouts, with Tab pane focus and separate selection and scrolling. Completed children retain unfinished ancestors as dimmed, nonselectable ghost rows.
- A resolved database-path preview below the active database when the draft or saved setting points elsewhere.
- Normal, Visual, Visual Line, and Replace editing in every input field, with counted Vim operators, word/bracket/quote objects, find/till, yank/put, undo/redo, and repeat. Fields open in Insert; Esc enters Normal and another Esc cancels.
- An `Esc` configuration view with the ASCII logo and an inline database-path editor. Saves preserve TOML comments and apply on the next launch.
- A `config.toml` beside the default or overridden database, with a configurable `database_path` and `--db` precedence.
- A fullscreen task tree with nested tasks, inline editing, and Vim-style navigation.
- High, mid, and low priorities with persistent sibling order.
- Search with ancestor context, subtree completion, and deletion undo.
- SQLite storage, schema migrations, and a `--db` location override.
- Unicode-aware editing, bracketed paste, and layouts for narrow terminals.
- Open-source community documentation, issue and pull request templates, cross-platform CI, and dependency update configuration.

### Changed

- Remove the Vim field-editing sections from the help guide and shortcut content.
- Match help to the settings screen with shared branding, a selectable section guide, bold and spaced titles, a shortcut pane for the selected section, and a compact single-column layout. Use j/k or Tab to choose sections and PgUp/PgDn to scroll their shortcuts. Keep scroll progress and close hints visible.
- Return collapses or expands task branches; Space and `x` toggle completion. Child navigation and creation reveal collapsed branches, and search expands the tree.
- Configuration details follow the selected setting, showing database paths for `database_path` and live layout details and a preview for `task_view`.
- Group help shortcuts with highlighted keys, responsive descriptions, page navigation, a scrollbar, and a position indicator. Keep scrolling within the last full page and isolate help input from task commands.
- Split configuration into an editor pane and a paths pane, with stacked sections on narrow terminals and readable path values.
- Keep child counts visible beside long task titles, underline selection, and strike through completed titles. Footer hints fit their available width, and help keeps its close instructions visible while scrolling.
- Renamed the project and executable to `argv-todo`. The default database is now in the platform config directory under `argv-todo/db.sql`. Existing databases can be opened with `--db`.
