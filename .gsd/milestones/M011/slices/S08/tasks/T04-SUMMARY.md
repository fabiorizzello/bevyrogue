---
id: T04
parent: S08
milestone: M011
key_files:
  - src/data/skills_ron.rs
  - src/combat/follow_up.rs
  - assets/data/skills.ron
  - assets/data/units.ron
  - tests/form_identity.rs
key_decisions:
  - BonusToughnessDamage/BonusDamageVsAttribute parsed and round-trip tested but NOT used in S08 skill activations; reserved for S09 rebalance. The simpler 'fire a separate ToughnessHit/Damage skill' approach is used instead, consistent with T02's GrantEnergy pattern.
  - OnFirstSkillCastWithTag(tag) implemented as matching OnDamageDealt { amount > 0, damage_tag == tag } from the owner — same as OnFirstHitVsTagThisRound. Skill-vs-basic distinction is not enforced at engine level in S08; acceptable since all DORUgamon attacks share the Dark tag.
  - OnAttackVsAttribute(attr) uses a per-listener-invocation attribute_map HashMap<UnitId, Attribute> built from the FormIdentityRosterQuery rather than adding Attribute to FormIdentitySnapshot. This avoids touching the snapshot struct used by existing arms.
  - Angemon negative test (Data target) placed in a fresh app instance rather than resetting state mid-test. Stale OnDamageDealt events from round 1's resolve_follow_up_action_system re-triggered the form_identity listener in round 2 (same issue class as MEM029).
duration: 
verification_result: passed
completed_at: 2026-04-28T10:31:54.726Z
blocker_discovered: false
---

# T04: Wired DORUgamon (OnFirstSkillCastWithTag/ToughnessHit) and Angemon (OnAttackVsAttribute/Damage) Form Identities — final 2 Adults, 2 new Effect variants, 65 skills, 6 units wired, all tests green

**Wired DORUgamon (OnFirstSkillCastWithTag/ToughnessHit) and Angemon (OnAttackVsAttribute/Damage) Form Identities — final 2 Adults, 2 new Effect variants, 65 skills, 6 units wired, all tests green**

## What Happened

T04 wires the final two Adults to complete the Form Identity S08 slice.

**DORUgamon (UnitId 16) — OnFirstSkillCastWithTag(Dark):**
Added `Effect::BonusToughnessDamage(i32)` to the Effect enum (imported `Attribute` in skills_ron.rs). Per the task plan's simpler approach, `dorugamon_form_identity` is a separate ToughnessHit(10) skill fired as a follow-up rather than modifying the triggering skill's toughness in-place. `BonusToughnessDamage` is parsed and round-trip tested but unused in S08 activation (reserved for S09 rebalance). units.ron DORUgamon entry gained `form_identity: Some((trigger: OnFirstSkillCastWithTag(Dark), action: SkillId("dorugamon_form_identity")))`.

**Angemon (UnitId 17) — OnAttackVsAttribute(Virus):**
Added `Effect::BonusDamageVsAttribute { attribute: Attribute, bonus_pct: i32 }`. Same simpler approach: `angemon_form_identity` is a separate Damage(15 Light) skill fired when Angemon attacks a Virus target. `BonusDamageVsAttribute` parsed but unused in S08. units.ron Angemon entry gained `form_identity: Some((trigger: OnAttackVsAttribute(Virus), action: SkillId("angemon_form_identity")))`.

**Listener extension (`src/combat/follow_up.rs`):**
`evaluate_form_identity_trigger` gained two new arms: `OnFirstSkillCastWithTag(tag)` matches `OnDamageDealt { amount > 0, damage_tag == tag }` from the owner (same pattern as `OnFirstHitVsTagThisRound`; skill-vs-basic distinction not enforced at engine level in S08 since all DORUgamon attacks share the Dark tag); `OnAttackVsAttribute(attr)` matches `OnDamageDealt { amount > 0 }` from the owner against a target whose attribute equals `attr`, looked up via a new `attribute_map: HashMap<UnitId, Attribute>` built from the FormIdentityRosterQuery. Added `Attribute` to the types import.

**Tests:** Four new tests in tests/form_identity.rs:
- `dorugamon_first_dark_skill_grants_bonus_toughness`: DORUgamon casts power_metal (Dark), asserts `form_identity_used = true` and toughness reduced by 18 + 10 = 172 (vs 200). Also asserts once-per-round guard: second cast (cannonball) reduces only by 20, not +10 again.
- `angemon_attack_vs_virus_grants_bonus`: Angemon attacks Virus enemy, asserts `form_identity_used = true` and at least 2 `OnDamageDealt` events from Angemon (base + follow-up).
- `angemon_attack_vs_data_no_bonus` (separate fresh app): Angemon attacks Data enemy, asserts `form_identity_used = false`.
- Helper functions added: `toughness_current`, `spawn_enemy_with_attribute`.

**Stale-event gotcha:** The negative Angemon test was initially combined with the positive in one app. After round 1, leftover `OnDamageDealt` events from `resolve_follow_up_action_system` (the form_identity follow-up skill's execution) were re-read by `form_identity_listener_system` in round 2 (before those events had been consumed by any system). Splitting into two separate test functions with independent app instances eliminated the issue. This is the same class of bug as MEM029 (message buffer drain between updates).

**Slice closure:** `parse_canonical_skills_ron` updated to 65, all 6 form_identity IDs in MVP list. `grep -c 'form_identity:' units.ron` returns 6. Full cargo test suite: all binaries green.

## Verification

1. `cargo check` — clean (warnings only, no errors). 2. `cargo test --test form_identity dorugamon_first_dark_skill_grants_bonus_toughness` — ok. 3. `cargo test --test form_identity angemon_attack_vs_virus_grants_bonus` — ok. 4. `cargo test --test form_identity angemon_attack_vs_data_no_bonus` — ok. 5. `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` — ok (65 skills, all 6 form_identity IDs). 6. `grep -c 'form_identity:' assets/data/units.ron` — 6. 7. `cargo test` full suite — all binaries green (0 failures).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | ✅ pass | 130ms |
| 2 | `cargo test --test form_identity dorugamon_first_dark_skill_grants_bonus_toughness` | 0 | ✅ pass | 280ms |
| 3 | `cargo test --test form_identity angemon_attack_vs_virus_grants_bonus` | 0 | ✅ pass | 560ms |
| 4 | `cargo test --test form_identity angemon_attack_vs_data_no_bonus` | 0 | ✅ pass | 560ms |
| 5 | `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` | 0 | ✅ pass (65 skills) | 130ms |
| 6 | `grep -c 'form_identity:' assets/data/units.ron` | 0 | ✅ pass (6) | 10ms |
| 7 | `cargo test` | 0 | ✅ pass (all binaries green) | 15000ms |

## Deviations

1. BonusToughnessDamage and BonusDamageVsAttribute are added to the Effect enum and round-trip tested but are NOT wired into any active skill in S08. DORUgamon uses ToughnessHit(10) and Angemon uses Damage(15) in their respective form_identity skills. The 'modifier applied to the triggering skill' design is deferred to S09 as stated in the task plan. 2. The DORUgamon negative test ('does not fire on a Light skill cast') was not implemented as a standalone test because DORUgamon has no Light skills in its RON kit. The once-per-round guard is verified implicitly via the second-cast assertion in dorugamon_first_dark_skill_grants_bonus_toughness. Product owner should validate in S09 UAT.

## Known Issues

Stale-event re-triggering: if `form_identity_used` is externally reset to false between app updates without a drain update, leftover OnDamageDealt events from a prior resolve_follow_up_action_system execution can re-trigger the form_identity listener. This does not affect real gameplay (advance_turn_system resets the flag in a separate update), but is a test-authoring gotcha documented in the task summary.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/follow_up.rs`
- `assets/data/skills.ron`
- `assets/data/units.ron`
- `tests/form_identity.rs`
