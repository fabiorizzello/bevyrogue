# S04: Roster ready assets and real hot reload proof — UAT

**Milestone:** M001
**Written:** 2026-05-19T08:00:34.704Z

# S04 UAT — Roster Ready Assets and Manual Hot-Reload Proof

## UAT Type
Operational (manual `cargo run --features windowed` proof for R006; headless tests for R007)

## Preconditions
- Rust toolchain installed; project builds with `cargo build`.
- Feature flag `windowed` available and gated correctly (no winit/wgpu outside the feature).
- Renamon assets present: `assets/digimon/renamon/clip.ron`, `assets/digimon/renamon/anim_graph.ron`.
- Agumon assets present: `assets/digimon/agumon/clip.ron`, `assets/digimon/agumon/anim_graph.ron`.

## Headless Verification Steps (automated)

1. Run `cargo test --test anim_asset_validation`.
   **Expected:** 4 tests pass — `valid_assets_set_plugin_validation_ready`, `agumon_real_assets_validate_correctly`, `renamon_real_assets_validate_correctly`, `broken_assets_set_failed_state_with_typed_diagnostics`. Exit 0.

2. Run `ls assets/digimon/renamon/clip.ron assets/digimon/renamon/anim_graph.ron`.
   **Expected:** Both paths listed. Exit 0.

3. Run `grep -q "AnimationValidationState" src/windowed.rs`.
   **Expected:** Exit 0 (symbol present in roster panel code).

4. Run `cargo check --features windowed`.
   **Expected:** Exit 0; no compile errors under the windowed feature gate.

## Manual Hot-Reload UAT Steps (R006)

1. Launch the game: `cargo run --features windowed`.
   **Expected:** Application opens. Windowed roster panel shows validation status labels (PENDING → READY) as assets load. No crash on startup.

2. Observe the roster panel for Agumon and Renamon entries.
   **Expected:** Both entries eventually show GREEN (READY) with 0 errors.

3. While the app is running, open `assets/digimon/agumon/anim_graph.ron` in an editor and introduce a deliberate invalid node type (e.g. change a valid node name to `"bogus_node"`). Save the file.
   **Expected:** The app detects the hot-reload, re-validates the graph, and the Agumon entry in the roster panel turns RED (FAILED) with an error count ≥ 1. A typed diagnostic is logged to the console naming the offending file. The world state remains intact (no crash, no corrupted ECS).

4. Revert the edit in step 3 to restore the valid `anim_graph.ron`. Save the file.
   **Expected:** The app re-validates and the Agumon entry returns to GREEN (READY). No crash. World state intact.

5. Repeat steps 3–4 for `assets/digimon/renamon/anim_graph.ron`.
   **Expected:** Same behavior — FAILED on bad data, READY on restore, no crash.

## Edge Cases

- **Broken fixture at boot:** If `assets/digimon/agumon/anim_graph.ron` is corrupted before launch, the roster panel must show FAILED for Agumon at startup, not a panic.
- **Invalid hot-reload keeps last valid state:** After a broken asset is hot-reloaded, the last known-good asset should remain in use until a valid replacement arrives.
- **Renamon validates identically to Agumon:** The same validator code path handles both; no Digimon-specific branch should be needed.

## Not Proven By This UAT

- Runtime animation playback or FSM execution (R009, deferred).
- Full gameplay integration or combat authority (not in scope for M001).
- Agumon clip.ron geometry parity against atlas (blocked by pre-existing S02 regression in `clip_geometry_parity` test — must be remediated before M001 milestone validation).
- Full `cargo test` suite passing (blocked by the same S02 regression).
