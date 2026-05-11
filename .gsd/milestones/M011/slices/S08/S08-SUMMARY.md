---
id: S08
parent: M011
milestone: M011
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["D046: Engine re-entrancy cap removed — chains bounded by data not engine guard. resolve_follow_up_action_system depth increment fixed to origin+1.", "FormIdentityKit as separate ECS component (not extending UnitSkills) to avoid 25+ construction sites", "GrantEnergy applied in step_app via separate Query<&mut Energy>, not inside apply_effects, to preserve 15 resolution_tests callsites", "damage_tag added to OnDamageDealt (not a separate event) — keeps listener logic self-contained", "SelfAdvance emits TurnAdvance{target:source} (not target:target) — reuses existing apply_turn_advance_system consumer with no new system", "OnStatusApplied trigger uses std::mem::discriminant comparison — ignores inner field values so RON only needs discriminant shape", "OnAttackVsAttribute builds per-invocation HashMap<UnitId,Attribute> from FormIdentityRosterQuery — avoids touching FormIdentitySnapshot struct", "BonusToughnessDamage/BonusDamageVsAttribute added to schema but unused in S08 — simpler fire-a-separate-skill approach used; reserved S09", "Angemon negative test in fresh app instance — stale OnDamageDealt events from prior resolve_follow_up_action_system re-trigger listener within same app (MEM029 class)"]
patterns_established:
  - ["Form Identity listener mirrors follow_up_listener_system structure: build snapshots once, HashSet guard for dedup, emit FollowUpIntent with origin_kind tag", "RoundFlags boolean pattern: add flag, reset in advance_turn_system Part 1, flip in resolver post-execution (same as break_sealed, S07)", "Once-per-round conditional bonuses: data-configured, zero new engine systems, reuse FollowUpIntent scheduler", "New Effect variants follow the pattern: add to enum → add field to ResolvedAction → add extractor in resolution.rs → wire in apply_effects"]
observability_surfaces:
  - ["CombatEventKind::EnergyGained{unit_id, amount} — emitted on every Form Identity GrantEnergy application, visible in BEVYROGUE_JSONL=1 stream", "CombatEventKind::TurnAdvance{target:self} — emitted by SelfAdvance effect (Kyubimon), uses existing TurnAdvance consumer", "form_identity_listener_system emits info!/debug! traces under target 'combat.form_identity'", "RoundFlags.form_identity_used — queryable in tests/debug builds for once-per-round verification"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T10:34:27.928Z
blocker_discovered: false
---

# S08: Form Identity framework + 6 Adult wired + rimozione D026 cap (D046)

**Form Identity framework delivered end-to-end: 6 MVP Adults each have a once-per-round conditional bonus configured 100% in RON, follow-up re-entrancy cap removed (D046), 65 skills, 29 integration binaries green.**

## What Happened

S08 delivered the Form Identity system from scratch across 4 tasks.

**T01 — D026 cap removal + chain semantics rewrite:** Deleted the `OneHopSuppressed` guard (`follow_up_depth >= 1` check in evaluate_follow_up) and its enum variant from follow_up.rs. The preserved check in ultimate.rs:77 (OnAllyFollowUp ult-charge gating) was left intact. Fixed the depth increment in resolve_follow_up_action_system from hardcoded `1` to `intent.origin.follow_up_depth + 1` — required to produce observable depth-2+ events for D046. Deleted tests/follow_up_reentrancy.rs; created tests/follow_up_chains.rs asserting chain progression to depth 2 and natural termination when preconditions don't re-fire.

**T02 — Form Identity infrastructure + Greymon canonical demo:** Built the full pipeline from schema to integration test. Added FormIdentityTrigger (4 variants: OnFirstHitVsTagThisRound, OnStatusApplied, OnFirstSkillCastWithTag, OnAttackVsAttribute), FormIdentityConfig, and FormIdentityKit as a separate ECS component (not extending UnitSkills — avoids touching 25+ construction sites). Added form_identity_used to RoundFlags, GrantEnergy(i32) to Effect enum, EnergyGained{unit_id, amount} to CombatEventKind. form_identity_listener_system parallels follow_up_listener_system; energy grant handled in step_app via separate Query<&mut Energy> (not inside apply_effects) to preserve 15 resolution_tests callsites. FollowUpOriginKind enum distinguishes FormIdentity vs FollowUp intents in the resolver. form_identity_used reset in advance_turn_system alongside break_sealed. Greymon (OnFirstHitVsTagThisRound(Fire)→greymon_form_identity GrantEnergy(5)) wired in data; 3 integration tests: first-hit grants, second-hit blocked, next-round resets.

**T03 — Garurumon, Kabuterimon, Kyubimon:** Added damage_tag field to OnDamageDealt (closing T02's tag-specificity debt; collateral fix to 4 test files constructing the event). Implemented tag-specific matching via the new field. Added SelfAdvance(i32) Effect variant; in apply_effects emits TurnAdvance{target:source,amount_pct} targeting the attacker (not defender), reusing the existing apply_turn_advance_system without any new consumer. OnStatusApplied trigger uses std::mem::discriminant comparison — inner field values (e.g. speed_reduction) are irrelevant to the trigger match. Wired Garurumon (Ice/GrantEnergy(5)), Kabuterimon (Electric/GrantEnergy(5)), Kyubimon (OnStatusApplied(Freeze)/SelfAdvance(20)). 4 more tests including cross-contamination negative (Greymon Fire trigger does not fire on Garurumon Ice hit).

**T04 — DORUgamon, Angemon:** Added BonusToughnessDamage(i32) and BonusDamageVsAttribute{attribute, bonus_pct} to Effect enum — parsed and round-trip tested but not used in S08 activations (reserved S09 rebalance). Simpler 'fire a separate skill' approach used for both: dorugamon_form_identity is ToughnessHit(10), angemon_form_identity is Damage(15 Light). OnFirstSkillCastWithTag matches OnDamageDealt{amount>0, damage_tag==tag} from owner (same pattern as OnFirstHitVsTagThisRound; skill-vs-basic distinction not enforced since all DORUgamon attacks share Dark tag). OnAttackVsAttribute uses a per-invocation HashMap<UnitId, Attribute> built from FormIdentityRosterQuery. Angemon negative test (Data target) placed in a fresh app instance to avoid stale OnDamageDealt events from prior resolve_follow_up_action_system execution re-triggering the listener. Skills catalog reaches 65; all 6 form_identity IDs in parse_canonical_skills_ron MVP list.

## Verification

1. `cargo check` — clean (warnings only, pre-existing). 2. `grep -rn 'OneHopSuppressed' src/ tests/` — 0 matches. 3. `grep -n 'follow_up_depth >= 1' src/combat/ultimate.rs` — line 77 preserved. 4. `cargo test --test follow_up_chains` — 2/2 green. 5. `cargo test --test form_identity` — 10/10 green (all 6 Adults: greymon_first_fire_hit_grants_energy, greymon_second_fire_hit_blocked, greymon_resets_next_turn, garurumon_first_ice_hit_grants_energy, kabuterimon_first_electric_hit_grants_energy, kyubimon_freeze_application_self_advances, greymon_fire_trigger_does_not_fire_on_garurumon_ice_hit, dorugamon_first_dark_skill_grants_bonus_toughness, angemon_attack_vs_virus_grants_bonus, angemon_attack_vs_data_no_bonus). 6. `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` — 65 skills, all 6 form_identity IDs present. 7. `grep -c 'form_identity:' assets/data/units.ron` — 6. 8. `cargo test` full suite — 29 binaries, 0 failures.

## Requirements Advanced

None.

## Requirements Validated

- R080 — 10 integration tests in tests/form_identity.rs cover all 6 MVP Adults. Each trigger is once-per-round gated (form_identity_used flag). Full cargo test suite green (29 binaries, 0 failures). All configurations in RON; no hardcoded Adult logic in engine.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

1. FormIdentityKit is a separate ECS component rather than extending UnitSkills (plan suggested extending UnitSkills). Rationale: 25+ construction sites would need updating. 2. GrantEnergy wired in step_app with a separate Query<&mut Energy>, not inside apply_effects (plan suggested unified query). Rationale: preserves 15 resolution_tests callsites unchanged. 3. BonusToughnessDamage/BonusDamageVsAttribute added to schema but unused in S08 skill activations (plan's simpler 'fire a separate skill' approach selected). 4. DORUgamon Light-skill negative test omitted — no Light skills in DORUgamon's kit; once-per-round implicitly covered by second-cast guard. T01: depth increment to origin+1 not explicit in plan but required to achieve D046 observability (depth-2+ events in JSONL).

## Known Limitations

Stale-event re-triggering: if form_identity_used is externally reset to false between updates without a drain, leftover OnDamageDealt events from a prior resolve_follow_up_action_system execution can re-trigger the form_identity listener. Does not affect real gameplay (advance_turn_system resets in a separate update). Test authors must use fresh app instances when testing negative conditions across simulated rounds. BonusToughnessDamage (modifier-in-place for DORUgamon) and BonusDamageVsAttribute (modifier-in-place for Angemon) deferred to S09 rebalance.

## Follow-ups

S09 rebalance should evaluate whether BonusToughnessDamage (modifier-in-place) and BonusDamageVsAttribute (modifier-in-place) deliver better numerical feel than the current separate-skill approach. Product owner UAT in S09 should explicitly sign off on DORUgamon and Angemon deviations.

## Files Created/Modified

None.
