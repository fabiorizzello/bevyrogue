---
estimated_steps: 8
estimated_files: 2
skills_used: []
---

# T02: Add Blast variant + pure resolve_targets() helper with table-driven tests

Extend TargetShape with a Blast variant in src/data/skills_ron.rs. Treat the roadmap label 'AoE(All)' as a DSL alias for the existing AllEnemies variant — do NOT add a new variant (avoid wide diff across 11 deferred skills). Document the alias in the slice SUMMARY when T05 lands.

Add a pure helper `pub fn resolve_targets(shape: &TargetShape, primary: UnitId, snapshot: &TargetableSnapshot) -> Vec<UnitId>` to src/combat/resolution.rs. Contract:
- Single → [primary]
- Blast → [primary, primary's slot-1 same-team, primary's slot+1 same-team] filtered to alive, sorted by slot_index ascending. KO'd adjacent absorbed (omitted from result).
- AllEnemies → every alive unit on the enemy team of primary, sorted by slot_index ascending.
- Row, SelfOnly → leave at current behavior (Single fallback or unimplemented — match what step_app does today).

Use the existing TargetableSnapshot if it carries Team + SlotIndex + alive; otherwise extend it minimally to include slot_index per entry. The helper must be deterministic — sort explicitly, never trust query order.

Add table-driven tests (inline in resolution.rs under #[cfg(test)] mod tests, OR in tests/target_shape_resolve.rs — pick whichever already exists for resolution.rs tests). Cases: Blast at edge slot (primary=slot 0 → returns [0,1] only); Blast with KO'd adjacent (returns [primary, far-side]); AllEnemies with one dead (omits the dead unit); all results slot_index ascending.

## Inputs

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `src/combat/unit.rs`
- `src/combat/bootstrap.rs`

## Expected Output

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`

## Verification

cargo test resolve_targets 2>&1 | grep -E '(test result|FAILED)' && cargo check 2>&1 | tail -5
