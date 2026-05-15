# S02: Timeline FSM + validate_timeline_refs — UAT

**Milestone:** M021
**Written:** 2026-05-15T08:18:35.127Z

# S02 UAT — Timeline FSM + validate_timeline_refs

## UAT Type
Automated integration test — headless, no UI interaction required.

## Preconditions
- Repo on branch `milestone/M021`, all S02 tasks complete.
- `cargo test` exits 0 with the full suite green.

## Test Cases

### UAT-1: Fixture OnTurnStart kills target (Demo Gate 1)

**Steps:**
1. Run `cargo test --test timeline_onturnstart_kills`.

**Expected outcome:**
- Test `fixture_onturnstart_kills_target` passes.
- A hand-built `CompiledTimeline` with an `OnTurnStart` hook drives `BeatRunner::run_to_completion`, which enqueues `Intent::DealDamage` and the S01 `intent_applier` resolves it — target HP reaches 0.

**Edge cases:**
- If `intent_applier` is not registered or the `CombatPlugin` is not loaded, the intent queue is never drained and the test fails with a non-zero HP assertion.

---

### UAT-2: Validator reports dangling-reference typo (Demo Gate 2)

**Steps:**
1. Run `cargo test --test timeline_validate_typo`.

**Expected outcome:**
- Test `validate_timeline_refs_catches_typo_in_hook_id` passes.
- `validate_timeline_refs` returns at least one `TimelineError` naming the axis (`hook`) and the site (the typo'd id string), without panicking or returning Ok.

**Edge cases:**
- A timeline with 0 beats should pass validation (empty graph).
- Multiple errors on a single timeline are all collected before returning.

---

### UAT-3: chain_bolt 3-hop Loop timeline produces correct falloff (Demo Gate 3)

**Steps:**
1. Run `cargo test --test timeline_chain_bolt_port`.

**Expected outcome:**
- Test `chain_bolt_hits_3_targets_with_falloff` passes.
- Three `Intent::DealDamage` values are emitted in CHAIN_ORDER target order with damage values following a 0.8^n ladder (100 → 80 → 64, or equivalent integer representation).
- `cast_hit_set` NoRepeat enforcement prevents any target from being hit twice.

**Edge cases:**
- If `exit_when` logic is absent, the loop runs indefinitely (guarded by the circuit-breaker in `BeatRunner`).
- With fewer than 3 live targets, the loop must exit early without panicking.

---

### UAT-4: CombatPlugin::finish validator hook wired

**Steps:**
1. Run `cargo test` (full suite).
2. Inspect: `rg 'fn finish' src/combat/plugin.rs` → 1 match.

**Expected outcome:**
- `CombatPlugin` implements `Plugin::finish`, calls `validate_timeline_refs` over the `TimelineLibrary` resource, and panics with a descriptive message if any dangling reference is found — exercised by the S02 validator unit tests.
- Full suite remains green (empty `TimelineLibrary` in existing tests means zero validation errors).

---

### UAT-5: Kernel discipline — no Bevy windowing/rendering imports in src/combat/api/

**Steps:**
1. `rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/ --glob '*.rs'` (excluding comments).

**Expected outcome:** 0 real import lines (documentation comments referencing these paths are acceptable).

---

## Not Proven By This UAT
- Clock-gated `AdvanceMode` (windowed step-through) — deferred to S04.
- `SignalBus` / `PassiveRunner` integration — deferred to S04.
- Real world-aware selectors querying live Bevy ECS — deferred to S03.
- JSONL `CombatKernelTransition::Blueprint` round-trip — deferred to S04.
- Multi-level nested `LoopFrame` — deferred to S06 when Loop tier-N is needed.
