---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Add Bounce(u8) variant + pure next_bounce_hop() selector with table-driven tests

Add `Bounce(u8)` as a tuple variant to `TargetShape` in `src/data/skills_ron.rs`. Add a pure helper `next_bounce_hop(snapshot: &TargetableSnapshot, already_hit: &HashSet<UnitId>, primary_team: Team) -> Option<UnitId>` in `src/combat/resolution.rs`. Selector semantics: filter snapshot to entries where (team == primary_team.opposite() OR same-side enemies of caster — pass enemy team explicitly), alive == true, id not in already_hit; rank by HP%-per-mille ascending (`(hp_current * 1000) / hp_max`) with `slot_index` ascending as deterministic tie-break; return first or None. Use integer math only (no f32) — per-mille mirrors the existing HP-fraction convention. Add `#[cfg(test)] mod tests` cases inside resolution.rs alongside existing `resolve_targets_*` tests: (a) two enemies same HP% → lower slot_index wins; (b) one enemy at lower HP% but already_hit → next-lowest selected; (c) all candidates KO → None; (d) empty pool → None; (e) primary in already_hit → not selected. Do NOT yet wire Bounce into validation gates or pipeline — those are T02/T03. Keep `resolve_targets()` total: for `TargetShape::Bounce(_)`, return `vec![primary]` (hop 0 only).

## Inputs

- ``src/data/skills_ron.rs` — current TargetShape enum (line 9) lacks Bounce; tuple variant must derive same traits as Single/Blast`
- ``src/combat/resolution.rs` — TargetEntry/TargetableSnapshot definitions (lines 58-70), existing resolve_targets() impl (line 77)`
- ``.gsd/milestones/M018/slices/S03/S03-RESEARCH.md` — selector semantics and integer-math convention`

## Expected Output

- ``src/data/skills_ron.rs` — TargetShape gains `Bounce(u8)` variant`
- ``src/combat/resolution.rs` — `pub fn next_bounce_hop(...) -> Option<UnitId>` plus #[cfg(test)] mod cases; `resolve_targets()` Bounce match arm returning vec![primary]`

## Verification

cargo test --lib resolution::tests::next_bounce_hop && cargo check
