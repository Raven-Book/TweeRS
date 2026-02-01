# Changelog

## v1.0.0

### Chores

- [`aa5e3ab`](https://github.com/Raven-Book/TweeRS/commit/aa5e3abc89edce0840b5917ce479e900c0cb39bb): Fix Windows compilation error in update module by correcting Send + Sync trait bounds, and apply clippy lint suggestions to improve code quality across all crates.

### Refactors

- [`4c4b4d6`](https://github.com/Raven-Book/TweeRS/commit/4c4b4d630086e56b5b984ef7ef505f0c2ad7464e): Split components and package structure, reorganize project layout and dependencies, update modules to fit the new structure.

### New Features

- [`18f6879`](https://github.com/Raven-Book/TweeRS/commit/18f6879bb2f0fa743aaef84e12a9963bf29d9cda): Integrate JavaScript hook scripts support in CLI. Scripts in `scripts/data/` and `scripts/html/` directories are now automatically detected and executed during the build pipeline at appropriate stages.
