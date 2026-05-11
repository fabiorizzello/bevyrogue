---
id: S02
parent: M011
milestone: M011
provides:
  - ["Damage Tag matchup ±25% (R075 validated)", "Attribute Triangle v5.3 multiplicative modifiers (R076 validated)", "CombatRng deterministic resource (R019 closed)", "OnDamageDealt with tag_mod_pct/triangle_mod_pct for JSONL inspection", "OnStatusResisted for observable miss events", "tests/triangle_matchup.rs, tests/status_accuracy.rs, tests/damage_breakdown_log.rs as baseline fixtures for S03+"]
requires:
  []
affects:
  []
key_files:
  - ["src/combat/damage.rs", "src/combat/rng.rs", "src/combat/events.rs", "src/combat/resolution.rs", "src/combat/turn_system/pipeline.rs", "src/combat/types.rs", "src/combat/unit.rs", "src/combat/toughness.rs", "assets/data/units.ron", "assets/data/skills.ron", "tests/triangle_matchup.rs", "tests/status_accuracy.rs", "tests/damage_breakdown_log.rs", "src/combat/damage_tests.rs"]
key_decisions:
  - ["calculate_damage returns DamageBreakdown struct (not bare i32) to expose tag_mod_pct/triangle_mod_pct on OnDamageDealt", "weaknesses passed as &[DamageTag] from Toughness at call site — not embedded in Unit", "dmg_modifier is a single outgoing multiplier (not split dmg_in/dmg_out); asymmetric: defender-wins=0.87, attacker-loses=1.11", "No clamp on multiplicative damage result — discrete modifier set is naturally bounded", "CombatRng seeded [42u8;32] by default in tests; configurable via BootstrapConfig", "OnStatusResisted emitted between OnActionPreApp and OnActionApplied — preserves S01 lifecycle contract"]
patterns_established:
  - ["DamageBreakdown pattern: return structs from formula functions to carry observability fields alongside results", "CombatRng pattern: single seeded Bevy Resource for all RNG; never call thread_rng() inside combat systems", "Observable miss pattern: emit a specific event variant (OnStatusResisted) rather than silently dropping the effect"]
observability_surfaces:
  - ["BEVYROGUE_JSONL=1 log entries include tag_mod_pct and triangle_mod_pct per hit via OnDamageDealt", "OnStatusResisted event makes status misses observable in the event bus (previously silent drops)", "damage_breakdown_log.rs serves as a living integration spec for the formula: inspect it to verify JSONL values"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-27T14:34:19.406Z
blocker_discovered: false
---

# S02: Damage Tag rename + matchup ±25% + Attribute Triangle v5.3 in-line

**Atomically renamed Element→DamageTag, replaced Resistances array with Vec<DamageTag> resists, rewrote damage formula to multiplicative v5.3 model, added CombatRng for deterministic status accuracy, and exposed tag_mod_pct/triangle_mod_pct on OnDamageDealt for JSONL inspection.**

## What Happened

S02 delivered four tightly coupled changes that together align the combat engine to combat_design.md v5.3.

**T01 — Atomic Element→DamageTag rename:** The rename was largely complete from prior sessions; T01 finished the remaining stale references in follow_up_tests.rs, turn_system/tests.rs, turn_system/pipeline.rs, roster_smoke.rs, pipeline_dispatch.rs, and follow_up_triggers.rs. Variant remap applied: Water→Ice, Electro→Electric, Plant→Physical. The Resistances([i8;6]) struct was intentionally preserved for T02. After T01 the full suite (24+ binaries) remained green and grep for 'Element::|: Element|basic_element' returned zero matches.

**T02 — Resistances→Vec<DamageTag> resists + multiplicative damage formula:** Removed the legacy Resistances([i8;6]) struct from types.rs and replaced it with resists: Vec<DamageTag> on both Unit (Bevy component) and UnitDef (RON schema). Rewrote calculate_damage on the v5.3 multiplicative model: tag_mod ∈ {1.25 weak, 0.75 resist, 1.0 neutral} × triangle_mod from triangle_modifiers() × (2.0 if break else 1.0), rounded. The no-clamp design is correct — the discrete multiplier set is naturally bounded. calculate_damage now returns DamageBreakdown { final_damage, tag_mod_pct, triangle_mod_pct } for downstream observability. damage_tests.rs was rewritten with an 18-case matrix plus 4 edge cases. tests/triangle_matchup.rs was added covering all 16 (attacker_attr, defender_attr) pairs asserting the full TriangleMods triple. Key call-site decision: weaknesses are passed as &[DamageTag] from the Toughness component rather than embedded in Unit, keeping the data ownership clean. HP assertions in follow_up_triggers had to be updated (60→49) due to the formula change.

**T03 — CombatRng + OnStatusResisted + status accuracy roll + Shock retrofit:** Introduced src/combat/rng.rs with CombatRng(SmallRng) as a Bevy Resource seeded by bootstrap (default [42u8;32] for tests, configurable via BootstrapConfig). Wired roll_pct(threshold) into pipeline.rs at the status application site; if the roll fails, OnStatusResisted { kind } is emitted between OnActionPreApp and OnActionApplied (preserving the S01 lifecycle contract), and the StatusEffect is not inserted. The pre-existing rand::thread_rng() call in turn_system/mod.rs:267 (Shock cancel roll) was retrofitted to CombatRng, closing R019. tests/status_accuracy.rs was added with three seeded scenarios: attacker-loses miss, attacker-loses hit, and neutral (always hits).

**T04 — tag_mod_pct/triangle_mod_pct on OnDamageDealt + Greymon-vs-Devimon scenario:** Extended OnDamageDealt with tag_mod_pct: i32 and triangle_mod_pct: i32. All exhaustive matchers across event_stream.rs, encounter_e2e.rs, follow_up_triggers.rs, follow_up_reentrancy.rs, combat_coherence.rs, and pipeline_dispatch.rs were updated to use .. for the new fields. tests/damage_breakdown_log.rs was added: Devimon (Virus attacker) vs Greymon (Vaccine defender, resists Fire) produces OnDamageDealt { amount: 83, tag_mod_pct: 75, triangle_mod_pct: 111 } — confirming the JSONL breakdown is correct per the S02 demo requirement. A second scenario covers the weak case with tag_mod_pct: 125.

**Final state:** 28 test binaries, 0 failed. grep for banned symbols (Element::, basic_element, Resistances, : Element, thread_rng) returns zero matches across src/, tests/, and assets/data/.

## Verification

1. `cargo test --no-fail-fast` — 28 binaries, 0 failed across all test targets including the 3 new S02 tests (triangle_matchup, status_accuracy, damage_breakdown_log).
2. `! grep -rn 'Element::|basic_element|Resistances|: Element' src/ tests/ assets/data/` → CLEAN (zero matches).
3. `! grep -rn 'thread_rng' src/combat/` → CLEAN (zero matches).
4. damage_breakdown_log.rs: Devimon (Virus) attacks Greymon (Vaccine, resists Fire) → OnDamageDealt { amount: 83, tag_mod_pct: 75, triangle_mod_pct: 111 } — formula round(100 × 0.75 × 1.11) = 83 confirmed.
5. triangle_matchup.rs: all 16 (attacker_attr, defender_attr) pairs pass with correct TriangleMods triples.
6. status_accuracy.rs: seeded miss (OnStatusResisted emitted, StatusEffect absent) and seeded hit (OnStatusApplied emitted) both pass.
7. pipeline_dispatch.rs: continues to pass; OnStatusResisted lifecycle position (between PreApp and Applied) preserved.

## Requirements Advanced

None.

## Requirements Validated

- R075 — tests/triangle_matchup.rs and damage_tests.rs validate ±25% tag matchup; damage_breakdown_log.rs confirms tag_mod_pct on bus; 28 test binaries green
- R076 — triangle_modifiers() covers all 16 pairs; status_accuracy.rs confirms 0.90 miss penalty on attacker-losing matchup; 28 test binaries green

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

["S03 EvoStage schema can reuse the DamageTag type directly — no Element references remain", "S08 Form Identity will need to read triangle_modifiers() result; the TriangleMods struct is the correct surface", "S09 numerical rebalance should verify TTK targets against damage_breakdown_log.rs fixture values"]

## Files Created/Modified

None.
