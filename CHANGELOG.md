# Changelog

Notable changes to argv-todo, grouped by release.

## 0.1.0 - 2026-10-04

### Added

- Add Vim key handling and Unicode text editing ([24d96c0](https://github.com/argv-tech/argv-todo/commit/24d96c04440a850854e892eefb121698272d5570))
- Persist todos in platform config directories with SQLite ([ecd386c](https://github.com/argv-tech/argv-todo/commit/ecd386c9c6fae696c6c2fb2f066c807d48905e8b))
- Build fullscreen Ratatui todo app with Vim workflows ([090f456](https://github.com/argv-tech/argv-todo/commit/090f456a2b5430417a064484730f29aab6095556))
- Replace filter tabs with nested todo navigation ([e1c898e](https://github.com/argv-tech/argv-todo/commit/e1c898e6e371ecb8729cf7785fc5edb3568dba0d))
- Show all tasks in a compact nested tree ([c8fbcc1](https://github.com/argv-tech/argv-todo/commit/c8fbcc15d1a8f38d25ab3e7be25238e7e9bcee1a))
- Add task priorities and inline tree editing ([d090994](https://github.com/argv-tech/argv-todo/commit/d0909947d1b4dd4c17ae1842784fc3d377b22372))
- Create children and insert siblings with Vim keys ([e95e56b](https://github.com/argv-tech/argv-todo/commit/e95e56b923ab82793c68aa1ee5b34b3b84ed5719))
- Inherit parent priority when creating child tasks ([68720d7](https://github.com/argv-tech/argv-todo/commit/68720d7fd18212809713a0d6a6bcf40f991dd78a))
- Add TOML configuration beside database ([46fe390](https://github.com/argv-tech/argv-todo/commit/46fe390c4e4fc255e93e1f6f8b7a1c55ae07f5e5))
- Add Esc configuration UI with ASCII logo ([8ffca3d](https://github.com/argv-tech/argv-todo/commit/8ffca3d026180d6e27a6bfc09f8610e1bc93b377))
- Add Vim modes, operators, and text objects to all inputs ([9ff917b](https://github.com/argv-tech/argv-todo/commit/9ff917bd5140da5176c1f0efd6ce1e61feb57a8f))
- **config:** Preview database path after restart ([d59944e](https://github.com/argv-tech/argv-todo/commit/d59944e213414592f77c10f39ba460d577fc1661))
- **ui:** Improve help menu layout and navigation ([f2a3457](https://github.com/argv-tech/argv-todo/commit/f2a34578ef9ebef22f9fe721b590d295fdff223b))
- **ui:** Add live split task view with ghost parent context ([16194cb](https://github.com/argv-tech/argv-todo/commit/16194cba173b02192ff6fdccc8feb155781e1485))
- **ui:** Show app version and setting-specific configuration details ([b216b7a](https://github.com/argv-tech/argv-todo/commit/b216b7af0e6be2e98b03f7350647413bbebe14e0))
- Base config settings ([e169388](https://github.com/argv-tech/argv-todo/commit/e169388e25b58ca421a348143a9ae0cb0dccbddf))
- **ui:** Collapse and expand task branches with Return ([6d19370](https://github.com/argv-tech/argv-todo/commit/6d19370e7a3def7893edb59d36cb2eb989123a8e))
- **ui:** Redesign help with selectable sections ([758a034](https://github.com/argv-tech/argv-todo/commit/758a034773162e20a1c4fee4c6d4372fc4b4c462))

### Fixed

- Toggle parent task descendants atomically ([491ad1e](https://github.com/argv-tech/argv-todo/commit/491ad1ebb35a19043601e79a36b4c3173fde4dc5))
- Preserve task colors on focus and completion ([dad6506](https://github.com/argv-tech/argv-todo/commit/dad6506d822ad4ff9d9923e3dd9d6da0591fdc01))
- **ui:** Display app version in red ([78045ba](https://github.com/argv-tech/argv-todo/commit/78045ba8bc7fb4df55c08f16e51ab0a8563a1af7))
- **ui:** Remove Vim field editing sections from help ([1c2b0f1](https://github.com/argv-tech/argv-todo/commit/1c2b0f1f2900fee273486b9fb6034dc2436711ef))
- **platform:** Improve macOS and Windows compatibility ([7918df5](https://github.com/argv-tech/argv-todo/commit/7918df5e19fe1e1710c9813eea0f84825e793758))

### Changed

- Split app into focused modules and document development ([a8e21cb](https://github.com/argv-tech/argv-todo/commit/a8e21cb9bf3cf24da2853d90f0af21a9f32b843e))
- **ui:** Split configuration into responsive panes ([f830023](https://github.com/argv-tech/argv-todo/commit/f8300237ee48c50aa9a89ab23732d64d89729e80))

### Documentation

- Document setup storage locations and keyboard shortcuts ([07b21e4](https://github.com/argv-tech/argv-todo/commit/07b21e45c2a2ce76439d715921b6737de3397f6a))
- Explain nested tasks and hierarchy keybindings ([03b40f8](https://github.com/argv-tech/argv-todo/commit/03b40f857be576b00528001c3b35732ec1ccb5b1))
- Add minimal app development guidance ([3fed093](https://github.com/argv-tech/argv-todo/commit/3fed093554b73444d511a91cfba85f87df8d0584))
- Describe the unified tree and task navigation ([91e601f](https://github.com/argv-tech/argv-todo/commit/91e601f09ba7cc86ad4928269f4ff99d147a4ec4))
- Explain subtree completion and minimal layout ([7e6f565](https://github.com/argv-tech/argv-todo/commit/7e6f56570e155cb9ee890351884035ee2d953b80))
- Require conventional commit messages ([949afcb](https://github.com/argv-tech/argv-todo/commit/949afcb669c2f289ecdbe238d68918c2e4e711cb))

### Testing

- Move unit tests and helpers into tests directory ([985cac4](https://github.com/argv-tech/argv-todo/commit/985cac49ab1c40758c09e90064128d46c2320bc3))

### Continuous Integration

- Add reusable cross-platform workflows and versioned release artifacts ([7739a7c](https://github.com/argv-tech/argv-todo/commit/7739a7ceef36e8d7f48d55fada9693d69a6ba326))

### Maintenance

- Initialize Rust project and app dependencies ([0c0a776](https://github.com/argv-tech/argv-todo/commit/0c0a7768734e481d48d489146294fcaa034c2260))
- Simplify task tree using terminal colors ([494050a](https://github.com/argv-tech/argv-todo/commit/494050a4b7ac85a73acc29e21efecd4dbdc40356))
- Rename to argv-todo and prepare open-source repository ([93869e8](https://github.com/argv-tech/argv-todo/commit/93869e88fa60a56e9ba1ed4475476daaddd56460))
- Add Ratatui development skills ([5aa1a75](https://github.com/argv-tech/argv-todo/commit/5aa1a75ae62602e1cfe2e5dd5d1c08188e516cf6))
