---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Migrate follow_up + in-tree tests to StatusBag

Update `src/combat/follow_up.rs:90-108` query tuple from `Option<&'static mut StatusEffect>` to `Option<&'static mut StatusBag>`. If any read accessed `.kind` directly, replace with `bag.has(kind)` / `bag.get_dur(kind)` lookups. Update every fixture in `src/combat/turn_system/tests.rs` (lines 136, 155, 183, 234, 286, 324, 338 per research) that currently spawns `StatusEffect { kind, duration_remaining }` directly: replace with `let mut bag = StatusBag::default(); bag.apply(kind, dur); commands.spawn((..., bag))`. Update every assertion that reads `app.world().get::<StatusEffect>(entity)` to `app.world().get::<StatusBag>(entity)` plus `.has(kind)` / `.get_dur(kind)`. Do not bypass the policy by constructing instances with private-field access — go through `apply`. After this task `cargo check` and `cargo test --lib` must be fully green; integration tests in `tests/` may still have stale references (handled in T05).

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/follow_up.rs`
- `src/combat/turn_system/tests.rs`

## Expected Output

- `src/combat/follow_up.rs`
- `src/combat/turn_system/tests.rs`

## Verification

`cargo check` clean across the whole tree. `cargo test --lib` green. Grep `rg 'StatusEffect\s*\{' src/` returns zero hits (all spawns go through `StatusBag::apply`).

## Observability Impact

None — pure migration of read sites.
