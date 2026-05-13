# S03: Heated + Chilled — damage amp% pipeline + DoT + speed mod — UAT

**Milestone:** M017
**Written:** 2026-05-13T09:19:20.689Z

# UAT — S03: Heated + Chilled damage amp% pipeline + DoT + speed mod

## UAT Type
Integration-test (headless, deterministic)

## Preconditions
- `cargo test` is green on the branch
- `tests/status_amp_pipeline.rs` exists
- No `windowed` feature required

## Test Cases

### 1. Fire base=100 on non-Heated defender → damage 100
**Steps:**
1. Spawn attacker (Vaccine) + defender (no Heated status), Fire-tagged skill base=100, neutral attribute match.
2. Run resolve_action_system.
3. Inspect OnDamageDealt event stream.

**Expected:** `OnDamageDealt.amount == 100` (no amp applied).

---

### 2. Fire base=100 on Heated defender → damage 115
**Steps:**
1. Spawn attacker + defender; apply `StatusEffectKind::Heated` to defender via `StatusBag::apply`.
2. Run resolve_action_system with Fire-tagged skill base=100.
3. Inspect OnDamageDealt event stream.

**Expected:** `OnDamageDealt.amount == 115` (×1.15 amp).

---

### 3. Ice base=100 on Chilled defender → damage 115
**Steps:**
1. Spawn attacker + defender; apply `StatusEffectKind::Chilled` to defender.
2. Run resolve_action_system with Ice-tagged skill base=100.
3. Inspect OnDamageDealt event stream.

**Expected:** `OnDamageDealt.amount == 115`.

---

### 4. Heated unit takes its turn → DoT 4 HP Fire event emitted
**Steps:**
1. Spawn unit with Heated in its StatusBag, HP=100.
2. Write `TurnAdvanced::of(unit_id)` into the message queue.
3. Run advance_turn_system.
4. Inspect event stream and unit HP.

**Expected:** Event stream contains `OnDamageDealt { amount: 4, damage_tag: Fire, .. }` attributed to that unit; unit HP == 96.

---

### 5. Chilled unit AV gain is reduced by 20% vs control (implicit in turn-order behaviour)
**Steps:** (manual / not currently a dedicated test case)
1. Spawn two identical-speed units; apply Chilled to one.
2. Advance several turns.
3. Observe the Chilled unit's AV ticks slower.

**Expected:** Chilled unit accumulates AV at `base_speed × 0.8` relative to the control unit.

---

## Edge Cases
- Heated+Stunned: DoT fires unconditionally before the stun-skip; stun-skipped unit still loses 4 HP.
- HP exactly 4 with Heated: HP drops to 0, OnKO emitted, no double-kill.
- Wrong tag: Heated + Ice skill → `status_amp_pct` returns 100 (no amp).
- Chilled + Fire skill → `status_amp_pct` returns 100 (no amp).

## Not Proven By This UAT
- Paralyzed skip-turn logic (S04).
- Slowed delay-on-apply (S04).
- Blessed buff-dealt / Ult-charge / cleanse-immune (S05).
- JSONL canon naming and ValidationSnapshot canon emission (S06).
- Stack-aware status (deferred post-M017).
