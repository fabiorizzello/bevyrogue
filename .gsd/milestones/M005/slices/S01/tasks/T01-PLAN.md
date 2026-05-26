---
estimated_steps: 3
estimated_files: 4
skills_used: []
---

# T01: Add pure stance-reaction mapping function with headless hit/death/precedence/no-op tests

Why: The milestone requires the event-to-stance-reaction mapping to be a pure, deterministic lib function (mirroring the R009 AnimGraphInput purity seam) so it has a headless contract instead of living only behind K001 in the windowed binary; integration tests link only against the lib crate, not the windowed binary.

Do: Create `src/animation/reaction.rs` defining a closed typed `StanceReaction` enum (`Hurt`, `Death`) and a pure function `stance_reaction_for(kind: &CombatEventKind) -> Option<StanceReaction>` that maps `OnHitTaken` → `Some(Hurt)`, `UnitDied` → `Some(Death)`, and every other variant → `None` (total over the taxonomy, no panic, no catch-all that silently swallows future variants — match explicitly or with a documented `_ => None`). Add a batch resolver `resolve_stance_reaction<'a>(kinds: impl IntoIterator<Item = &'a CombatEventKind>) -> Option<StanceReaction>` that encodes death-precedence: return `Some(Death)` if any kind maps to Death, else `Some(Hurt)` if any maps to Hurt, else `None`. Add `impl StanceReaction { pub fn stance_node(self) -> NodeId }` returning `NodeId("hurt")` / `NodeId("death")` (these node names match assets/digimon/agumon/stance.ron). Import `CombatEventKind` from `crate::combat::observability::events` and `NodeId` from `crate::animation::anim_graph`. Register the module in `src/animation/mod.rs` (`pub mod reaction;` + `pub use reaction::*;`). Then add `tests/animation/stance_reaction_mapping.rs` with four cases: (1) hit — OnHitTaken maps to Hurt and stance_node() == NodeId("hurt"); (2) death — UnitDied maps to Death and stance_node() == NodeId("death"); (3) death-precedence — a slice containing both OnHitTaken and UnitDied resolves to Death; (4) no-op — a representative non-reaction kind (e.g. OnSkillCast / OnActionResolved) maps to None and an empty batch resolves to None. Register the file in `tests/animation.rs` with a `#[path = "animation/stance_reaction_mapping.rs"] mod stance_reaction_mapping;` line.

Done when: `cargo test --test animation` is green including the four new cases, and the mapping lives entirely in the lib with no windowed/bevy-render dependency.

## Inputs

- `src/combat/observability/events.rs`
- `src/animation/anim_graph.rs`
- `src/animation/mod.rs`
- `tests/animation.rs`
- `assets/digimon/agumon/stance.ron`

## Expected Output

- `src/animation/reaction.rs`
- `src/animation/mod.rs`
- `tests/animation/stance_reaction_mapping.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation stance_reaction_mapping
