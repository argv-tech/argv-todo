# Contributing to argv-todo

Bug reports, documentation improvements, tests, and focused code changes are welcome. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Before you start

Search [existing issues](https://github.com/argv-tech/argv-todo/issues) before opening a new one. For a larger change, describe the problem and proposed behavior in an issue first so maintainers can discuss the scope.

The project favors a small dependency set and a normal task tree with parents and children shown together. The optional split todo/completed layout shares the same task workflow. Keep the existing keyboard controls and platform-specific storage behavior. Avoid adding dashboards, separate task pages, or unrelated refactors.

## Local setup

Install the current stable Rust toolchain and a C compiler. SQLite is bundled. Fork the repository, then clone your fork and create a branch:

```sh
git clone https://github.com/YOUR-USERNAME/argv-todo.git
cd argv-todo
git switch -c your-change
cargo build --locked
```

Use a disposable database when running the app during development:

```sh
cargo run --locked -- --db ./target/dev/db.sql
```

Do not use your personal database for verification. Database tests must use temporary or in-memory storage.

## Making a change

- Keep changes small and describe their user-visible effect.
- Route keyboard input through `src/vim_motion/`.
- Keep SQL parameterized and task-tree operations atomic.
- Preserve existing data, parent links, priorities, and sibling order during migrations.
- Update `README.md` and help text when behavior or keybindings change.
- Add a regression test for a bug or changed behavior when it provides useful coverage. Documentation-only changes do not need new tests.
- Describe user-visible changes in clear Conventional Commit messages so they appear in the generated changelog.

The source layout and project constraints are described in [AGENTS.md](AGENTS.md).

## Source layout

`src/main.rs` owns startup and terminal cleanup; `src/cli.rs` parses command-line options. Larger concerns use directory modules:

- `src/app/`: state, action routing, editing, events, tree navigation, folding, settings, and task operations. `folding.rs` owns session collapse state, branch visibility, and selection preservation when folding. `help.rs` owns help section selection and scrolling. `views.rs` owns pane focus, selection, and task projections, including ghost ancestor rows.
- `src/ui/`: full-frame composition, shared branding and geometry, Unicode labels and editor rendering, header and footer, help, and theme. `tasks/` separates pane composition, draft placement, and row rendering; `config/` composes setting controls and selected-setting details, including paths and layout previews, into responsive panes; `help/` separates shortcut content and wrapping, section-guide and footer chrome, and responsive frame composition. `branding.rs` shares the logo and compact app title between settings and help.
- `src/db/`: task types, queries, schema migrations, and atomic tree operations.
- `src/config/`: configuration loading, validation, and atomic saving.
- `src/vim_motion/`: input targets, modes, and actions. `manager/` maps task keys, field commands, and help navigation, with a typed pending-command parser. `editor/` owns Unicode cursor and selection state, edits, motions, text objects, undo/repeat history, and the horizontal viewport.
- `tests/unit/`: all unit suites and test helpers, grouped by owning module; keyboard and editor suites live in `tests/unit/vim_motion/`. `terminal_smoke.py` checks the built app in a disposable platform terminal.

Each directory's `mod.rs` defines its interface. Keep implementation modules private with `mod`; use `pub(super)` or `pub(crate)` for the access callers need and re-export shared types. Add focused files within the appropriate directory instead of expanding unrelated modules.

Keep all tests and test helpers under `tests/unit/`. Source modules load these files using `#[cfg(test)]` and a relative `#[path = "..."] mod tests;` declaration. This keeps unit tests in their owning module's scope, including access to private helpers, while storing them outside `src/`. Cargo runs them with `cargo test --locked`; `tests/unit/` files are not separate integration-test targets. Include `tests/**` in the package so these paths also work in packaged source.

## Checks

Run these before submitting code changes:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

Use `cargo fmt` to fix formatting. Commit `Cargo.lock` when a dependency change updates it. CI runs the same checks on Linux, macOS, and Windows for pull requests targeting `main` or `dev`.

After building, run `python3 tests/unit/terminal_smoke.py target/debug/argv-todo` on Linux or macOS, or `python tests/unit/terminal_smoke.py target/debug/argv-todo` on Windows. This check requires Python 3.11 or newer and uses only the standard library. It creates disposable storage and a Unix PTY or Windows console to verify startup, input, persisted tasks and settings, and terminal cleanup. CI runs it on each platform after the build.

The reusable workflows are `.github/workflows/ci.yml` for checks and `.github/workflows/build.yml` for release builds and archives. Other workflows call them with a job-level `uses: ./.github/workflows/ci.yml` or `uses: ./.github/workflows/build.yml`. Both workflows support manual runs; `.github/workflows/publish.yml` calls both workflows to test, build, and publish GitHub releases on `v*` tags. `.github/workflows/crates-io.yml` supports manual crates.io validation and publication after the reusable CI checks pass. See the [maintainer guide](docs/maintaining.md#publishing-to-cratesio) for credentials and publishing steps.

## Changelog

Install git-cliff 2.14 or newer, then generate the changelog from committed history:

```sh
git-cliff --offline
```

The repository's `cliff.toml` writes to `CHANGELOG.md`. Entries use plain headings, scopes, commit links, and explicit breaking-change markers. Release tags use `vMAJOR.MINOR.PATCH`; the existing `v.0.1.0` tag is also recognized. Unreleased commits appear above dated releases. Release-preparation commits are omitted, dependency changes are retained, and breaking changes are never skipped.

Review the generated diff before committing it. Regeneration replaces `CHANGELOG.md`, so describe changes in commit messages instead of editing generated entries. To preview unreleased changes in a separate file, run `git-cliff --offline --unreleased --output release-notes.md`.

## Pull requests

Describe the problem, resulting behavior, and checks you ran. Include an issue link when one exists. For interface changes, a small terminal capture can help reviewers; use sample tasks and remove personal data.

Maintainers may ask for revisions or decline changes that expand the project's scope. Contributions are reviewed as time permits.

## License

By submitting a contribution, you agree that it may be distributed under the project's [MIT license](LICENSE). Preserve notices for any third-party material you include.
