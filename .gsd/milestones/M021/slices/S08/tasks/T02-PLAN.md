---
estimated_steps: 8
estimated_files: 2
skills_used: []
---

# T02: Convert Gabumon to directory module with twin_core imports

**Why:** Gabumon currently lives in a single flat file (`gabumon.rs`) and imports TwinCore types from `blueprints::agumon::`. After T01, those types live in `blueprints::twin_core::`. This task restructures Gabumon into a proper directory module (matching Agumon's structure) and fixes the import coupling.

**Do:**
1. Create `src/combat/blueprints/gabumon/` directory.
2. Move `src/combat/blueprints/gabumon.rs` → `src/combat/blueprints/gabumon/mod.rs`.
3. Extract signal dispatch logic (the `GabumonSignal` enum, `parse()`, `dispatch()` fns) into `src/combat/blueprints/gabumon/signals.rs`. Re-export `dispatch` and `OWNER` from `mod.rs`.
4. Replace `use crate::combat::blueprints::agumon::{TwinCoreDesignTag, twin_core_added_tag_transition}` with `use crate::combat::blueprints::twin_core::{TwinCoreDesignTag, twin_core_added_tag_transition}`.
5. Verify `src/combat/blueprints/mod.rs` still compiles (`pub mod gabumon;` works for both file and directory modules in Rust — no change needed there).

**Done when:** `cargo test` passes; `rg "blueprints::agumon" src/combat/blueprints/gabumon/` → 0 lines.

## Inputs

- `src/combat/blueprints/gabumon.rs`
- `src/combat/blueprints/twin_core/mod.rs`

## Expected Output

- `src/combat/blueprints/gabumon/mod.rs`
- `src/combat/blueprints/gabumon/signals.rs`

## Verification

cargo test && rg "blueprints::agumon" src/combat/blueprints/gabumon/
