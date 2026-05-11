---
id: T03
parent: S02
milestone: M011
key_files:
  - src/combat/rng.rs
  - src/combat/events.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - src/combat/follow_up.rs
  - src/headless.rs
  - src/combat/mod.rs
  - tests/event_stream.rs
  - tests/status_accuracy.rs
key_decisions:
  - CombatRng uses StdRng (rand 0.8 default features) instead of SmallRng (requires small_rng feature opt-in) — functionally equivalent for game RNG
  - Option<ResMut<CombatRng>> in both advance_turn_system and resolve_action_system keeps all 17 existing test files working without modification
  - Fallback when CombatRng absent: one-shot CombatRng::from_seed(42) — boundary cases (0/100) always handled correctly by roll_pct clamp logic
  - status_accuracy tests use seed search loops (miss_seed/hit_seed) so tests are self-documenting and implementation-agnostic
duration: 
verification_result: passed
completed_at: 2026-04-27T14:17:19.611Z
blocker_discovered: false
---

# T03: feat(combat): add CombatRng resource + OnStatusResisted event + status accuracy roll + Shock RNG retrofit (R019)

**feat(combat): add CombatRng resource + OnStatusResisted event + status accuracy roll + Shock RNG retrofit (R019)**

## What Happened

## What Happened

Introduced deterministic RNG infrastructure for the combat system (R019).

**`src/combat/rng.rs` (new)**: `CombatRng(StdRng)` Bevy Resource with `from_seed(u64)` constructor and `roll_pct(threshold: i32) -> bool` helper. Uses `StdRng` (available in rand 0.8 default features) instead of `SmallRng` (requires opt-in feature) to avoid Cargo.toml changes — functionally equivalent for game RNG. Boundary cases are clamped: threshold ≤ 0 → always false, threshold ≥ 100 → always true.

**`src/combat/events.rs`**: Added `OnStatusResisted { kind: StatusEffectKind }` variant to `CombatEventKind`. Emitted when the accuracy roll fails; source = attacker, target = intended defender, no `StatusEffect` inserted.

**`src/combat/turn_system/pipeline.rs`**: Wired the accuracy roll into `step_app` status application block (formerly lines 191–196). After `outcome.succeeded`, computes `threshold = (triangle_modifiers(attacker.attribute, defender.attribute).status_acc_modifier * 100.0) as i32` then calls `rng.roll_pct(threshold)`. Hit → `OnStatusApplied` + insert `StatusEffect`. Miss → `OnStatusResisted` only. Lifecycle contract preserved: roll happens between `OnActionPreApp` and `OnActionApplied`.

**`src/combat/turn_system/mod.rs`**: Removed `rand::thread_rng()` from the Shock cancel path in `advance_turn_system`, replaced with `combat_rng.roll_pct(cancel_chance_pct)`. Both `resolve_action_system` and `advance_turn_system` now take `Option<ResMut<CombatRng>>` — optional so existing tests without the resource continue to work (fallback: one-shot seed-42 local rng, boundary cases still handled correctly).

**`src/combat/follow_up.rs`**: `resolve_follow_up_action_system` also updated to accept `Option<ResMut<CombatRng>>` and pass it through to `step_app`.

**`src/headless.rs`**: Added `init_resource::<CombatRng>()` to the headless plugin registration.

**`src/combat/mod.rs`**: Exported `pub mod rng`.

**`tests/event_stream.rs`**: Added `OnStatusApplied { .. }` and `OnStatusResisted { .. }` arms to the exhaustive match guard.

**`tests/status_accuracy.rs` (new)**: Three deterministic tests:
1. Vaccine→Data miss: `miss_seed(90)` finds lowest seed where `roll_pct(90)` returns false → `OnStatusResisted`, no `StatusEffect` component.
2. Vaccine→Data hit: `hit_seed(90)` finds lowest seed where `roll_pct(90)` returns true → `OnStatusApplied`, `StatusEffect` present.
3. Vaccine→Vaccine neutral: seed=0, threshold=100 → always passes (R076).

Seed search loops are deterministic given a fixed RNG implementation — the searches always find the same seeds.

## Verification

Ran `cargo test --test status_accuracy --no-fail-fast`: 3/3 pass. Ran `cargo test --test pipeline_dispatch --no-fail-fast`: 3/3 pass. Ran `cargo test --no-fail-fast`: 0 failures across all 27 test binaries. Verified `grep -rn 'thread_rng' src/combat/` returns CLEAN.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test status_accuracy --no-fail-fast` | 0 | ✅ pass | 3100ms |
| 2 | `cargo test --test pipeline_dispatch --no-fail-fast` | 0 | ✅ pass | 700ms |
| 3 | `cargo test --no-fail-fast 2>&1 | grep -E 'FAILED|0 failed'` | 0 | ✅ pass — 0 failed across all binaries | 15000ms |
| 4 | `grep -rn 'thread_rng' src/combat/` | 1 | ✅ pass — no thread_rng remaining | 50ms |

## Deviations

StdRng used instead of SmallRng (plan said SmallRng). Reason: SmallRng requires the `small_rng` feature flag for rand 0.8, which is not in Cargo.toml defaults. StdRng is available without any dependency changes and is behaviorally identical for deterministic seeded game RNG. The systems use Option&lt;ResMut&lt;CombatRng&gt;&gt; instead of ResMut&lt;CombatRng&gt; — this preserves backward compat across 17 existing test files that don't need RNG control and would otherwise require boilerplate updates.

## Known Issues

None.

## Files Created/Modified

- `src/combat/rng.rs`
- `src/combat/events.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `src/headless.rs`
- `src/combat/mod.rs`
- `tests/event_stream.rs`
- `tests/status_accuracy.rs`
