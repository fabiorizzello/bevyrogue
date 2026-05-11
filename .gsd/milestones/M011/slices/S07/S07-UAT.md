# S07: Toughness 3 categorie (Standard/Armored/Shielded) + Break Seal — UAT

**Milestone:** M011
**Written:** 2026-04-28T09:16:37.283Z

# S07 UAT — Toughness 3 Categories + Break Seal

## Preconditions
- `cargo test` full suite passes (33 test groups, 0 failures)
- `tests/toughness_categories.rs` present with 4 tests
- Devimon (id 101) has `toughness_category: Armored` in `assets/data/units.ron`
- All other units default to `Standard` (no explicit field or `Standard` set)

## Test Cases

### TC-01: Standard enemy breaks in one full toughness hit
**Test:** `standard_breaks_in_one_full_hit`
1. Spawn Standard enemy with toughness_max=20 and Fire weakness
2. Fire one ToughnessHit(20) via a Fire-tagged skill
3. **Expected:** Exactly 1 `CombatEventKind::OnBreak` emitted; `Toughness.broken == true`
4. **Failure signal:** If 0 OnBreak, Standard category semantics are broken

### TC-02: Armored enemy requires two full hits to break
**Test:** `armored_requires_two_full_hits`
1. Spawn Armored enemy with toughness_max=20 and Fire weakness
2. Fire first ToughnessHit(20)
3. **Expected after 1st hit:** 0 OnBreak; `Toughness.current == 10` (ceiling halving: (20+1)/2 = 10)
4. Fire second ToughnessHit(20)
5. **Expected after 2nd hit:** 1 OnBreak; `Toughness.broken == true`
6. **Failure signal:** If OnBreak fires on first hit, Armored halving is not applied

### TC-03: Shielded enemy never breaks regardless of hit count
**Test:** `shielded_never_breaks`
1. Spawn Shielded enemy with toughness_max=20 and Fire weakness
2. Fire three ToughnessHit(20) in sequence
3. **Expected after all hits:** 0 OnBreak; `Toughness.broken == false`; `Toughness.current == 0` (floor-clamped, not preserved at max)
4. **Failure signal:** Any OnBreak event means Shielded guard is missing

### TC-04: Break Seal lifecycle — set on break, blocks same-round re-break, lifts on TurnAdvanced
**Test:** `break_seal_blocks_repeat_break_in_same_round_then_lifts_on_next_turn`

**Step 1 — Initial break:**
1. Spawn Standard enemy with toughness_max=20 and Fire weakness
2. Fire ToughnessHit(20)
3. **Expected:** 1 OnBreak; `RoundFlags.break_sealed == true`

**Step 2 — Sealed re-break attempt (same round):**
4. Restore enemy toughness (mutate component to max, broken=false) simulating a second attempt within the same round
5. Fire another ToughnessHit(20)
6. **Expected:** 0 new OnBreak; `RoundFlags.break_sealed` remains true; `Toughness.current` unchanged (apply_hit short-circuits)

**Step 3 — Seal lifted on TurnAdvanced:**
7. Send `TurnAdvanced` event for the defender; run one `app.update()`
8. **Expected:** `RoundFlags.break_sealed == false`

**Step 4 — Break possible again after seal reset:**
9. Fire another ToughnessHit(20)
10. **Expected:** 1 OnBreak; `Toughness.broken == true`

### TC-05: RON round-trip — Devimon loads as Armored
**Manual / CLI check:**
1. `cargo run --bin combat_cli` (post-S04 binary)
2. Select a party including Devimon's side (enemy spawn)
3. Inspect loaded unit state or JSONL log
4. **Expected:** Devimon (id 101) shows `toughness_category: Armored` in the spawned Toughness component

### TC-06: Non-breaking units default to Standard
**Check via code review / test coverage:**
- All UnitDef entries in `units.ron` without explicit `toughness_category` field should default to Standard
- Verify `cargo test --test roster_smoke` passes (exercises all unit defs via spawn)

## Edge Cases Verified
- `break_sealed=true` short-circuits before category dispatch — noop for all three categories
- Armored ceiling division: 1 raw → 1 effective (no zero-damage infinite loop)
- Shielded current floors at 0, not at toughness_max — `current` drains but never crosses into broken territory
- Stunned unit (from break) does not generate spurious ActionIntent during advance_turn_system seal-reset update
