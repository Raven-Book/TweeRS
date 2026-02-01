# Changelog

## v1.0.0

### Refactors

- [`4c4b4d6`](https://github.com/Raven-Book/TweeRS/commit/4c4b4d630086e56b5b984ef7ef505f0c2ad7464e): Split components and package structure, reorganize project layout and dependencies, update modules to fit the new structure.

### Bug Fixes

- [`7bd63da`](https://github.com/Raven-Book/TweeRS/commit/7bd63da2df65f44008795f8bfce705615f8e3354): Add file sorting to FileCollector to ensure deterministic file processing order across different filesystems and environments. This prevents non-deterministic build behavior caused by random file ordering from fs::read_dir().

### Chores

- [`aa5e3ab`](https://github.com/Raven-Book/TweeRS/commit/aa5e3abc89edce0840b5917ce479e900c0cb39bb): Fix Windows compilation error in update module by correcting Send + Sync trait bounds, and apply clippy lint suggestions to improve code quality across all crates.
