# Maintainer guide

## Repository launch

The intended repository is `https://github.com/argv-tech/argv-todo`. The local files are ready for it; repository settings must be configured after it is created.

1. Create a public repository named `argv-tech/argv-todo` with `main` as the default branch. Leave generated README, license, and ignore files unchecked so the local files remain authoritative.
2. Push the reviewed local history and confirm that all three CI jobs pass. The checks are `Rust (ubuntu-latest)`, `Rust (macos-latest)`, and `Rust (windows-latest)`.
3. Add a description: “A minimal Vim-style terminal todo app with nested tasks and SQLite storage.” Suggested topics: `rust`, `todo`, `tui`, `ratatui`, `sqlite`, and `vim`.
4. Enable private vulnerability reporting and Dependabot alerts and security updates in the repository's security settings. The committed Dependabot configuration schedules weekly Cargo and GitHub Actions version updates.
5. Add a monitored private contact for security and conduct reports to `SECURITY.md` and `CODE_OF_CONDUCT.md`, or list it on maintainers' GitHub profiles. The policies include a fallback until a contact is available.
6. Add a ruleset for `main`: require pull requests, resolution of review conversations, and the three CI checks; prevent force pushes and branch deletion. Require an approving review when another maintainer is available.
7. Review Actions permissions and contributor workflow approval settings. CI requires only read access to repository contents.

See GitHub's documentation for [private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository) and [repository rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets).

## Reviewing changes

Keep pull requests focused. Check data compatibility, parameterized SQL, transaction behavior, Unicode editing, and keyboard documentation when those areas change. Use sample databases for reproductions.

Review dependency updates before merging. CI checks correctness and compilation; it does not prove every dependency is free of vulnerabilities.

## Releasing

No release or crates.io publication is triggered automatically by this setup.

1. Start with a clean working tree on `main` and passing CI on all platforms.
2. Update the version in `Cargo.toml` and refresh `Cargo.lock` with `cargo check`. Move the relevant changelog entries into a dated section matching the new version; keep an `Unreleased` section for future work.
3. Run the checks from `CONTRIBUTING.md`, followed by:

   ```sh
   cargo build --release --locked
   cargo package --locked
   ```

4. Smoke-test `--help`, `--version`, and the task workflow with a disposable database. Confirm migrations with copies of representative old databases rather than personal data.
5. Commit the reviewed release changes. Create and push a tag named `v` followed by the exact Cargo version, then create a GitHub release using the matching changelog entry.
6. If publishing to crates.io, first confirm the crate name is available, ownership is configured, and the package contents are correct. Run `cargo publish --dry-run --locked` before an intentional `cargo publish --locked`.

Keep the version, tag, and release notes consistent. Do not describe an unreleased version as available on crates.io.
