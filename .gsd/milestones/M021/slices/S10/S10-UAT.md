# S10: S10 — UAT

**Milestone:** M021
**Written:** 2026-05-17T06:45:14.487Z

## UAT Type
Integration regression and observability contract verification

## Preconditions
- Working tree contains the completed S10 changes.
- Rust toolchain and dependencies are available locally.
- No additional environment setup is required.

## Steps
1. Run the shared-combat grep gate:
   `rg -E 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat/ --glob '!blueprints/**'`
2. Run the focused regression tests:
   - `cargo test --test patamon_blueprint_seam`
   - `cargo test --test holy_support_resolution`
   - `cargo test --test renamon_precision_runtime`
   - `cargo test --test battery_loop_kernel`
   - `cargo test --test dorumon_predator_runtime`
   - `cargo test --test event_stream`
   - `cargo test --test validation_snapshot`
   - `cargo test --test combat_cli_shared_surface`
3. Run both build modes:
   - `cargo check`
   - `cargo check --features windowed`
4. Inspect the validation snapshot and CLI output to confirm shared diagnostics use generic labels and that blueprint-owned mechanics remain visible only through owner modules.

## Expected Outcomes
- The grep gate returns no matches outside `blueprints/`.
- Focused Patamon, Renamon, Dorumon, event-stream, validation, and CLI tests all pass.
- Both headless and windowed builds pass without new warnings or regressions.
- Shared validation/CLI output reports generic blueprint diagnostics rather than digimon-named runtime fields.

## Edge Cases
- Malformed or foreign Blueprint envelopes remain no-op/rejected rather than mutating the wrong owner state.
- The canonical Dorumon predator transition remains the applied prey-lock transition, not a cap-rejection artifact.
- The grep gate must ignore blueprint-owned files and only validate shared combat surfaces.

## Not Proven By This UAT
- UI/AI preview consumers from later slices.
- Roster-entry blueprint-keying and registry-keyed validation cleanup.
- Any unrelated combat mechanics outside the S10 shared-surface migration.
