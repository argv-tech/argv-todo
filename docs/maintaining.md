# Maintainer guide

## Repository launch

The intended repository is `https://github.com/argv-tech/argv-todo`. The local files are ready for it; repository settings must be configured after it is created.

1. Create a public repository named `argv-tech/argv-todo` with `main` as the default branch. Leave generated README, license, and ignore files unchecked so the local files remain authoritative.
2. Push the reviewed local history and open a pull request targeting `main` or `dev`, or run CI manually, to confirm that all three CI jobs pass. The checks are `Rust (ubuntu-latest)`, `Rust (macos-latest)`, and `Rust (windows-latest)`.
3. Add a description: “A minimal Vim-style terminal todo app with nested tasks and SQLite storage.” Suggested topics: `rust`, `todo`, `tui`, `ratatui`, `sqlite`, and `vim`.
4. Enable private vulnerability reporting and Dependabot alerts and security updates in the repository's security settings. The committed Dependabot configuration schedules weekly Cargo and GitHub Actions version updates.
5. Add a monitored private contact for security and conduct reports to `SECURITY.md` and `CODE_OF_CONDUCT.md`, or list it on maintainers' GitHub profiles. The policies include a fallback until a contact is available.
6. Add a ruleset for `main`: require pull requests, resolution of review conversations, and the three CI checks; prevent force pushes and branch deletion. Require an approving review when another maintainer is available.
7. Review Actions permissions and contributor workflow approval settings. Tests and builds require only read access to repository contents; the publish job needs write access to create releases and upload assets.

See GitHub's documentation for [private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository) and [repository rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets).

## Reviewing changes

Keep pull requests focused. Check data compatibility, parameterized SQL, transaction behavior, Unicode editing, and keyboard documentation when those areas change. Use sample databases for reproductions.

Review dependency updates before merging. CI checks correctness and compilation; it does not prove every dependency is free of vulnerabilities.

## Releasing

Manual runs and calls from other workflows run the reusable build workflow and upload `argv-todo-<version>-linux.tar.gz`, `argv-todo-<version>-macos.tar.gz`, and `argv-todo-<version>-windows.zip` as workflow artifacts. The version comes from the built binary's Cargo package version, for example `argv-todo-0.1.0-linux.tar.gz`. Each archive contains the release binary for its native GitHub runner.

Pushing a `v*` tag runs the reusable checks on all three platforms, builds the release archives, and publishes a GitHub release with generated notes. Rerunning a tag workflow replaces assets on an existing release. Crates.io publication uses the separate manual workflow or local commands below.

1. Start with a clean working tree on `main` and passing CI on all platforms.
2. Update the version in `Cargo.toml` and refresh `Cargo.lock` with `cargo check`. Regenerate the changelog with `git-cliff --offline --tag v<version>` and review the dated section matching the new version. Do not hand-edit generated entries.
3. Run the checks from `CONTRIBUTING.md`, followed by:

   ```sh
   cargo build --release --locked
   cargo package --locked
   ```

4. Smoke-test `--help`, `--version`, and the task workflow with a disposable database. Confirm migrations with copies of representative old databases rather than personal data.
5. Commit the reviewed release changes. Create and push a tag named `v` followed by the exact Cargo version. Confirm the Publish workflow passes and attaches all three archives to the GitHub release, then update its generated notes with the matching changelog entry.
6. Publish to crates.io using the workflow or local commands below.

Keep the version, tag, and release notes consistent. Do not describe an unreleased version as available on crates.io.

## Publishing to crates.io

The package name and installed command are both `argv-todo`. The manifest restricts publication to crates.io and includes the source, unit tests, license, and linked documentation. SQLite is bundled, so installation requires Rust and a C compiler.

Before the first publication, sign in to [crates.io](https://crates.io), verify your account email, and confirm that `argv-todo` is available or owned by your account. Create an [API token](https://crates.io/settings/tokens) with permission to publish this crate. A first-release token must allow creating the crate. See the [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html) for account and ownership details.

### Publish locally

Commit the reviewed changes first so Cargo can package a clean working tree. Authenticate using the interactive prompt, then inspect and validate the package:

```sh
cargo login --registry crates-io
cargo package --list --locked
cargo publish --dry-run --locked --registry crates-io
```

When validation passes, upload the release:

```sh
cargo publish --locked --registry crates-io
```

Verify installation in a disposable directory, then remove it when finished:

```sh
cargo install argv-todo --version 0.1.0 --locked --root ./target/crates-io-install
./target/crates-io-install/bin/argv-todo --version
```

Use the release's actual version in place of `0.1.0`; on Windows the executable ends in `.exe`. A published version cannot be overwritten, so increment `Cargo.toml` for each subsequent release.

### Publish from GitHub Actions

Add the API token as a repository Actions secret named `CARGO_REGISTRY_TOKEN`. Once `.github/workflows/crates-io.yml` is on the default branch, open **Actions → Publish to crates.io → Run workflow** and select the reviewed release branch or tag.

Leave `dry_run` enabled for validation; this needs no token and performs no upload. The workflow first runs the existing Linux, macOS, and Windows CI checks, lists package contents, and runs Cargo's publishing dry run. To upload the same reviewed revision, run it again with `dry_run` disabled. The upload step uses the repository secret and reports a missing token before publishing. Pushing a GitHub release tag alone does not upload to crates.io.
