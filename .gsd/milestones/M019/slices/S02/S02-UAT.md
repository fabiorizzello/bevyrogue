# S02: Effect::Heal { amount_pct_max_hp } primitive — UAT

**Milestone:** M019
**Written:** 2026-05-14T08:51:19.235Z

# UAT: S02 — Effect::Heal primitive

**UAT Type:** Contract verification (integration tests, headless, deterministic)

## Preconditions
- Rust toolchain installed per `rust-toolchain.toml`
- No modifications to `assets/data/units.ron`, `assets/data/skills.ron`, or `assets/data/party.ron` (baseline fixtures unchanged)
- Working directory: `/home/fabio/dev/bevyrogue`

## Test Cases

### 1. Single heal on damaged ally
**Steps:**
1. Construct a unit with `hp_max=100`, `hp_current=60`
2. Call `apply_effects` with `Effect::Heal { amount_pct_max_hp: 30, target: TargetShape::Single }` on that unit
**Expected:** `hp_current` becomes 90 (floor(100×30/100)=30, capped to hp_max-hp_current=40, so 30 applied). `OnHealed { amount: 30, hp_after: 90 }` emitted. `sp_ok=true`.

### 2. Single heal at full HP emits zero-amount event
**Steps:**
1. Construct a unit with `hp_max=100`, `hp_current=100`
2. Apply `Effect::Heal { amount_pct_max_hp: 50, target: TargetShape::Single }`
**Expected:** `hp_current` remains 100. `OnHealed { amount: 0, hp_after: 100 }` still emitted. `sp_ok=true`.

### 3. Single heal on KO target is a no-op
**Steps:**
1. Construct a unit with `hp_current=0` (KO)
2. Apply `Effect::Heal { amount_pct_max_hp: 50, target: TargetShape::Single }`
**Expected:** `hp_current` remains 0. No `OnHealed` event emitted. `sp_ok=true` (no SP consumed).

### 4. AllAllies fan-out — KO skipped, alive healed in slot order
**Steps:**
1. Construct 3 units on the same team: slot 0 alive (hp_max=100, hp_current=70), slot 1 KO (hp_current=0), slot 2 alive (hp_max=80, hp_current=40)
2. Apply `Effect::Heal { amount_pct_max_hp: 20, target: TargetShape::AllAllies }` via pipeline fan-out
**Expected:** Slot 0 receives `OnHealed { amount: 20, hp_after: 90 }`. Slot 1 receives no event and no state change. Slot 2 receives `OnHealed { amount: 16, hp_after: 56 }` (floor(80×20/100)=16). Events ordered slot 0 before slot 2.

### 5. Heal cap — over-heal clamped to hp_max
**Steps:**
1. Construct a unit with `hp_max=100`, `hp_current=97`
2. Apply `Effect::Heal { amount_pct_max_hp: 50, target: TargetShape::Single }`
**Expected:** `healed` = min(50, 3) = 3. `hp_current` becomes 100. `OnHealed { amount: 3, hp_after: 100 }` emitted.

## Edge Cases Covered
- KO target: no state mutation, no event, sp_ok=true
- Full HP target: zero-amount event still emitted (event completeness)
- AllAllies with heterogeneous team: KO members skipped, alive sorted by slot_index ascending
- Cap: excess heal silently clamped, hp_current never exceeds hp_max

## Not Proven By This UAT
- JSONL stream OnHealed entries during a live Bevy world run (only tested via apply_effects direct-call; JSONL wiring relies on existing CombatEvent bus and serde::Serialize derive — no new serialization code introduced)
- Heal triggered by full turn cycle (system-level integration via Bevy world not exercised; T03 covers contract-level only)
- ATK-scaling Heal (deferred to M021)
- Interaction of Heal with status effects (e.g. healing received reduction) — not yet implemented

