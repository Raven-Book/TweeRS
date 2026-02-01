---
tweers-core-full: "patch:fix"
---

Add file sorting to FileCollector to ensure deterministic file processing order across different filesystems and environments. This prevents non-deterministic build behavior caused by random file ordering from fs::read_dir().
