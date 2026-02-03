# Changelog

## v1.0.1

### Performance Improvements

- [`c9b3658`](https://github.com/Raven-Book/TweeRS/commit/c9b36584195e2f93885a0667b4f3d2990c7d99c4): Remove redundant JSON serialization when parsing story format

### New Features

- [`621a696`](https://github.com/Raven-Book/TweeRS/commit/621a69698eaa5e6f3e906db24062c4ed7112e89c): Add fallback mechanism for StoryFormat parsing to skip non-standard fields and support Harlowe format

## v1.0.0

### Refactors

- [`4c4b4d6`](https://github.com/Raven-Book/TweeRS/commit/4c4b4d630086e56b5b984ef7ef505f0c2ad7464e): Split components and package structure, reorganize project layout and dependencies, update modules to fit the new structure.

### Chores

- [`aa5e3ab`](https://github.com/Raven-Book/TweeRS/commit/aa5e3abc89edce0840b5917ce479e900c0cb39bb): Fix Windows compilation error in update module by correcting Send + Sync trait bounds, and apply clippy lint suggestions to improve code quality across all crates.

### Bug Fixes

- [`dc53a6c`](https://github.com/Raven-Book/TweeRS/commit/dc53a6cb8b0193c9c87c48cc9b2fbaec773d985a): Fix StoryData parsing logic to correctly handle multiple source files. Previously, if the first file had no StoryData, subsequent files with valid StoryData would be ignored, causing "Story name is required (missing StoryTitle passage?)" errors. Now only non-None StoryData is used, ensuring the first valid StoryData is always preserved regardless of file order.
