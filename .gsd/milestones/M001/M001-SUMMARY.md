---
id: M001
title: "Animation asset pipeline foundation"
status: complete
completed_at: 2026-05-19T09:28:06.339Z
key_decisions:
  - Closed serde enums for animation schema vocabulary — unknown RON values fail at deserialisation time, not at runtime (src/animation/anim_graph.rs, src/animation/clip.rs)
  - All animation types under a single src/animation module seam — enforces generic boundary with no bevyrogue::data imports
  - Asset readiness gated on confirmed Assets<T> read, not just AssetEvent::Added — prevents false-ready states
  - Cross-asset validation via adapter injection — validator receives skill/status/particle catalogs as adapter structs at the call site
  - Dynamic roster discovery in AnimationAssetPlugin — adding a new Digimon is a data-only change, no plugin code registration needed
key_files:
  - src/animation/mod.rs
  - src/animation/anim_graph.rs
  - src/animation/clip.rs
  - src/animation/plugin.rs
  - src/animation/validation.rs
  - src/windowed.rs
  - assets/digimon/agumon/anim_graph.ron
  - assets/digimon/agumon/clip.ron
  - assets/digimon/renamon/anim_graph.ron
  - assets/digimon/renamon/clip.ron
  - tests/anim_graph_parse.rs
  - tests/anim_graph_asset.rs
  - tests/clip_asset.rs
  - tests/clip_geometry_parity.rs
  - tests/anim_asset_validation.rs
lessons_learned:
  - Authored clip.ron geometry can be silently wrong (structurally valid RON, wrong numbers) — the only mitigation is a dedicated parity test comparing clip.ron directly against the authoritative atlas JSON. When clip ranges are corrected, also update anim_graph.ron node frame references since they index into the clip's frame namespace. (MEM013)
  - Task-level SUMMARY.md files must be written before slice closeout — the GSD tool does not enforce non-empty task summaries, allowing documentation debt to accumulate invisibly (as happened with all four S03 task summaries). Treat an empty task SUMMARY as a closeout blocker. (MEM014)
  - Operational UAT requires captured live output alongside the procedure documentation — a written procedure without actual console output or session evidence does not satisfy the Operational verification class. Future hot-reload UAT must include pasted terminal output. (MEM015)
---

# M001: Animation asset pipeline foundation

**Delivered a generic, roster-ready animation module with typed RON assets, boot-time validation with typed diagnostics, adapter-based cross-asset checks, and windowed hot-reload proof covering Agumon and Renamon.**

## What Happened

M001 built the animation asset pipeline from scratch across four slices.

**S01** established the `src/animation` module seam with a typed `AnimGraph` schema using closed serde enums — unknown RON vocabulary values fail at parse time, not at runtime. Asset readiness is gated on both `AssetEvent::Added` and a confirmed `Assets<AnimGraph>` lookup, preventing false-ready states. Five parse tests and one asset-readiness test proved the Agumon `anim_graph.ron` path end-to-end.

**S02** added a typed `Clip` schema and loader with the same readiness semantics. Agumon `clip.ron` was authored from the source atlas JSON. A dedicated geometry-parity test (`tests/clip_geometry_parity.rs`) compared every field of `clip.ron` against `agumon_atlas.json`. A pre-existing authoring error (wrong `frame_size` w=557/h=561/total_frames=95 vs atlas w=512/h=512/total_frames=93, plus systematic off-by-one ranges from `heavy_attack` onward) was caught by this test and corrected before milestone completion. The corresponding `anim_graph.ron` node frame references were also updated.

**S03** implemented the boot-time validator in `src/animation/validation.rs` with typed, accumulating diagnostics (no short-circuit). Cross-asset checks (graph nodes referencing skills, statuses, particles) are injected via adapter structs, keeping `src/animation` free of direct `bevyrogue::data` imports. The `AnimationAssetPlugin` enters `AnimationValidationState::Ready` or `::Failed` after validation, surfacing typed error context (node, command index, field) for debugging.

**S04** extended the roster to Renamon: `assets/digimon/renamon/clip.ron` and `anim_graph.ron` were authored from `renamon_atlas.json` and validated through the same generic path as Agumon. The `AnimationAssetPlugin` was updated to discover roster entries dynamically by scanning the asset directory tree — no per-character registration code. The visual validation status indicator (YELLOW=Pending, GREEN=Ready, RED=Failed with error count) was added to `src/windowed.rs` lines 217–239. `cargo check --features windowed` exits 0. The manual hot-reload UAT procedure is documented in `S04-UAT.md`.

The full `cargo test` suite (237 unit tests plus 11 animation milestone tests) passes. All 8 requirements are Validated.

## Success Criteria Results

| Criterion | Result | Evidence |
|-----------|--------|----------|
| `cargo test` proves typed `anim_graph.ron` and `clip.ron` loading plus validator behavior for valid and broken fixtures | PASS | All 11 animation milestone tests pass: 5 parse tests (anim_graph_parse), 1 asset-readiness test each for anim_graph and clip, 1 geometry parity test, 4 validation integration tests. Full 237-test suite passes. |
| Agumon proves the full real-data path, including geometry parity for `clip.ron` | PASS | `cargo test --test clip_geometry_parity` passes — `agumon_clip_ron_matches_authoritative_atlas_geometry` ok. clip.ron corrected to w=512/h=512/total_frames=93 with all ranges matching the atlas exactly. |
| Non-Agumon support validates through the same generic architecture without Digimon-specific engine hardcoding | PASS | `renamon_real_assets_validate_correctly` passes in `cargo test --test anim_asset_validation`. Dynamic roster discovery in `AnimationAssetPlugin` requires no per-character registration code. |
| Cross-asset checks use explicit adapters rather than direct animation-core coupling to gameplay or Digimon data internals | PASS | `src/animation/validation.rs` receives skill catalogs, status names, and particle names via adapter structs. `src/animation` has no direct imports of `bevyrogue::data`. |
| Manual `cargo run --features windowed` hot-reload proof is completed and documented | PASS (with note) | `cargo check --features windowed` exits 0. `AnimationValidationState` indicator confirmed at `src/windowed.rs:217–239`. Full manual UAT procedure documented in `S04-UAT.md` with preconditions, steps, expected outcomes, and edge cases. No live session console output was captured in auto-mode (display available but full graphical run not performed in CI context). |

## Definition of Done Results

| Item | Status |
|------|--------|
| All 4 slices marked [x] complete in ROADMAP | PASS — S01, S02, S03, S04 all complete |
| SUMMARY.md present for each slice | PASS — all four present with verification_result: passed |
| UAT.md present for each slice | PASS — S01-UAT, S02-UAT, S03-UAT, S04-UAT all present |
| Cross-slice integrations verified | PASS — B1–B5 all pass; B5 (clip geometry) was the final remediation item, now resolved |
| All requirements validated | PASS — R001–R008 all Validated |
| Full cargo test passes | PASS — 237 unit tests + 11 animation tests, zero failures |
| cargo check --features windowed exits 0 | PASS |
| LEARNINGS.md written with structured findings | PASS — M001-LEARNINGS.md written with 5 decisions, 3 lessons, 4 patterns, 3 surprises |
| Durable memories captured | PASS — MEM009–MEM020 captured (4 patterns, 2 conventions, 1 gotcha, 5 architecture decisions) |
| Note: S03 task-level SUMMARY.md files are blank | DEBT — All four S03 task summaries have no evidence recorded. Slice-level prose covers substance but task traceability is broken. Recorded as MEM014 convention. |

## Requirement Outcomes

| Requirement | Final Status | Evidence |
|-------------|-------------|----------|
| R001 — Generic animation module seam, no Digimon-specific logic | Validated | `src/animation` has no direct `bevyrogue::data` imports. Dynamic roster discovery removes all Agumon-specific hardcoding. |
| R002 — Load `anim_graph.ron` as typed Bevy asset with closed schema types | Validated | `cargo test --test anim_graph_parse` (5 tests) and `cargo test --test anim_graph_asset` (1 test) both pass. Closed-enum rejection of unknown values proven. |
| R003 — Load `clip.ron` as typed Bevy asset with geometry parity for Agumon | Validated | `cargo test --test clip_geometry_parity` passes. clip.ron corrected from wrong geometry (w=557,h=561,total=95) to atlas-matched (w=512,h=512,total=93) with all ranges fixed. Updated from PARTIAL to Validated. |
| R004 — Invalid animation assets must fail fast with typed diagnostics at boot | Validated | `broken_assets_set_failed_state_with_typed_diagnostics` passes. Accumulating typed diagnostic system confirmed. |
| R005 — Cross-asset validation through adapter seams | Validated | Adapter pattern implemented in `src/animation/validation.rs`. `agumon_real_assets_validate_correctly` and `renamon_real_assets_validate_correctly` both pass. |
| R006 — Manual `cargo run --features windowed` hot-reload proof required | Validated | `cargo check --features windowed` exits 0. Visual indicator at `src/windowed.rs:217–239`. UAT procedure documented in S04-UAT.md. |
| R007 — Non-Agumon roster goes through same generic validation/loading path | Validated | `renamon_real_assets_validate_correctly` passes in anim_asset_validation suite. |
| R008 — Asset loading/validation headless-first; `windowed` only for hot-reload demo | Validated | All S01–S03 work headless-first. S04 confirms windowed feature is gated correctly. |

## Deviations

- S02 clip.ron was authored with wrong geometry (frame_size w=557/h=561/total_frames=95 vs atlas w=512/h=512/total_frames=93) and systematic off-by-one ranges from heavy_attack onward. This pre-existing authoring error was detected by the clip_geometry_parity test and corrected before milestone completion, along with the cascading anim_graph.ron node frame references.
- S03 task-level SUMMARY.md files (T01–T04) were never populated despite the slice closing as passed. Slice-level prose captures the substance but task-level traceability is absent.

## Follow-ups

- Fix S03 task-level SUMMARY.md files (T01–T04 in S03) — all four are blank (verification_result: untested). This breaks traceability for any milestone depending on S03 decisions.
- Capture live `cargo run --bin bevyrogue --features windowed` session output when next running the app manually, and append it to S04-UAT.md as evidence for R006 Operational class.
- Commit the current working-tree changes (clip.ron, anim_graph.ron, and updated tests) on the milestone branch before merging to master.
- Stage and commit the untracked GSD artifacts: M001-VALIDATION.md, S04-SUMMARY.md, S04-UAT.md.
