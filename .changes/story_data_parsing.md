---
tweers-core: "patch:fix"
---

Fix StoryData parsing logic to correctly handle multiple source files. Previously, if the first file had no StoryData, subsequent files with valid StoryData would be ignored, causing "Story name is required (missing StoryTitle passage?)" errors. Now only non-None StoryData is used, ensuring the first valid StoryData is always preserved regardless of file order.
