# S05 Assessment

**Milestone:** M006
**Slice:** S05
**Completed Slice:** S05
**Verdict:** pass
**Created:** 2026-05-27T07:53:58.765Z

## Assessment

Cross-slice dependency/scope audit of the remaining slices (S06–S15) found ordering contradictions where content-mutating slices were not ordered after the structural render.rs split, plus shared-helper and anti-churn gaps. Adjustments applied:

DEPENDENCY FIXES (this reassessment)
- S12 depends -> [S10, S11]: S12 mutates effect-registry content and references "the registries module". That module is created by S09 and render.rs is decomposed by S10. Previously S12 depended only on S11, so it could land while render.rs was still a monolith (stale file-list) or race the split. Ordering S12 after S10 means all content mutation happens on the settled submodule layout.
- S14 depends -> [S10, S12]: S14's T01 edits render.rs for the VfxAsset->enoki adapter. Same reason as S12 — must run after the split (S10) and after the keyed-registry conversion (S12).
- S13 depends -> [S08, S11, S12]: S13's "added purely by new data + a register(), zero path-constant edits" claim REQUIRES S11 (data-driven catalog discovery that removes DEFAULT_ANIM_GRAPH/CLIP/STANCE_PATHS). S11 was only transitively present via S12; now explicit so the zero-edit thesis is guaranteed.
- S15 depends -> [S10, S13]: S15 thins/removes the contract tests (renamon_extension_contract.rs, agumon_module_extraction.rs) and writes the anti-churn DECISIONS rule. S13 T03 ADDS assertions to renamon_extension_contract.rs and catalog_discovery.rs. With no ordering, S13 after S15 would re-introduce exactly the brittleness S15 removed. S15 must run last (after S13).

SCOPE DIRECTIVES FOR EXECUTING AGENTS (no separate replan; fold in at execution)
- S09 (extract): in addition to moving registries/types, EXTRACT A SHARED warn-once helper. Today S06's warn-once is an inline `Local<HashSet<AssetId<AnimGraph>>>` in src/animation/registry.rs, specific to AssetId<AnimGraph> and not reusable as-is. S12 (keyed-registry miss), S14 (unmapped VfxAsset verb), and S13 (exercises spawn/cue-miss diagnostics) all assume a reusable warn-once. S09 must promote it to a generic dedup util keyed by an arbitrary id type so downstream slices reuse rather than re-implement. Fix the "reuse the S06 warn helper" wording in S08/S11/S12/S14 to point at this shared util.
- S12 / S14 file-lists: their "Files Likely Touched" still name src/windowed/render.rs as a monolith. After S10 the split is render/{playback,spawn,effects,feedback,clock,registries}.rs. Retarget edits to the relevant submodule(s) instead of render.rs.
- S13 T03 contract-test additions must already comply with the S15 anti-churn rule: assert only the durable invariant (engine render core contains no per-species id) as an absence-guard, NOT exact file shape. This prevents S15 from having to undo S13's additions.

Slices S01–S05 are complete and unchanged. Slice titles, risks, and demos are otherwise preserved.
