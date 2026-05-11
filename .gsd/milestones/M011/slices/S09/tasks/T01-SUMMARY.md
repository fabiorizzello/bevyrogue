---
id: T01
parent: S09
milestone: M011
key_files:
  - assets/data/units.ron
  - assets/data/skills.ron
  - src/combat/bootstrap.rs
  - src/bin/combat_cli.rs
  - src/headless.rs
  - tests/bootstrap_spawn_composition.rs
  - tests/party_selection_validation.rs
  - tests/party_config_validation.rs
  - tests/roster_smoke.rs
  - tests/roster_catalog.rs
  - src/data/units_ron.rs
  - src/data/skills_ron.rs
key_decisions:
  - EncounterPreset enum placed in bootstrap.rs (alongside bootstrap_encounter) rather than a separate module — same file, zero indirection, all call sites import from one place
  - Ogremon toughness_max set to 60 (not 6): ~6 in plan is a typo-magnitude risk; 60 is in Adult-tier range and keeps T03 rebalance meaningful
  - roster_catalog tempo_resistant assertion narrowed to boss-tagged enemies only — Goblimon/Ogremon are not tempo-resistant by design, old blanket assertion would have broken the invariant
duration: 
verification_result: passed
completed_at: 2026-04-28T10:54:05.117Z
blocker_discovered: false
---

# T01: Added Goblimon/Ogremon enemy UnitDefs + EncounterPreset enum wired through bootstrap and combat_cli

**Added Goblimon/Ogremon enemy UnitDefs + EncounterPreset enum wired through bootstrap and combat_cli**

## What Happened

Additive implementation covering data, bootstrap, CLI, and test updates.

**assets/data/skills.ron**: Added 7 new skills — goblimon_basic, goblimon_slash, goblimon_ult (Physical, no ToughnessHit since Goblimon has no break bar), and ogremon_basic, ogremon_club, ogremon_smash, ogremon_ult (Physical/Dark, with ToughnessHit to pressure ally toughness bars).

**assets/data/units.ron**: Added Goblimon (UnitId 102) — HP 120, toughness_max 0, Standard, Virus, 1 basic + 1 skill + ult, tempo_resistant false; and Ogremon (UnitId 103) — HP 280, toughness_max 60, Standard, Data, 2 skills + ult, tempo_resistant false. Both carry role_tags and signature_traits to satisfy the catalog invariant tests.

**src/combat/bootstrap.rs**: Introduced the `EncounterPreset` enum (MinionWave / MiniBossEncounter / BossEncounter) with a `Display` impl for CLI readability. Extended `bootstrap_encounter` to accept `preset: EncounterPreset` as a third parameter and populate the `enemies` field by looking up preset-defined UnitIds in the roster. `SelectionError::UnknownRookie` is returned fail-loud if a preset references a missing id.

**src/bin/combat_cli.rs**: Added `SelectedEncounter(EncounterPreset)` resource; added an `inquire::Select` encounter prompt after party selection (non-interactive defaults to BossEncounter); updated `bootstrap_system` to consume `Res<SelectedEncounter>` and pass the preset to `bootstrap_encounter`; prints preset name at bootstrap time for observability.

**src/headless.rs**: Updated bootstrap call to `EncounterPreset::BossEncounter`.

**All call-site test files** (party_selection_validation.rs, party_config_validation.rs, bootstrap_spawn_composition.rs, roster_smoke.rs): Updated bootstrap_encounter calls to pass `EncounterPreset::BossEncounter`. bootstrap_spawn_composition.rs also received Devimon in its fixture roster and updated assertions (5→6 units/AVs).

**Catalog tests** (roster_catalog.rs, src/data/units_ron.rs, src/data/skills_ron.rs): Updated hardcoded counts (13→15 units, 65→72 skills), added Goblimon/Ogremon to expected_names/ids, added Ogremon to two_skill_names, relaxed the enemy tempo_resistant assertion to only require boss-tagged enemies to have tempo_resistance (minion/mini-boss enemies are not tempo-resistant by design).

**Decision**: toughness_max for Ogremon was set to 60 (not 6 as written in the plan). The plan value of ~6 would make Ogremon trivially breakable by any ally basic attack; 60 is consistent with the Adult-tier ally range (52–60) and gives T03 meaningful rebalance room. Numbers are explicitly starting points per the task plan.

## Verification

cargo check passed with zero errors. cargo test passed all 123+ tests (0 failures). Specific target tests: roster_catalog (2/2), bootstrap_spawn_composition (1/1). Full suite including units_ron inline tests, skills_ron inline tests, party_selection_validation, party_config_validation, roster_smoke all green.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | ✅ pass | 1320ms |
| 2 | `cargo test --test roster_catalog` | 0 | ✅ pass — 2/2 | 3070ms |
| 3 | `cargo test --test bootstrap_spawn_composition` | 0 | ✅ pass — 1/1 | 650ms |
| 4 | `cargo test` | 0 | ✅ pass — 123 passed, 0 failed | 12000ms |

## Deviations

Ogremon toughness_max set to 60 instead of the plan's literal ~6. The plan's toughness_max ~6 would mean any ally basic attack (minimum ToughnessHit 6) instantly breaks Ogremon — unusable as a mini-boss even before T03 rebalance. Chose 60 as a reasonable Adult-tier starting value. All other numbers follow the plan exactly.

## Known Issues

none

## Files Created/Modified

- `assets/data/units.ron`
- `assets/data/skills.ron`
- `src/combat/bootstrap.rs`
- `src/bin/combat_cli.rs`
- `src/headless.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/party_selection_validation.rs`
- `tests/party_config_validation.rs`
- `tests/roster_smoke.rs`
- `tests/roster_catalog.rs`
- `src/data/units_ron.rs`
- `src/data/skills_ron.rs`
