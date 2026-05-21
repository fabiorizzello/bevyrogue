---
estimated_steps: 3
estimated_files: 6
skills_used: []
---

# T01: Bootstrap a windowed Agumon-vs-Agumon-dummy encounter with two on-screen sprites

Why: today the windowed app starts with no units on screen and no party selection, so the slice cannot show any combat. Establish the smallest end-to-end pass that lights up the visible scene and gives every later task something to verify against.

Do: (1) extend `src/combat/encounter/bootstrap.rs` with a new `EncounterPreset::AgumonTrainingDummy` whose enemy side is one Agumon-shaped `UnitDef` cloned from the Ally roster entry but rebranded onto `Team::Enemy` with a stable enemy `UnitId` distinct from the ally Agumon (do not edit `unit.ron`; assemble the dummy by cloning the loaded def and overriding `team`/`id`/`name`). Keep the enum exhaustive — handle the new variant in `Display`, `bootstrap_encounter`, `src/bin/combat_cli.rs`, and `src/bin/combat_cli/config.rs` (no behavior change in the CLI binaries beyond compile-time enum exhaustiveness). (2) Add a windowed-only `Startup` system in `src/windowed/mod.rs` that, once `DataReady` and the roster/party handles are loaded, runs the new preset through `apply_composition` and emits `PartySelected` + `TurnOrderSeeded` via `MessageWriter<CombatEvent>` (mirror the `headless.rs:200-283` pattern but for one ally Agumon vs the new dummy; SP cap to a large value so resource shortfalls do not block the demo). Seed `agumon::bouncing_fire` rank=1 on the player-side Agumon via the existing `TalentRanks` resource. (3) Extend `src/windowed/render.rs` to spawn one sprite per combatant unit (per-`Unit` query with `Added<Unit>` or a once-per-startup pass), placing the player at a fixed left transform and the dummy at a fixed right transform (mirrored `Sprite { flip_x: true, .. }`); both reuse the existing `AGUMON_STANCE_GRAPH_ID` / `AGUMON_SKILL_GRAPH_ID` and the existing `advance_agumon_presentation` system updated to be unit-scoped instead of singleton-scoped.

Done-when: a `cargo build --features windowed` succeeds; a new `tests/encounter_bootstrap_windowed.rs` test asserts the new preset returns exactly one ally Agumon and one enemy Agumon-shaped dummy with a distinct `UnitId` and `Team::Enemy`; `cargo test --features windowed --test windowed_preview_cache` keeps passing.

## Inputs

- `src/combat/encounter/bootstrap.rs`
- `src/headless.rs`
- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `src/bin/combat_cli.rs`
- `src/bin/combat_cli/config.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `assets/data/digimon/agumon/unit.ron`

## Expected Output

- `src/combat/encounter/bootstrap.rs`
- `src/bin/combat_cli.rs`
- `src/bin/combat_cli/config.rs`
- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `tests/encounter_bootstrap_windowed.rs`

## Verification

cargo test --test encounter_bootstrap_windowed --features windowed

## Observability Impact

PartySelected + TurnOrderSeeded events now fire in windowed mode just like headless, so downstream observability (turn-order panel, combat panel, validation snapshot) lights up the same way.
