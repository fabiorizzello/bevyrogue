---
id: T02
parent: S02
milestone: M011
key_files:
  - src/combat/types.rs
  - src/combat/unit.rs
  - src/combat/damage.rs
  - src/combat/damage_tests.rs
  - src/combat/toughness.rs
  - src/data/units_ron.rs
  - assets/data/units.ron
  - src/combat/resolution.rs
  - src/combat/bootstrap.rs
  - tests/triangle_matchup.rs
  - tests/follow_up_triggers.rs
key_decisions:
  - calculate_damage takes weaknesses: &[DamageTag] param (not embedded in Unit) — weaknesses live on Toughness so must be passed at call site from defender_tough.weaknesses
  - dmg_modifier is a single outgoing multiplier (not split dmg_in/dmg_out per MEM022 framing) — documented in triangle_modifiers docstring to prevent future confusion
  - No clamp on multiplicative result — discrete mod set is naturally bounded without 0.25/2.5 floor/cap
  - HP expectation in follow_up_triggers updated to 49 from 60 (formula change, documented in narrative)
duration: 
verification_result: passed
completed_at: 2026-04-27T13:57:38.549Z
blocker_discovered: false
---

# T02: Replace Resistances([i8;6]) with Vec&lt;DamageTag&gt; resists and rewrite calculate_damage to v5.3 multiplicative model with triangle_modifiers

**Replace Resistances([i8;6]) with Vec&lt;DamageTag&gt; resists and rewrite calculate_damage to v5.3 multiplicative model with triangle_modifiers**

## What Happened

Removed the `Resistances([i8;6])` struct entirely from `types.rs`. Added `pub resists: Vec<DamageTag>` to `Unit` (component) and `UnitDef` (RON schema) — both now carry an explicit resist list instead of a fixed-width array indexed by enum ordinal.

Rewrote `calculate_damage` in `damage.rs` with the v5.3 multiplicative model:
- `tag_mod = 1.25` if defender's toughness weaknesses contain the attack tag, `0.75` if defender's `resists` contains it, `1.0` otherwise.
- `triangle_modifiers(att_attr, def_attr) → TriangleMods { dmg_modifier, tough_modifier, status_acc_modifier }`: Vaccine > Virus > Data > Vaccine cycle; Free neutral to all. Attacker wins → dmg_modifier=1.11, status_acc=1.1; defender wins → dmg_modifier=0.87, status_acc=0.9; neutral → 1.0. Convention documented in function docstring: `dmg_modifier` is applied outgoing (single multiplier per attack, not split dmg_in/dmg_out per MEM022 framing).
- `calculate_damage` now takes an additional `weaknesses: &[DamageTag]` param since weaknesses live on `Toughness`, not `Unit`.
- Formula: `round(base × tag_mod × tri_mod × (2.0 if is_break else 1.0))`. No clamp — the discrete modifier set is naturally bounded.

Updated `toughness.rs::classify` signature from `target_resists: &Resistances` to `resists: &[DamageTag]`; classify now checks `resists.contains(&attack_tag)` directly.

Updated all call sites: `resolution.rs` passes `&defender_tough.weaknesses` to `calculate_damage` and `defender_unit.resists.as_slice()` to `classify`; `bootstrap.rs` spawns units with `resists: def.resists.clone()`; `headless.rs` logs `u.resists`.

Updated `assets/data/units.ron` (all 12 units): replaced `resistances: Resistances((0,0,0,0,0,0))` with `resists: []`.

Rewrote `damage_tests.rs` with 18-cell matrix (3 tag-buckets × 3 triangle-buckets × 2 break states) plus 4 edge cases. All expected values computed from the new formula and documented in comments.

Created `tests/triangle_matchup.rs` — parametric test enumerating all 16 (attacker_attr, defender_attr) pairs and asserting the full TriangleMods triple.

Updated HP expectation in `follow_up_triggers.rs::s10_agumon_break_follow_up_uses_real_pilot_config`: old value was 60 (two hits at 1.0× each: 18+22=40); new value is 49 (root hit Fire vs weak=1.25 → 22.5→23; follow-up 22×1.25→27.5→28; 100-23-28=49).

## Verification

Ran full test suite: all 26 test targets pass, 0 failures. Specific checks: `cargo test --test triangle_matchup` (1 test, all 16 pairs OK), `cargo test` covers damage_tests inline (18+4 edge cases). Verified no `Resistances`/`resistances` references remain in src/, tests/, or assets/data/ via grep.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-fail-fast 2>&1 | grep -E 'test result'` | 0 | ✅ pass — all 26 test targets, 0 failed | 8000ms |
| 2 | `cargo test --test triangle_matchup --no-fail-fast` | 0 | ✅ pass — 1/1 (all 16 pairs validated) | 1200ms |
| 3 | `! grep -rn 'Resistances\|resistances' src/ tests/ assets/data/` | 0 | ✅ pass — no Resistances references remain | 200ms |

## Deviations

calculate_damage signature extended with `weaknesses: &[DamageTag]` parameter not explicitly called out in the task plan (plan said 'tag_mod checks defender.toughness.weaknesses' but did not state the signature change). This is the correct minimal adaptation since weaknesses live on Toughness, not Unit. All call sites updated accordingly.

## Known Issues

none

## Files Created/Modified

- `src/combat/types.rs`
- `src/combat/unit.rs`
- `src/combat/damage.rs`
- `src/combat/damage_tests.rs`
- `src/combat/toughness.rs`
- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `src/combat/resolution.rs`
- `src/combat/bootstrap.rs`
- `tests/triangle_matchup.rs`
- `tests/follow_up_triggers.rs`
