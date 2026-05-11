---
estimated_steps: 12
estimated_files: 5
skills_used: []
---

# T01: Extend enemy roster + wire EncounterPreset into bootstrap and CLI

Add a minion and a mini-boss UnitDef to `assets/data/units.ron`, introduce an `EncounterPreset` enum (`MinionWave` / `MiniBossEncounter` / `BossEncounter`), extend `bootstrap_encounter` to accept a preset and populate the `enemies` field of `EncounterComposition`, and add an `inquire::Select` encounter prompt to `combat_cli` (with a non-interactive default of `BossEncounter` to keep CI smoke-tests green). This is purely additive — no engine logic changes, only data + bootstrap signature + CLI UX.

Canonical enemy fixtures (numbers are starting points; T03 will rebalance):
- **Minion** (UnitId 102, e.g. `Goblimon`): HP ~120, toughness 0 (no break bar), Standard category, attribute Virus, basic_damage_tag Physical, no form_identity, no follow_up, tempo_resistant=false, 1 basic + 1 cheap skill, low ult_cap.
- **Mini-boss** (UnitId 103, e.g. `Ogremon`): HP ~280, toughness ~6, Standard or Armored, attribute Data, no form_identity, no tempo resistance, 1 basic + 2 skills + 1 ultimate. Resists none (or 1 weak resist) so the rebalance has room.
- Devimon (UnitId 101) stays the boss anchor unchanged.

Encounter presets:
- `MinionWave` → 3 minions (UnitId 102 ×3).
- `MiniBossEncounter` → 1 mini-boss (103) + 2 minions (102).
- `BossEncounter` → Devimon (101) + 0 supporting minions (matches current behavior the closest).

**bootstrap_encounter** changes: add a second parameter `preset: EncounterPreset`. Populate `enemies` by looking up the preset's enemy ids in the roster (returning `SelectionError::UnknownRookie { id }` if missing — reuse existing error variant). All call sites must pass a preset; the unit test in `bootstrap.rs` (and any test in `tests/bootstrap_spawn_composition.rs`) must be updated.

**CLI** changes: in `src/bin/combat_cli.rs::main()`, after the party `MultiSelect`, add an `inquire::Select` for `EncounterPreset` (3 labels). Non-interactive branch defaults to `BossEncounter`. Insert a new `SelectedEncounter(EncounterPreset)` resource and have `bootstrap_system` consume it when calling `bootstrap_encounter`.

Keep this task strictly mechanical — do not retune numbers here. Verify by parsing the new units.ron and by running the full integration suite (existing tests must still pass; new minion/mini-boss must round-trip through `roster_catalog` invariants).

## Inputs

- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- `src/bin/combat_cli.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/roster_catalog.rs`

## Expected Output

- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- `src/bin/combat_cli.rs`
- `tests/bootstrap_spawn_composition.rs`

## Verification

cargo check && cargo test --test roster_catalog && cargo test --test bootstrap_spawn_composition && cargo test && echo 'OK'

## Observability Impact

- Signals added/changed: encounter preset name printed at CLI startup; enemy spawn events flow through the same OnSpawn / OnDamageDealt path with no new event kind.
- How a future agent inspects this: `cargo run --bin combat_cli` then choose a preset, watch the dashboard show enemy units; or inspect `EncounterComposition` post-bootstrap in a test.
- Failure state exposed: SelectionError::UnknownRookie surfaces if the preset references a missing unit id (fail-loud, not silent fallback).
