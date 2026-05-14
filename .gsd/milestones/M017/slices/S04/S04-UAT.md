# S04: Paralyzed + Slowed — turn skip + delay-on-apply — UAT

**Milestone:** M017
**Written:** 2026-05-13T09:41:42.287Z

## UAT: S04 — Paralyzed + Slowed

### UAT Type
Automated integration tests (deterministic, headless, seeded)

### Preconditions
- `cargo check` exits 0 (no compilation errors)
- `cargo test` exits 0 before this slice (established by S01–S03)

### Test Cases

**TC-1: Paralyzed — 100-turn deterministic skip loop**
1. Spawn 1 ally + 1 enemy via test bootstrap
2. Apply `StatusEffectKind::Paralyzed { duration: 100 }` to the enemy
3. Drive 100 `TurnAdvanced` cycles for the enemy; run `app.update()` each cycle
4. Collect all `CombatEvent` and `ActionIntent` entities via `get_cursor_current()`-anchored cursor
- **Expected:** Exactly 100 `CombatEventKind::OnActionFailed { reason: "paralyzed" }` events emitted
- **Expected:** Zero `ActionIntent` components written for the enemy across all 100 cycles
- **Covered by:** `tests/status_paralyzed_skip.rs` — 1/1 pass ✓

**TC-2: Slowed — first-apply pushes AV and emits exactly one TurnAdvance**
1. Spawn attacker + defender (ActionValue=5000, no TempoResistance, StatusBag)
2. Apply Slowed via skill-resolution path (pure-ApplyStatus SkillDef, CombatRng::from_seed(0))
3. Chain `resolve_action_system` → `apply_turn_advance_system`
4. Read CombatEvents for this frame
- **Expected:** Exactly one `CombatEventKind::TurnAdvance { target: defender_id, amount_pct: -30 }` emitted
- **Expected:** `OnStatusApplied` precedes `TurnAdvance` in event stream (JSONL log order)
- **Expected:** Defender `ActionValue` = 2000 (5000 − 3000; 3000 = 30% of MAX_AV=10000, no TempoResistance)
- **Covered by:** `tests/status_slowed_delay.rs` — 1/1 pass ✓

**TC-3: Slowed — re-apply does NOT re-push AV**
1. Continue from TC-2 (defender already has Slowed in StatusBag)
2. Apply Slowed again via same skill-resolution path
3. Run `resolve_action_system` → `apply_turn_advance_system`
4. Read CombatEvents for this frame
- **Expected:** Zero `TurnAdvance` events emitted (refresh_max_dur path only)
- **Expected:** Defender AV unchanged from previous value
- **Covered by:** `tests/status_slowed_delay.rs` (second-apply assertion) ✓

**TC-4: Regression — existing test suites unchanged**
1. Run full `cargo test`
- **Expected:** status_amp_pipeline, combat_coherence, follow_up_chains, form_identity, validation_snapshot, ultimate_meter all pass
- **Covered by:** T05 full-suite run, exit:0, 0 failures ✓

**TC-5: Grep guard — no new legacy taxonomy violations**
1. Run `grep -rn -E 'Burn|Freeze|Shock|DeepFreeze' src/ tests/ | grep -v 'reserved'`
- **Expected:** All hits are pre-existing canonical uses in exempted files (status_effect.rs, skills_ron.rs) or legacy compound identifiers (ShockTransfer, MissingPreExistingShock); S04 introduced zero new occurrences
- **Covered by:** T05 grep guard, 11 pre-existing hits ✓

### Edge Cases
- Paralyzed duration tick on last turn (dur=1→0): skip still fires because `is_paralyzed` captured pre-tick — verified by TC-1 counting 100 skips across exactly 100 turns with dur=100
- Slowed re-apply guard uses pre-mutation bag state (`is_first_apply_slowed` computed before `bag.apply()`) — verified by TC-3

### Not Proven By This UAT
- Slowed interaction with non-zero TempoResistance (only tested with zero resistance; TempoResistance integration is routed through `resistance::apply_av_change` but not exercised here)
- Paralyzed stacking or interaction with simultaneous Stunned (separate hard-control semantics)
- AV clamp at 0 or ±50% cap (delegated to M018)
- Paralyzed applied mid-combat via skill resolution (only tested via direct StatusBag insert at construction)
