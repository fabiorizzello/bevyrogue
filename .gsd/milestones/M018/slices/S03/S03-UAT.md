# S03: BounceSelector + RepeatPolicy DSL + generic Bounce kernel hop loop — UAT

**Milestone:** M018
**Written:** 2026-05-13T21:58:16.719Z

# S03: BounceSelector + RepeatPolicy DSL + generic Bounce kernel hop loop — UAT

**Milestone:** M018
**Written:** 2026-05-13

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: The Bounce pipeline is pure game logic with deterministic behavior under a fixed seed/composition. Integration tests cover all selectors, repeat policies, curve types, and edge cases. No runtime UI or external service is involved.

## Preconditions

- Rust toolchain installed per `rust-toolchain.toml`
- Working directory: `/home/fabio/dev/bevyrogue`
- No uncommitted source edits

## Smoke Test

```
cargo test --test target_shape_bounce_chain
```
Expected: `4 passed; 0 failed` — confirms the Bounce hop loop, selectors, and DamageCurve scaling all work end-to-end.

## Test Cases

### 1. NextSlot selector, NoRepeat, FalloffPct curve, mid-chain KO

```
cargo test --test target_shape_bounce_chain bounce_next_slot_no_repeat_falloff_ko_mid_chain
```
1. Skill with `hops=3`, `selector=NextSlot`, `repeat=NoRepeat`, `curve=FalloffPct(50)` is cast.
2. An enemy in the chain is KO'd mid-hop.
3. **Expected:** Remaining hops skip the KO'd target, damage falls off by 50% each hop, chain terminates at pool exhaustion without error.

### 2. LowestHp selector, NoRepeat, Constant curve, full chain

```
cargo test --test target_shape_bounce_chain bounce_lowest_hp_no_repeat_constant_full_chain
```
1. Skill with `selector=LowestHp`, `repeat=NoRepeat`, `curve=Constant` is cast against 3 enemies.
2. **Expected:** Each hop hits the lowest-HP remaining target not yet in `already_hit`; damage is constant across all hops.

### 3. LowestHp selector, AllowRepeat, PerHop curve

```
cargo test --test target_shape_bounce_chain bounce_lowest_hp_allow_repeat_per_hop_curve
```
1. Skill with `selector=LowestHp`, `repeat=AllowRepeat`, `curve=PerHop([100, 80, 60])` is cast.
2. **Expected:** Each hop may re-hit the same target; damage per hop matches the PerHop array values exactly.

### 4. Pool exhaustion truncates silently

```
cargo test --test target_shape_bounce_chain bounce_pool_exhaustion_truncates_silently
```
1. Skill requests more hops than there are valid targets under `NoRepeat`.
2. **Expected:** Loop exits early without panic, no `OnActionFailed` event emitted, completed hops are applied correctly.

## Edge Cases

### Bounce against single remaining enemy with NoRepeat

1. Only one enemy alive; skill has `hops=3`, `NoRepeat`.
2. **Expected:** First hop hits the enemy; remaining 2 hops find empty candidate pool and exit silently.

### DamageCurve::PerHop length validation at skill load

1. `skills.ron` entry with `hops=3` and `PerHop([100, 80])` (length mismatch).
2. **Expected:** `validate_skill_def` rejects at load time with a parse/validation error logged; the skill is not registered.

## Failure Signals

- Any `cargo test` failure in `target_shape_bounce_chain` indicates a regression in the Bounce hop loop.
- `cargo check` errors indicate a type mismatch in BounceSelector, RepeatPolicy, or DamageCurve propagation.
- Silent over-targeting (hitting KO'd enemies or repeating under NoRepeat) would surface as wrong damage totals in test assertions.

## Not Proven By This UAT

- Per-hop `CombatEvent` emission (deferred — hop loop currently applies damage without emitting individual hop events).
- `OnActionFailed` on pool exhaustion (deferred — currently silent).
- UI rendering of Bounce skill chains (no windowed feature exercised here).
- Performance under large enemy counts (> 10 targets).

## Notes for Tester

All tests run headless with deterministic seeding — no wall-clock or random dependencies. The `chain_bolt` fixture lives inline in `tests/target_shape_bounce_chain.rs` and is not in `assets/data/skills.ron`; this is intentional to preserve the catalog-size assertion in lib tests.
