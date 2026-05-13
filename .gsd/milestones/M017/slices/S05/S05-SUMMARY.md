---
id: S05
parent: M017
milestone: M017
provides:
  - Blessed ×1.15 attacker damage multiplier wired in apply_effects/calculate_damage pipeline
  - Blessed +1 Ult charge per non-Reset action wired in apply_effects
  - Cleanse-immunity regression guard for Blessed (integration test)
  - attacker_statuses: Option<&StatusBag> parameter threaded through apply_effects and both pipeline call sites
requires:
  []
affects:
  - S06
key_files:
  - src/combat/damage.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - tests/status_blessed_cleanse_immune.rs
  - tests/status_blessed_offensive.rs
  - tests/status_blessed_ult_charge.rs
  - tests/resolution_tests.rs
  - tests/holy_support_resolution.rs
key_decisions:
  - attacker_dmg_mult is a plain f32 parameter on calculate_damage (not folded into AttackContext) to keep AttackContext as a pure attack descriptor and avoid breaking test helpers.
  - Blessed ×1.15 multiplier computed in apply_effects — co-located with other status-driven modifiers — rather than inside calculate_damage where it would have no status context.
  - Blessed +1 Ult charge skips the Reset branch per §H.1 to prevent the firing Ultimate from self-feeding its own charge meter.
  - Blessed +1 is gated on outcome.succeeded so aborted or failed actions don't grant the bonus.
  - None inserted at all non-pipeline call sites (tests) mechanically — zero semantic change at those sites.
patterns_established:
  - Attacker-side status multipliers are computed in apply_effects by inspecting Optional<&StatusBag>, then forwarded as a plain scalar into calculate_damage — keeping calculate_damage status-agnostic.
  - Reset-branch guard for Ult self-charge: always check ult_effect != UltEffect::Reset before granting per-action Ult charge bonuses.
observability_surfaces:
  - Existing CombatEvent damage payloads naturally reflect the post-Blessed-mult damage value — no new surfaces required. JSONL log/ValidationSnapshot canon naming is owned by S06.
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-13T10:09:38.441Z
blocker_discovered: false
---

# S05: Blessed — buff dealt + Ult charge + cleanse-immune

**Wired Blessed ×1.15 damage-dealt multiplier and +1 Ult charge per action into apply_effects; three integration tests confirm the offensive bonus, the charge bump, and cleanse-immunity.**

## What Happened

S05 closed three remaining Blessed hooks with zero source rewrites outside the targeted files.

**T01 — Cleanse-immune regression guard** added `tests/status_blessed_cleanse_immune.rs` with two deterministic tests: (a) Blessed alone survives `cleanse_debuffs()`, (b) Blessed alongside debuffs — the debuffs are stripped but Blessed remains. No src/ changes were needed because `BuffKind::Buff` classification and the `cleanse_debuffs()` filter were already in place from S02.

**T02 — Blessed ×1.15 damage multiplier** threaded a new `attacker_dmg_mult: f32` parameter into `calculate_damage` in `src/combat/damage.rs` and computed `1.15 if attacker has Blessed else 1.0` inside `apply_effects` in `src/combat/resolution.rs`. The two `apply_effects` call sites in `src/combat/turn_system/pipeline.rs` (~lines 280, 576) were updated to fetch the attacker's `StatusBag` from the existing tuple and pass it through; all other call sites (tests) received `None`. `tests/status_blessed_offensive.rs` (4 tests) asserted `round(base×tag×tri×break×1.15)` for a Blessed attacker vs baseline 1.0× for a non-Blessed or empty-bag attacker; a Heated-but-not-Blessed attacker also confirmed orthogonality.

**T03 — Blessed +1 Ult charge per action** added a post-outcome guard in `apply_effects`: after `match resolved.ult_effect`, if `attacker_statuses` has Blessed AND `ult_effect != UltEffect::Reset` AND `outcome.succeeded`, call `attacker_ult.try_add(1)`. The Reset guard prevents the firing Ultimate from charging itself mid-cast (§H.1). `tests/status_blessed_ult_charge.rs` (3 tests) covered: baseline no-Blessed Basic (delta 0), Blessed Basic (delta +1 over baseline), and Blessed Ultimate cast (no +1 leak post-reset). A deviation from the plan: `holy_support_resolution.rs` also required a mechanical `None` argument added — not listed in the task plan but required to compile.

All three tasks verified green in isolation and as part of the full `cargo test` suite (0 failures across all binaries).

## Verification

cargo check: clean (warnings only, no errors). cargo test --test status_blessed_cleanse_immune: 2/2 pass. cargo test --test status_blessed_offensive: 4/4 pass. cargo test --test status_blessed_ult_charge: 3/3 pass. Full cargo test suite: all test binaries pass, 0 failed.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

["holy_support_resolution.rs required a mechanical None argument for the new apply_effects signature — not listed in the T02 task plan but required to compile. No semantic change."]

## Known Limitations

["JSONL log and ValidationSnapshot canon naming for Blessed events is delegated to S06."]

## Follow-ups

None.

## Files Created/Modified

None.
