---
tweers-cli: "patch:chore"
---

Update CI workflow path filters to match new crates directory structure. Semifold CI now triggers only on Cargo.toml/Cargo.lock changes, and test workflow monitors crates directory instead of src.
