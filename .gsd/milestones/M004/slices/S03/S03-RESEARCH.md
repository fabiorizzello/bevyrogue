# S03 Research: Skill-tree variation via variant selection + Baby Burner port

**Depth:** Targeted. Known stack (Bevy + owned typed VfxAsset), familiar subsystem (S01/S02 built the schema, resolver, PlacementExt axis, and windowed dispatcher). The one genuinely-new piece is the variant-selection seam; the Baby Burner half is largely already in place.

## Summary

S03 has two halves, and the surprising finding is that **one of them is already done**:

1. **Baby Burner detonate port — effectively complete from S02.** `assets/digimon/agumon/vfx.ron` already carries a `baby_burner.detonate` effect (lines 140-161), and `src/windowed/render.rs` already spawns it from data via `spawn_detonate_particles` (render.rs:1000-1068) → `spawn_effect_by_id` → `resolve_effect`. `VfxParticleKind` / `kind_from_name` were deleted in S02 and a headless grep-guard (`tests/animation/render_no_vfx_kind_guard.rs`) already enforces their absence. **The S03 "no hardcoded VFX paths left in render.rs" criterion is therefore already satisfied** — the grep-guard makes it CI-provable today. What remains for Baby Burner is *quality*: today's detonate is a deliberate placeholder (a single static size-18 flat quad reproducing the old Generic-kind detonate, vfx.ron:135-139 comment). S03 should enrich it into a real detonate (e.g. multi-particle burst + flash, reusing `fan_out` + a `static` flash, mirroring the Baby Flame impact pair) so the K001 visual review has something worth signing off.

2. **Variant-selection seam — net-new, the real architecture work of this slice.** No `VfxContext`, `variant`, or selection map exists in source today (only in the M004 planning docs). This is D033 graft 5: "variant SELECTION keyed by `(skill_id, variant_key)`, same idiom as anim_graph (state picks which effect-tree to instantiate)." Per M004-CONTEXT, S03 must **prove the seam deterministically (headless) but wire NO real gameplay skill-tree unlock** — there is no unlock system in the codebase (only `Predicate::Unlock` for anim_graph node-gating).

## Active Requirements This Slice Owns

- **R004 (determinism):** the variant-selection fn must be pure, seeded-free, and snapshot-stable — identical `VfxContext` → identical selected `EffectId`. This is the headless contract that proves the slice.
- **R002 / R016 (headless-first):** `VfxContext`, the variant map, and `select_variant` live in the headless `src/animation/` lib with zero windowed deps. Only consumption (if any) is windowed-gated.
- **R005 (dep gating):** no new winit/wgpu/egui. None needed — this is pure data + math.
- **R012 (no numeric gameplay payload on command surface):** the variant map is presentation data inside `VfxAsset`; it must not leak a gameplay value onto `SpawnParticle`/command surfaces (MEM044).

## Implementation Landscape

### Files and purpose

- **`src/animation/vfx_asset.rs`** — schema + resolver + validation home. Add: `VfxContext` (pure, render-free selection input), the variant map representation, and `select_variant(...)` (pure fn, mirrors `resolve_effect`/`validate_effects` style). Extend `validate_effects` to cover dangling variant targets.
- **`assets/digimon/agumon/vfx.ron`** — author (a) an enriched `baby_burner.detonate` (+ any sub-effects/flash it chains), and (b) a small `variants` block proving selection (e.g. a base detonate vs. an "empowered" detonate variant keyed by a synthetic `variant_key`). No real unlock — the variant_key is synthetic test/demo data.
- **`src/animation/placement.rs`** — likely reused as-is. An enriched detonate burst reuses `fan_out` (already registered); a flash reuses `static_placement`. Only a genuinely novel detonate motion would need a new pure verb + one `register()` call (per the slice's own acceptance bar).
- **`src/combat/blueprints/agumon/mod.rs`** — `register_agumon_ext` only needs a new line **if** a novel placement verb is added; pure RON enrichment needs no change here (this is the "RON-only reuse" path the milestone wants to demonstrate).
- **`src/combat/runtime/registry.rs`** — touch **only** if variant selection is exposed as its own ExtRegistries axis. **Recommendation: do NOT add a new axis.** Selection is a pure data lookup over `VfxAsset`, not an open-vocabulary behavior extension; keep it a free fn in `vfx_asset.rs` (see Recommendation). Adding a `VariationExt` axis would be premature per D033 ("modifiers/expression-DSL deferred-additive").
- **`src/windowed/render.rs`** — windowed consumption of variant selection is optional for the slice contract (the contract is the headless determinism test). If wired, the detonate spawn at render.rs:1047 would call `select_variant(ctx, asset)` before `spawn_effect_by_id`. Keep this thin; the burden of proof is headless.
- **`tests/animation/`** — new test module (e.g. `vfx_variant_selection.rs`) registered in `tests/animation.rs`, asserting deterministic `VfxContext` → `EffectId`. Also enrich the existing load test so the new detonate sub-effects + variant targets pass `validate_effects`.

### Natural seams (independent work units for the planner)

- **Seam A — variant-selection seam (headless, pure):** `VfxContext` type + variant map schema field + `select_variant` fn + validation extension + headless determinism test. Fully independent of windowed code. **This is the highest-risk / biggest-unblocker — do it first** (it is the only novel design in the slice and the only CI-provable S03 success criterion).
- **Seam B — Baby Burner detonate enrichment (data + visual):** author the richer detonate in `vfx.ron`, reusing existing verbs; extend the load test. Independent of Seam A except both touch `vfx.ron`. Visual quality is K001 (manual `cargo winx` sign-off only).
- **Seam C (optional) — windowed wiring of variant selection:** only if the team wants the selected variant visible in `cargo winx`. Not required for the slice's headless contract.

### First proof / build order

1. **Seam A first.** Land `VfxContext` + variant map + `select_variant` + the deterministic headless test. This resolves the only real design unknown and makes the slice's CI-provable criterion green. Run `cargo test --test animation`.
2. **Seam B.** Enrich `baby_burner.detonate` in `vfx.ron`; extend `validate_effects` coverage + load test. Run `cargo test --test animation` + `cargo test --features windowed --test windowed_only`.
3. **Seam C (optional).** Wire selection into the windowed detonate spawn; verify `cargo build --features windowed`. Visual sign-off is a manual K001 step the user explicitly asked to perform.

## Recommendation (variant-selection shape)

Keep selection a **pure free function over data**, not a new registry axis — consistent with D035's "only the vocabulary is closed data; dispatch stays open" and D033's deferral of the variation/expression layer:

- `VfxContext { skill_id: String, variant_key: String }` (closed, render-free scalars; extensible later without breaking the seam). Carries no gameplay numeric (R012).
- Schema: a `variants` map keyed by `(skill_id, variant_key)` → `EffectId`, expressed RON-friendly as nested `BTreeMap<String, BTreeMap<String, EffectId>>` (deterministic ordering per R004; avoids tuple-key RON awkwardness). Add it as a top-level optional `VfxAsset` field (`#[serde(default)]`) so existing assets keep loading.
- `select_variant(asset, ctx) -> Option<&EffectDef>`: pure, deterministic, returns `None` for an unmapped `(skill_id, variant_key)` (caller falls back to the base effect — mirrors `resolve_effect`'s None-not-panic discipline).
- Extend `validate_effects`: every variant target `EffectId` must resolve to a present effect (new `DanglingVariant` error variant, deterministic BTreeMap order — same pattern as `DanglingOnExpire`, MEM076).

This satisfies "selection lets a variant be a wholly different `on_expire` chain" (D033) because a variant simply points at a different base `EffectId` whose own `on_expire` chain differs — no field-patching, no modifiers needed for S03 (modifiers are D033 deferred-additive sugar; do not build them now — YAGNI with one Digimon).

## Don't Hand-Roll

- **Curve eval / placement math** — `eval_scale`/`eval_color` (vfx_asset.rs) and the four placement verbs (placement.rs) already exist and are tested for 1000-call bit-identical determinism. The enriched detonate must reuse them, not introduce parallel math.
- **Validation pattern** — extend the existing `validate_effects` + `VfxValidationError` enum (return first offender in BTreeMap order, as data, never panic). Do not invent a second validation path.
- **Load + resolve plumbing** — `RonAssetPlugin::<VfxAsset>`, `AgumonVfx` handle resource, `resolve_effect`, `spawn_effect_by_id` are all in place (MEM071/MEM077). Selection slots in front of `spawn_effect_by_id`, it does not replace the spawn machinery.

## Key Risks / Constraints

- **Scope creep from M004-CONTEXT.** CONTEXT lists HDR/Bloom/additive rendering, brand-new Sharp Claws VFX, and a 3-way texture spike as in-scope for the *milestone* — but these post-date the S01-S03 roadmap and belong to roadmap-reassessment slices, **not S03 as currently defined**. S03's roadmap line is strictly variant-selection + Baby Burner. Keep this slice to that; flag the reassessment as a milestone-level concern (the CONTEXT itself says the roadmap "almost certainly needs reassessment").
- **Visual quality is K001 — not CI-assertable.** Auto-mode can never run `cargo winx`. The deterministic variant-selection test + the grep-guard are the entire CI-provable surface of S03. The "Baby Burner looks good" bar is the user's manual sign-off only.
- **Bounce stays out of the FSM (MEM061).** If detonate enrichment ever implies re-entry/chaining, express it via data `on_expire`, never an anim_graph self-loop.
- **No real unlock system.** `select_variant` must be provable with a *synthetic* `VfxContext`; do not attempt to source `variant_key` from gameplay state (out of scope, D033 rationale).

## Skills Discovered

- `bevy-ecs-expert` (already installed) — relevant for any windowed-system wiring in optional Seam C.
- `rust-skills` (already installed) — applied: `err-result-over-panic` (selection returns `Option`/validation returns `Result`, no panics in load/render), `api-non-exhaustive` consideration for `VfxValidationError` if it gains a variant. No new skills needed; `npx skills find` not run (no novel external tech in this slice).

## Verification

- Headless: `cargo test --test animation` — new `vfx_variant_selection` determinism test (synthetic `VfxContext` → stable `EffectId`), extended `validate_effects` cases (dangling variant target named), enriched-detonate round-trip/load test. Existing `render_no_vfx_kind_guard::render_rs_has_no_vfx_kind_dispatch` continues to assert the no-hardcoded-paths criterion.
- Headless build clean: `cargo build` (no windowed leak, R016).
- Windowed: `cargo build --features windowed` + `cargo test --features windowed --test windowed_only` (detonate spawn contract still resolves).
- Visual (manual, K001): user reviews Baby Burner detonate in `cargo winx` and signs off — the only valid proof of the "looks good" bar.

## Sources

- `src/animation/vfx_asset.rs`, `src/animation/placement.rs`, `src/combat/runtime/registry.rs`, `src/windowed/render.rs` (read directly).
- `assets/digimon/agumon/vfx.ron` (baby_burner.detonate stub at lines 135-161).
- `.gsd/DECISIONS.md` D033 (graft 5 variant selection), D034 (editor-ready), D035 (closed PlacementParams), D036 (axis placement).
- Memory: MEM044 (closed-enum/opaque-id command surface, R012), MEM071/MEM072/MEM077 (data-path render + load), MEM076 (validate_effects pattern), MEM061 (bounce stays out of FSM).
- `.gsd/milestones/M004/slices/S02/S02-SUMMARY.md` (what S02 already delivered).
