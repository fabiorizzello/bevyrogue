---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Migrate follow_up + in-tree tests to StatusBag

Edit `src/combat/follow_up.rs`. Update import at line 4: `StatusEffect` → `StatusBag`. Update `ResolveActorsQuery` type alias at line 104: `Option<&'static mut StatusEffect>` → `Option<&'static mut StatusBag>`. Confirmed via grep that no `.kind` read access exists at this site (pure ownership pass-through for downstream systems), so the migration is a pure rename.

Edit `src/combat/turn_system/tests.rs`. Update the import at line 3 to add `StatusBag` (keep `StatusEffectKind`). Rewrite all fixture spawns to use `StatusBag::default()` + `bag.apply(kind, dur)` pre-spawn — confirmed sites: lines 136 (Heated dur=2 first apply), 155 (Heated dur=2 primer), 162 (Heated dur=4 re-apply over primer), 183 (Chilled dur=2 first apply), 199 (Chilled dur=2 primer), 206 (Chilled dur=4 re-apply), 234 (Paralyzed dur=1), 286 (Paralyzed dur=2 primer), 302 (Paralyzed dur=2 re-apply), 324 (Heated dur=1 for the world.get assert path). Pattern: `let mut bag = StatusBag::default(); bag.apply(StatusEffectKind::Heated, 2); commands.spawn((..., bag))`. Update all assertions: `app.world().get::<StatusEffect>(entity).is_none()` (lines 149, 193) → `app.world().get::<StatusBag>(entity).map_or(true, |b| b.is_empty())`; the `assert_eq!(app.world().get::<StatusEffect>(entity), Some(&StatusEffect { kind: ..., duration_remaining: N }))` pattern at lines 337-339 → `assert_eq!(app.world().get::<StatusBag>(entity).and_then(|b| b.get_dur(&StatusEffectKind::Heated)), Some(N))`. Do not bypass policy with direct field access — always go through `apply` / `has` / `get_dur`. After this task `cargo check` and `cargo test --lib` must be fully green; integration tests in `tests/` will still have stale references (handled in T05).

## Inputs

- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`
- `src/combat/status_effect.rs` (post-T01)
- `src/combat/follow_up.rs`
- `src/combat/turn_system/tests.rs`

## Expected Output

- `src/combat/follow_up.rs`
- `src/combat/turn_system/tests.rs`

## Verification

`cargo check` clean across the whole tree. `cargo test --lib` green. `rg 'StatusEffect\s*\{' src/` returns 0 hits (all spawns go through `StatusBag::apply`). `rg "&'static mut StatusEffect\b|&mut StatusEffect\b" src/combat/follow_up.rs` returns 0.

## Observability Impact

None — pure migration of read sites + fixture rewrite to honor the new policy.
