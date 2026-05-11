---
id: T03
parent: S08
milestone: M011
key_files:
  - src/combat/events.rs
  - src/combat/state.rs
  - src/combat/resolution.rs
  - src/data/skills_ron.rs
  - src/combat/follow_up.rs
  - assets/data/units.ron
  - assets/data/skills.ron
  - tests/form_identity.rs
  - tests/ultimate_meter.rs
  - tests/commander_flow.rs
  - tests/damage_breakdown_log.rs
key_decisions:
  - Added damage_tag to OnDamageDealt (not a separate event) — the event already carries the full damage context; embedding tag keeps listener logic self-contained without needing to cross-reference resolved action state
  - SelfAdvance emits TurnAdvance{target:source} (not target:target) — reuses existing apply_turn_advance_system without any new system, and discriminant-matches cleanly against the existing TurnAdvance { target, amount_pct } consumer
  - OnStatusApplied trigger uses std::mem::discriminant comparison — ignores inner field values (speed_reduction etc.) so RON trigger config only needs the discriminant shape, not exact field values
duration: 
verification_result: passed
completed_at: 2026-04-28T10:21:28.471Z
blocker_discovered: false
---

# T03: Wired Garurumon (Ice/GrantEnergy), Kabuterimon (Electric/GrantEnergy), Kyubimon (OnStatusApplied/SelfAdvance) Form Identities — added SelfAdvance effect, damage_tag in OnDamageDealt, tag-specific trigger matching, 4 new integration tests green.

**Wired Garurumon (Ice/GrantEnergy), Kabuterimon (Electric/GrantEnergy), Kyubimon (OnStatusApplied/SelfAdvance) Form Identities — added SelfAdvance effect, damage_tag in OnDamageDealt, tag-specific trigger matching, 4 new integration tests green.**

## What Happened

Extended Form Identity from Greymon's single-unit demo to three more Adults, introducing a new Effect variant and closing the tag-specificity debt left in T02.

**Schema changes:** Added `Effect::SelfAdvance(i32)` to `skills_ron.rs` (distinct from `TurnAdvance` which targets the defender; `SelfAdvance` targets the attacker). Added `self_advance_pct: i32` field to `ResolvedAction` in `state.rs`. Added `damage_tag: DamageTag` field to `CombatEventKind::OnDamageDealt` (previously missing, which T02 noted as blocking real tag-specificity). Updated all callers emitting `OnDamageDealt` (resolution.rs + two test files: ultimate_meter.rs, commander_flow.rs + one test helper: damage_breakdown_log.rs).

**Resolution wiring:** Added `skill_self_advance` extractor in `resolution.rs`. In `apply_effects`, when `self_advance_pct != 0`, emits `TurnAdvance { target: resolved.source, amount_pct }` — targeting the attacker instead of the defender, keeping the two variants cleanly separated.

**Listener extension in follow_up.rs:** Replaced T02's stub `evaluate_form_identity_trigger` with real tag-specific matching: `OnFirstHitVsTagThisRound(tag)` now reads `damage_tag` from the `OnDamageDealt` event and compares directly; `OnStatusApplied(trigger_kind)` uses `std::mem::discriminant` comparison so inner field values (e.g. `speed_reduction`) are irrelevant to the trigger match. Both branches still guard `event.source == follower_id` to ensure only the applier/attacker's own actions trigger Form Identity.

**Data:** Added `form_identity` entries to Garurumon (`OnFirstHitVsTagThisRound(Ice)` → `garurumon_form_identity`), Kabuterimon (`OnFirstHitVsTagThisRound(Electric)` → `kabuterimon_form_identity`), and Kyubimon (`OnStatusApplied(Freeze(speed_reduction:0))` → `kyubimon_form_identity`) in `units.ron`. Added three new skills in `skills.ron`: garurumon_form_identity (`GrantEnergy(5)`), kabuterimon_form_identity (`GrantEnergy(5)`), kyubimon_form_identity (`SelfAdvance(20)`). Skill catalog grows from 60 to 63.

**Tests (tests/form_identity.rs):** Added `spawn_unit_with_fi` generic helper (mirrors `spawn_greymon` but works for any unit with form_identity). Added `drain_combat_events` cursor helper. Added 4 tests: `garurumon_first_ice_hit_grants_energy`, `kabuterimon_first_electric_hit_grants_energy`, `kyubimon_freeze_application_self_advances` (uses `onibidama` which applies Freeze, asserts TurnAdvance{target:kyubimon,20%} emitted + once-per-round block), and the negative test `greymon_fire_trigger_does_not_fire_on_garurumon_ice_hit` (proves tag cross-contamination is impossible).

## Verification

cargo check clean (warnings pre-existing only); cargo test --test form_identity — all 7 tests pass including 4 new ones; cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron — passes with count 63; cargo test (full suite, all test binaries) — 0 failures across all suites.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | ✅ clean compile, warnings only (pre-existing) | 890ms |
| 2 | `cargo test --test form_identity garurumon_first_ice_hit_grants_energy` | 0 | ✅ pass | 120ms |
| 3 | `cargo test --test form_identity kabuterimon_first_electric_hit_grants_energy` | 0 | ✅ pass | 120ms |
| 4 | `cargo test --test form_identity kyubimon_freeze_application_self_advances` | 0 | ✅ pass | 120ms |
| 5 | `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` | 0 | ✅ pass — 63 skills | 200ms |
| 6 | `cargo test` | 0 | ✅ all test suites green, 0 failures | 4800ms |

## Deviations

Minor structural extension: T02 left OnFirstHitVsTagThisRound as a stub matching any damage. T03 closed this by adding damage_tag to OnDamageDealt (not a plan deviation — the plan explicitly called for this). Also fixed 3 test files (ultimate_meter.rs, commander_flow.rs, damage_breakdown_log.rs) and 1 lib test (ultimate.rs) that constructed OnDamageDealt without the new damage_tag field — these were collateral fixes required by the struct change, not planned but mechanical.

## Known Issues

None.

## Files Created/Modified

- `src/combat/events.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/data/skills_ron.rs`
- `src/combat/follow_up.rs`
- `assets/data/units.ron`
- `assets/data/skills.ron`
- `tests/form_identity.rs`
- `tests/ultimate_meter.rs`
- `tests/commander_flow.rs`
- `tests/damage_breakdown_log.rs`
