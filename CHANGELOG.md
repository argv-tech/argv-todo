# Changelog

User-visible changes are recorded here. Release entries will be added when a version is published.

## Unreleased

### Added

- An `Esc` configuration view with the ASCII logo and an inline database-path editor. Saves preserve TOML comments and apply on the next launch.
- A `config.toml` beside the default or overridden database, with a configurable `database_path` and `--db` precedence.
- A fullscreen task tree with nested tasks, inline editing, and Vim-style navigation.
- High, mid, and low priorities with persistent sibling order.
- Search with ancestor context, subtree completion, and deletion undo.
- SQLite storage, schema migrations, and a `--db` location override.
- Unicode-aware editing, bracketed paste, and layouts for narrow terminals.
- Open-source community documentation, issue and pull request templates, cross-platform CI, and dependency update configuration.

### Changed

- Renamed the project and executable to `argv-todo`. The default database is now in the platform config directory under `argv-todo/db.sql`. Existing databases can be opened with `--db`.
