# Contributing to argv-todo

Bug reports, documentation improvements, tests, and focused code changes are welcome. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Before you start

Search [existing issues](https://github.com/argv-tech/argv-todo/issues) before opening a new one. For a larger change, describe the problem and proposed behavior in an issue first so maintainers can discuss the scope.

The project favors a small dependency set and a single task tree with parents and children shown together. Keep the existing keyboard workflow and platform-specific storage behavior. Avoid adding dashboards, separate task pages, or unrelated refactors.

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
- Add a concise entry to the appropriate section of `CHANGELOG.md` for user-visible changes.

The source layout and project constraints are described in [AGENTS.md](AGENTS.md).

## Checks

Run these before submitting code changes:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

Use `cargo fmt` to fix formatting. Commit `Cargo.lock` when a dependency change updates it. CI runs the same checks on Linux, macOS, and Windows.

## Pull requests

Describe the problem, resulting behavior, and checks you ran. Include an issue link when one exists. For interface changes, a small terminal capture can help reviewers; use sample tasks and remove personal data.

Maintainers may ask for revisions or decline changes that expand the project's scope. Contributions are reviewed as time permits.

## License

By submitting a contribution, you agree that it may be distributed under the project's [MIT license](LICENSE). Preserve notices for any third-party material you include.
