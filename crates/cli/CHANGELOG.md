# Changelog

## v1.0.5

### Bug Fixes

- [`2b7bdfe`](https://github.com/Raven-Book/TweeRS/commit/2b7bdfebb9f2c9dd23bf23823edc17d4e04e5d2f): Fix passage header parsing panic when tags contain non-ASCII characters (e.g. Chinese)

## v1.0.4

### Bug Fixes

- [`6cc15d9`](https://github.com/Raven-Book/TweeRS/commit/6cc15d9d58ce94c61924e66698533881f320abd3): Apply start_passage override if config provided

## v1.0.3

### Bug Fixes

- [`563a497`](https://github.com/Raven-Book/TweeRS/commit/563a497908d04a14c89d33503137461fbbe86200): Unify file processing logic and fix sorting bug in WASM module that caused incorrect output

## v1.0.2

### New Features

- [`621a696`](https://github.com/Raven-Book/TweeRS/commit/621a69698eaa5e6f3e906db24062c4ed7112e89c): Add fallback mechanism for StoryFormat parsing to skip non-standard fields and support Harlowe format

### Performance Improvements

- [`c9b3658`](https://github.com/Raven-Book/TweeRS/commit/c9b36584195e2f93885a0667b4f3d2990c7d99c4): Remove redundant JSON serialization when parsing story format

## v1.0.1

### Bug Fixes

- [`7038612`](https://github.com/Raven-Book/TweeRS/commit/7038612eddda56e5c67b2337eb848d7aeaa39d15): Fix version comparison in update command to correctly parse release tags with 'tweers-cli-v' prefix, preventing unnecessary re-downloads of the same version.

## v1.0.0

### Chores

- [`701795b`](https://github.com/Raven-Book/TweeRS/commit/701795b403364a47c8019c6bcfd93c8cc424d49f): Update CI workflow path filters to match new crates directory structure. Semifold CI now triggers only on Cargo.toml/Cargo.lock changes, and test workflow monitors crates directory instead of src.
- [`a218e80`](https://github.com/Raven-Book/TweeRS/commit/a218e80c7e70918fb13d1807e2b84d80f77e41bf): Merge build workflow into Semifold CI, consolidate multi-platform build and release process into a single workflow.

### Refactors

- [`4c4b4d6`](https://github.com/Raven-Book/TweeRS/commit/4c4b4d630086e56b5b984ef7ef505f0c2ad7464e): Split components and package structure, reorganize project layout and dependencies, update modules to fit the new structure.

### New Features

- [`18f6879`](https://github.com/Raven-Book/TweeRS/commit/18f6879bb2f0fa743aaef84e12a9963bf29d9cda): Integrate JavaScript hook scripts support in CLI. Scripts in `scripts/data/` and `scripts/html/` directories are now automatically detected and executed during the build pipeline at appropriate stages.

### Bug Fixes

- [`aa5e3ab`](https://github.com/Raven-Book/TweeRS/commit/aa5e3abc89edce0840b5917ce479e900c0cb39bb): Fix Windows compilation error in update module by correcting Send + Sync trait bounds, and apply clippy lint suggestions to improve code quality across all crates.
