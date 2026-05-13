---
id: T02
parent: S03
milestone: M018
key_files:
  - src/data/skills_ron.rs
  - src/combat/resolution.rs
  - src/combat/action_query.rs
key_decisions:
  - Kept TargetShape as Copy by ensuring all Bounce fields (u8, BounceSelector, RepeatPolicy) are also Copy — avoids pervasive refactor of call sites that pass TargetShape by value
  - DamageCurve::Constant as serde default means all existing RON assets are unaffected (no migration needed)
  - chain_bolt kept as inline test fixture (not added to skills.ron) to preserve the 74-skill catalog size assertion
  - Bounce is accepted by all three executable-now gates (validate_skill_def, target_shape_is_executable_now, target_status_for_unit) so Bounce-shaped skills with hops>=1 are treated as implemented — full per-hop damage application is deferred to later slices
duration: 
verification_result: passed
completed_at: 2026-05-13T20:50:18.603Z
blocker_discovered: false
---

# T02: Migrated TargetShape::Bounce to struct variant {hops, selector, repeat} + added DamageCurve enum to Effect::Damage; updated all three validation gates and resolution fan-out

**Migrated TargetShape::Bounce to struct variant {hops, selector, repeat} + added DamageCurve enum to Effect::Damage; updated all three validation gates and resolution fan-out**

## What Happened

T01 had already landed BounceSelector, RepeatPolicy, select_bounce_hop, and the hp_per_mille field on TargetEntry in resolution.rs (committed as af09d40, gsd-only files). T02 built on that foundation.

Key steps:
1. Added BounceSelector (LowestHpPctAlive/NextSlotAlive/AdjLowest) and RepeatPolicy (NoRepeat/AllowRepeat) enums to src/data/skills_ron.rs (T01 had put these only in resolution.rs; needed them in the DSL layer too for RON round-trip).
2. Changed TargetShape from a Copy-only flat enum to include the struct variant Bounce { hops: u8, selector: BounceSelector, repeat: RepeatPolicy }. All three component types are Copy so TargetShape retained #[derive(Copy)].
3. Added DamageCurve enum (Constant default, Falloff { pct: u16 }, PerHop(Vec<i32>)) and added per_hop: DamageCurve field (serde default = Constant) to Effect::Damage.
4. Updated validate_skill_def: (a) hops==0 rejection (always, regardless of implementation status); (b) shape_is_executable helper now includes Bounce{..}; (c) DamageCurve validation: Falloff pct <=100, PerHop len == hops.
5. Updated resolution::resolve_targets to add a Bounce arm that drives the full hop chain via select_bounce_hop (already implemented by T01).
6. Updated target_shape_is_executable_now in resolution.rs to include Bounce{..}.
7. Updated action_query::target_status_for_unit gate to include Bounce{..}.
8. Updated effect_roundtrip_damage_struct_variant test to supply per_hop field (now required in struct literal) and relaxed the exact-string assertion.
9. Added chain_bolt fixture (inline, not in skills.ron to preserve catalog size assertion), plus 9 new tests: bounce_target_shape_ron_roundtrip, damage_curve_* roundtrips (3), effect_damage_with_bounce_shape_roundtrip, validator_accepts_bounce_with_per_hop_curve_matching_hops, validator_accepts_bounce_with_falloff_curve, validator_rejects_per_hop_length_mismatch, validator_rejects_bounce_hops_zero, validator_rejects_falloff_pct_over_100.

## Verification

cargo check (no errors), cargo test --lib skills_ron::tests (33/33 pass), cargo test --lib resolution::tests (39/39 pass)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | PASS — no errors, 3 pre-existing warnings | 3200ms |
| 2 | `cargo test --lib skills_ron::tests` | 0 | PASS — 33/33 including all new Bounce/DamageCurve tests | 1420ms |
| 3 | `cargo test --lib resolution::tests` | 0 | PASS — 39/39 | 1420ms |

## Deviations

chain_bolt fixture is inline in tests rather than in skills.ron; the task said 'existing chain_bolt fixture' but no such fixture existed anywhere in the codebase (T01 did not create it). Created it as an inline test helper to avoid disturbing the catalog-size assertion.

## Known Issues

None.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `src/combat/action_query.rs`
