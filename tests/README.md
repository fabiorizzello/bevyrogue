# Integration tests

All tests run headless (no `windowed` feature). Each scope is aggregated into a single harness binary — see R003 in `.gsd/KNOWLEDGE.md`.

## Scope harnesses

| Harness | Scope | Key coverage |
|---------|-------|-------------|
| `effects_kernel.rs` | effects_kernel | Effect::* primitives (Cleanse, Heal, Revive) |
| `invariants.rs` | invariants | Property-based invariants (proptest) for combat math |
| `windowed_only.rs` | windowed_only | Windowed-only UI features (phase strip, preview cache) |
| `passives_infra.rs` | passives_infra | Passive event infrastructure and reactive canon |
| `blueprints_infra.rs` | blueprints_infra | Blueprint signal dispatch and form identity |
| `follow_up.rs` | follow_up | Follow-up chain semantics, triggers, trigger internals |
| `target_shape.rs` | target_shape | Target shape resolution, bounce, AoE, blast |
| `action_query.rs` | action_query | Action affordance, cast ID propagation, engine legality |
| `tempo_toughness.rs` | tempo_toughness | Tempo resistance, toughness mechanics, attribute triangle |
| `assets_data.rs` | assets_data | RON data files, skill/unit definitions, roster catalog |
| `bootstrap_encounter.rs` | bootstrap_encounter | Encounter bootstrap, spawn composition, end-to-end setup |
| `animation.rs` | animation | Animation graphs, clips, player FSM, asset validation |
| `damage_resolution.rs` | damage_resolution | Damage calculation, block reactions, DR pipeline |
| `turn_economy.rs` | turn_economy | Turn system, SP economy, energy, ultimate meter, streaks |
| `status_effects.rs` | status_effects | Status mechanics, buffs, accuracy, bag operations |
| `preview_ai.rs` | preview_ai | Enemy AI decision routing, skill preview, presentation metadata |
| `runtime_events_obs.rs` | runtime_events_obs | Runtime internals, event bridge/filter, observability, signal bus |
| `timeline.rs` | timeline | Compiled timeline, boundary contracts, pipeline dispatch |
| `digimon_kits.rs` | digimon_kits | Individual Digimon blueprint kits and runtime behaviors |

## Adding new tests

1. Pick the appropriate scope harness (or create a new one if none fits).
2. Add your case file under `tests/<scope>/your_test.rs`.
3. Add `#[path = "<scope>/your_test.rs"] mod your_test;` to `tests/<scope>.rs`.
4. If your test needs shared helpers, use `crate::common::` (the harness declares `mod common;` at the top).

## Shared helpers (`tests/common/`)

Stable submodules:
- `common::units::{attacker, defender, unit}` — `Unit` factory standard.
- `common::actions::{basic_resolved, ult_resolved, default_ult, ready_ult}` — `ResolvedAction` / `UltimateCharge` fixtures.
- `common::apply::{LegacyOpsHarness, ApplyOpts, run_damage, run_ult_delta}` — wrapper on `apply_legacy_ops`.
- Various `build_app`/`*_app` helpers for Bevy scenario setup.

## Parameterization

- **`rstest`** for known-case tables (`#[case]`). Examples: `triangle_matchup.rs` (16-cell attribute matrix), `tempo_resistance.rs` (curve points).
- **`proptest`** for domain-wide invariants (AV floor/ceiling, TempoResistance monotonicity, Blessed immunity to cleanse). Shrunk counterexamples persisted in `tests/proptest-regressions/`.

## Conventions

- No wall-clock, no unseeded RNG.
- Asset RON loaded via inline fixtures or `assets/data/*`.
- File names are functional (no milestone/slice prefix).
- `include_str!` paths are relative to the case file (i.e. use `../../assets/` when the file is in `tests/<scope>/`).
