# argv-todo

```text
 █████╗ ██████╗  ██████╗ ██╗   ██╗      ████████╗ ██████╗ ██████╗  ██████╗
██╔══██╗██╔══██╗██╔════╝ ██║   ██║      ╚══██╔══╝██╔═══██╗██╔══██╗██╔═══██╗
███████║██████╔╝██║  ███╗██║   ██║█████╗   ██║   ██║   ██║██║  ██║██║   ██║
██╔══██║██╔══██╗██║   ██║╚██╗ ██╔╝╚════╝   ██║   ██║   ██║██║  ██║██║   ██║
██║  ██║██║  ██║╚██████╔╝ ╚████╔╝          ██║   ╚██████╔╝██████╔╝╚██████╔╝
╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═══╝           ╚═╝    ╚═════╝ ╚═════╝  ╚═════╝
```

A terminal todo list with Vim-style keys. Tasks can have children, notes,
priorities, and deadlines. Everything is saved locally in SQLite as you work.
Priority sorting ranks each branch by its highest-priority task, keeping
children with their parents.
Shift+H/L outdents/indents a whole task tree while keeping the selected sort mode.
Settings also offers deadline sorting, with the earliest branch deadline first
and undated branches last. Priority sorting ignores deadlines.

[![CI](https://github.com/argv-tech/argv-todo/actions/workflows/ci.yml/badge.svg)](https://github.com/argv-tech/argv-todo/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/argv-tech/argv-todo)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/argv-tech/argv-todo)](https://github.com/argv-tech/argv-todo/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/argv-tech/argv-todo)](https://github.com/argv-tech/argv-todo/forks)
[![Open issues](https://img.shields.io/github/issues/argv-tech/argv-todo)](https://github.com/argv-tech/argv-todo/issues)
[![Last commit](https://img.shields.io/github/last-commit/argv-tech/argv-todo)](https://github.com/argv-tech/argv-todo/commits)

**[Install](#install-and-run) · [App guide](GUIDE.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md) · [Activity](#project-activity)**

Runs on Linux, macOS, and Windows.

## Install and run

Build from source with a current stable Rust toolchain and a C compiler.
SQLite is bundled; you do not need to install it separately.

```sh
git clone https://github.com/argv-tech/argv-todo.git
cd argv-todo
cargo install --path . --locked
argv-todo
```

This installs `argv-todo` and the optional `argv-todo-daemon` notification
service. If the command is not found, add Cargo's binary directory to your
`PATH`: `~/.cargo/bin` on Linux/macOS or `%USERPROFILE%\.cargo\bin` on Windows.

On macOS, get the compiler with `xcode-select --install`. On Windows, use
Rust's MSVC toolchain and Visual Studio Build Tools with the **Desktop
development with C++** workload.

Run in an interactive terminal with at least 35 columns and 12 rows. Press
`?` for shortcuts and `q` to quit. To try a checkout without installing:

```sh
cargo run --locked -- --db ./target/dev/db.sql
```

That command keeps trial tasks and settings separate from your usual database.

## Getting started

Press `i`, type a task, and press `Enter` to save. Use `j` / `k` to select,
`Space` to complete, and `o` to add a sibling. `i` adds a child when a task
is selected. Press `Esc` twice to discard an edit.

The [app guide](GUIDE.md) covers shortcuts, projects, notes, deadlines,
delete/undo behavior, settings, and database locations and backups.

## Development and support

See [CONTRIBUTING.md](CONTRIBUTING.md) for local setup and checks, and the
[maintainer guide](docs/maintaining.md) for releases. This is a Rust app
built with Ratatui and Crossterm; SQLite storage uses rusqlite.

Report bugs through [GitHub issues](https://github.com/argv-tech/argv-todo/issues).
Include your OS, terminal, app version, and steps to reproduce. See
[SUPPORT.md](SUPPORT.md) for support and [SECURITY.md](SECURITY.md) for
vulnerability reports.

## Project activity

[Commit activity](https://github.com/argv-tech/argv-todo/graphs/commit-activity) ·
[Contributors](https://github.com/argv-tech/argv-todo/graphs/contributors) ·
[Issue tracker](https://github.com/argv-tech/argv-todo/issues)

### Star history

<a href="https://www.star-history.com/?repos=argv-tech%2Fargv-todo&amp;type=date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=argv-tech/argv-todo&amp;type=date&amp;theme=dark" />
    <img alt="argv-todo GitHub stars over time" src="https://api.star-history.com/chart?repos=argv-tech/argv-todo&amp;type=date" width="800" />
  </picture>
</a>

### Contributors

[![argv-todo contributors](https://contrib.rocks/image?repo=argv-tech/argv-todo)](https://github.com/argv-tech/argv-todo/graphs/contributors)

[Changelog](CHANGELOG.md) · [Code of Conduct](CODE_OF_CONDUCT.md) · [MIT license](LICENSE)
