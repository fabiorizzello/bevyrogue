---
estimated_steps: 17
estimated_files: 3
skills_used: []
---

# T01: Timeline data types + validate_timeline_refs + validator unit test

Why: the graph types and their validator are pure data, Bevy-agnostic, and unblock every later task. Porting them first keeps the surface small and lets the validator test (demo gate 2) drive TDD without a runner.

Do:
1. Create `src/combat/api/timeline.rs`. Port from `.gsd/workflows/spikes/M021-timeline-fsm/src/lib.rs`:
   - `BeatId` (newtype around `&'static str` or `u32` — match spike).
   - `Presentation { cue_id: &'static str, /* spike fields */ }` (data only).
   - `BeatKind` enum: `Cast`, `Phase`, `Impact`, `Aftermath`, `Loop { body: Vec<Beat>, exit_when: &'static str }` (predicate id). Drop `derive(PartialEq, Eq)`.
   - `Beat { id: BeatId, kind: BeatKind, hook: Option<&'static str>, selector: Option<&'static str>, presentation: Option<Presentation> }`.
   - `BeatEdge { from: BeatId, to: BeatId, gate: Option<&'static str> /* PredicateId */ }`.
   - `CompiledTimeline { id: &'static str, entry: BeatId, beats: Vec<Beat>, edges: Vec<BeatEdge> }`.
   - `BeatEvent { cast_id: CastId, beat_id: BeatId, hop_index: u32, beat_targets: Vec<UnitId> }`.
   - `SelectorCtx<'a>` and `CueCtx<'a>` — minimal: `caster`, `primary_target`, and a state handle field (use `&'a World` placeholder type — keep import-free here by making it a generic `S` parameter or by `pub use bevy::prelude::World` only in the runner/skill_ctx; for S02 fixture simplicity we accept `&'a World` in `SelectorCtx`).
   - `ValidationError { axis: &'static str, missing_id: String, site: String }` (site is `beat <id>` or `edge <from>-><to>`).
   - `pub fn validate_timeline_refs(timeline: &CompiledTimeline, regs: &ExtRegistries) -> Result<(), Vec<ValidationError>>` — recursive over `BeatKind::Loop.body`, validating hook/selector references on beats and gate predicates on edges; loop `exit_when` predicate must resolve in `regs.predicates`.
2. Add `pub mod timeline;` to `src/combat/api/mod.rs` and re-export the public types.
3. Add an inline `#[cfg(test)] mod tests` exercising (a) a clean timeline validates Ok, (b) a missing hook returns Err with axis="hook", (c) a missing edge gate returns Err with axis="predicate" + site="edge from->to", (d) a missing `exit_when` predicate inside a Loop returns Err with site referencing the Loop beat.
4. Create `tests/timeline_validate_typo.rs` mirroring case (b) at integration level so the slice-level demo gate runs end-to-end against the public API.

Done-when: `cargo check` clean; `cargo test timeline_validate_typo` green; inline timeline-validator unit tests green. No new bevy::winit/render/bevy_egui imports.

## Inputs

- `.gsd/workflows/spikes/M021-timeline-fsm/src/lib.rs`
- `src/combat/api/mod.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/intent.rs`
- `.gsd/milestones/M021/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/combat/api/timeline.rs`
- `src/combat/api/mod.rs`
- `tests/timeline_validate_typo.rs`

## Verification

cargo check && cargo test --test timeline_validate_typo && cargo test --lib combat::api::timeline:: && rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs
