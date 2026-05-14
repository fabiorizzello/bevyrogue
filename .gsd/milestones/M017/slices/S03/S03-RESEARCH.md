# S03 RESEARCH — Heated + Chilled (damage amp% + DoT + speed mod)

## Summary
S02 left in place a complete lifecycle skeleton (`StatusBag`, `StatusInstance`, `StatusEffectKind` including `Heated`/`Chilled`, `refresh_max_dur`, `tick_all` emitting `OnStatusTick`/`OnStatusExpired`) but with **explicit no-op per-status semantics** (see `src/combat/turn_system/mod.rs:474-513` totality match). All three S03 hooks land into already-existing seams — no new components, no new event kinds required. The pipeline lacks an attacker-perspective `defender_status` parameter in `calculate_damage`; that is the only non-trivial surface change.

## Implementation Landscape

| File | Purpose / current state | S03 work |
|------|-------------------------|----------|
| `src/combat/status_effect.rs` | `StatusBag` + `StatusEffectKind` + `tick_all`. Variants `Heated`/`Chilled` already exist. | Add `status_amp_pct(bag, DamageTag) -> i32` lookup helper (returns 115 if `Heated` & Fire, or `Chilled` & Ice; else 100). Pure function, easily unit-tested. |
| `src/combat/damage.rs` | `calculate_damage(attacker, &AttackContext, defender, &weaknesses)` — multiplicative pipeline: `base × tag_mod × tri_mod × break_mod`. No status awareness. | Add `defender_status: Option<&StatusBag>` parameter (or extend `AttackContext` with a precomputed `status_amp_pct: i32`). Apply `× amp_pct/100` as a new multiplicative factor; expose `status_amp_pct` in `DamageBreakdown` for log/observability symmetry with `tag_mod_pct` / `triangle_mod_pct`. |
| `src/combat/resolution.rs` | Site at `:276-285` builds `AttackContext` and calls `calculate_damage`. Has `defender_unit` but no access to defender's `StatusBag`. | Pass defender's `StatusBag` (or precomputed amp%) down. `apply_effects` signature needs `defender_status: Option<&StatusBag>` (immutable; tick still happens in turn_system). |
| `src/combat/turn_system/mod.rs:474-513` | On unit's own turn, iterates `bag.iter()` emitting `OnStatusTick`, then `bag.tick_all()`. The totality match has empty arms — this is the seam for DoT. | Before tick, for any `StatusEffectKind::Heated` instance on the active unit, apply 4 HP damage to its `Unit`, emit `OnDamageDealt` (or reuse via a dedicated `OnStatusTick` payload + an `OnHitTaken`). Decide: a) new `damage_tag: DamageTag::Fire`-tagged `OnDamageDealt` with `amount=4`, or b) keep `OnStatusTick` and add a `dot_damage: Option<i32>` field. Simpler/cheapest: emit existing `OnDamageDealt { amount: 4, damage_tag: Fire, kind: DamageKind::Neutral, tag_mod_pct: 100, triangle_mod_pct: 100 }` — matches "visible in log at turn-end with 4 dmg". |
| `src/combat/speed.rs` | `Speed(i32)` + `SpeedModifier(i32)` components; bootstrap inits `SpeedModifier(0)`. AV gain at `turn_system/mod.rs:570` is `(speed.0 + speed_mod.0) * AV_PER_SPEED`. **Sign convention**: comment says `Speed - SpeedModifier` but code adds; negative `SpeedModifier` slows the unit. | When `Chilled` is present, ensure `SpeedModifier` reflects −20% of base `Speed`. Two seams: (1) lazy/derived read at AV-gain site (`av_gain = (speed.0 + chilled_delta(&bag, speed.0)) * AV_PER_SPEED`) — cleanest, no state mutation; (2) eager: mutate `SpeedModifier` on apply/expire. Recommend (1) — derived read avoids stale `SpeedModifier` when status expires mid-round. Canon says "turno corrente": derive at AV-gain time, no persistence beyond `StatusBag`. |
| `src/combat/events.rs` | `CombatEventKind` already includes `OnStatusTick { kind, turns_left }` and `OnDamageDealt { amount, kind, damage_tag, .. }`. | No new variant needed; reuse `OnDamageDealt` for DoT. |
| `src/combat/bootstrap.rs:152-156` | Spawns units with `Speed`, `SpeedModifier(0)`, etc. | Unchanged. |

## Natural Seams (independent, can be parallelized)

1. **Pure lookup** (`status_amp_pct` in `status_effect.rs`) — zero coupling, easiest first proof.
2. **Damage pipeline hook** — wire `status_amp_pct` into `calculate_damage`; thread `&StatusBag` through `apply_effects` call site; extend `DamageBreakdown`.
3. **Chilled speed** — derived read at `turn_system/mod.rs:570` AV-gain loop. Helper `chilled_speed_delta(bag: &StatusBag, base_speed: i32) -> i32` returns `-base_speed / 5` (=−20%) iff `Chilled` present, else 0. Independent of (2).
4. **Heated DoT** — emit at `turn_system/mod.rs:478` (inside the existing per-instance iter, before `tick_all`); 4 HP loss on `Unit`, push `OnDamageDealt`. Independent of (2) and (3).

## Recommendation (build order + first proof)

1. **T01 (pure)**: `status_amp_pct(&StatusBag, DamageTag) -> i32` + unit tests in `status_effect.rs#tests`. First proof: `cargo test combat::status_effect::tests::status_amp -- --nocapture`.
2. **T02 (pipeline)**: extend `calculate_damage` signature with `defender_status: Option<&StatusBag>`, plumb through `resolution.rs:276-285`, add `status_amp_pct` field to `DamageBreakdown`. First proof: existing `damage_tests.rs` must still pass (defaulting `None`); then new fire-on-Heated test in `damage_tests.rs`.
3. **T03 (DoT)**: emit `OnDamageDealt { amount: 4, damage_tag: Fire, .. }` inside `advance_turn_system` for each `Heated` instance found on `bag.iter()`; mutate `unit.hp_current -= 4`. Guard against KO (skip if already KO).
4. **T04 (Chilled speed)**: helper + apply at AV-gain site in `turn_system/mod.rs:570`. Verify unaffected units unchanged.
5. **T05 (integration test)**: new `tests/status_amp_pipeline.rs` covers DoD scenarios.

## Verification

```bash
cargo check                                       # default headless
cargo test --test status_amp_pipeline             # new integration test
cargo test combat::damage_tests                   # pipeline regression
cargo test combat::status_effect::tests           # lookup unit tests
cargo test --test event_stream                    # OnDamageDealt schema unaffected
cargo test                                        # full suite
```

DoD assertions for `tests/status_amp_pipeline.rs`:
- Fire base=100, defender non-Heated → final 100 (neutral attrs, no weakness).
- Fire base=100, defender Heated → final 115.
- Ice base=100, defender Chilled → final 115.
- After active unit with Heated takes its turn-end tick, `ActionLog`/event-stream contains an `OnDamageDealt { amount: 4, damage_tag: Fire, .. }` for that unit.

## Risks

- **Sign convention for `SpeedModifier`**: doc comment says `Speed - SpeedModifier`, code adds. Use the doc as *intent* but **trust the code path**: a negative `SpeedModifier` slows. The derived-helper approach sidesteps this entirely (returns a negative delta to add to `speed.0`).
- **Multiplicative ordering**: with `tag_mod=1.25` (weakness) + `status_amp=1.15` (Heated) + `tri_mod=1.11` we get compounding. DoD examples assume neutral attrs and no weakness, so 100×1.15=115. Keep multiplicative — do not switch to additive — to stay consistent with the v5.3 model in `damage.rs:73-103`.
- **DoT timing vs. tick_all**: emit the damage event **before** `tick_all` so that a Heated of duration=1 still ticks for damage on the turn it expires (matches "DoT visible at turn-end" canon). The existing iter-then-tick pattern at `turn_system/mod.rs:478-502` already separates these phases.
- **Stunned units**: `turn_system/mod.rs:465-472` short-circuits with `continue` before the status block. Decide canon: does Heated DoT tick on a stunned unit's "turn"? Recommend yes (DoT bypasses stun) — move the Heated-DoT emission **above** the stun early-return, or restructure. Flag for planner.
- **No new event variant**: reusing `OnDamageDealt` for DoT means downstream consumers (follow-up listeners, observability) will see a damage event with no `OnSkillCast` predecessor. Audit `follow_up.rs` listeners to confirm they don't assume "every OnDamageDealt has a preceding cast". Quick grep should suffice.
