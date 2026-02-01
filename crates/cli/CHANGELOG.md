# Changelog

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
