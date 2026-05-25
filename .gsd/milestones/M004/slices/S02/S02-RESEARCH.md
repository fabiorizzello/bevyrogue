# S02 Research: Placement verbs in Registry + generic render dispatcher

**Lane:** research. **Depth:** targeted (known local patterns — `Registry<E>` and the data-path branch already exist from S01 — but the *full* enum removal + a new typed-param ext axis is new work with one genuine open design question).

## Summary

S02 closes the loop S01 opened. S01 proved the data path for **one** effect (`baby_flame.impact` shard fan-out) while leaving `VfxParticleKind` + `kind_from_name` (`vfx_particle_kind`) and all per-kind helper fns in place as a fallback. S02's contract (roadmap demo line) is: **(1)** Baby Flame charge ember-swirl and fast launch render through **Registry-resolved placement verbs**, and **(2)** a static grep confirms `VfxParticleKind` and `kind_from_name` no longer exist in `render.rs`.

Meeting criterion (2) is the forcing function: removing the enum removes everything keyed off it — not just the two named motions, but `vfx_particle_ttl/size/anchor`, the per-kind color match, the per-kind texture match, and the hardcoded `projectile→impact` chain. **All five Baby Flame effects** (`charge`, `ember`, `projectile`/launch, `impact`, `impact_shard`) must therefore be fully expressed in `assets/digimon/agumon/vfx.ron` and driven through one generic dispatcher. So S02 is "port the rest of Baby Flame + delete the enum", not just "two more verbs."

The seam is well-understood (it mirrors S01's `Some(effect) => data / None => fallback` branch, generalized and with the `None` fallback deleted). The **one real open question** is the shape of *typed, editor-ready placement-verb parameters* (D034) under an *open* registry dispatch (D033 "no closed match in the core") — these two constraints are in mild tension and the planner must pick the param-passing strategy. See **Key Design Tension** below.

Note S02 is **placement + dispatcher only**. The roadmap defers Baby Burner detonate + variant selection to S03. The milestone CONTEXT's expanded scope (HDR/bloom/additive rendering, brand-new Sharp Claws VFX, texture spike) is **not in the S01–S03 roadmap** and is flagged there as needing reassessment — do **not** pull it into S02 unless the roadmap is reassessed first.

## Requirements this slice owns/supports

- **R004 (determinism):** placement-verb math must be pure, seeded, snapshot-stable, headless-tested. The verb fns must take no `World`/render types — only a pure context (age/ttl/progress/phase + caster/target anchors + typed params) → `[f32; 2]` offset.
- **R002 / R005 (headless-first / dep gating):** verb fns + param structs live in the **headless lib** (no `windowed`); only the dispatcher glue (resolving `Res<ExtRegistries>` and applying offsets to `Transform`) is `#[cfg(feature = "windowed")]`. S01 confirmed this split keeps `cargo build` (headless) clean (R016).
- **D034 (editor-ready):** each placement verb's parameters are a typed struct deriving `Serialize + Deserialize + Reflect` — **not** a stringly-typed map. `Registry::iter()` (`registry.rs:59`) already gives verb enumeration for free.
- **Error-handling strategy (CONTEXT):** a placement-verb id in `vfx.ron` not present in the registry, and a malformed/absent param payload, must surface as an explicit **load-time validation error** (which Digimon / effect / verb), not a panic in the render loop and not a silent mis-render.

## Implementation Landscape (files + purpose)

**The generic registry seam (extend here):**
- `src/combat/runtime/registry.rs` — the D031 `Registry<E>` (`:28-68`: `register`/`get`/`iter`) and `ExtRegistries` resource (`:184-196`). Add a **new ext axis** here, e.g. `PlacementExt` with a pure `type Fn` signature, and a `pub placements: Registry<PlacementExt>` field on `ExtRegistries`. Pattern to copy: any existing axis (`SelectorExt:81-84`, `CueExt:114-117`).
- `src/combat/blueprints/agumon/mod.rs` — `register_agumon_ext(regs)` (`:36-51`) is where Agumon's ext fns register today. Add `regs.placements.register("agumon/baby_flame/converge_inward", ...)` etc. here. Novel-motion criterion = one `register(...)` line here, zero core change.

**ExtRegistries reaches the windowed app — confirmed:**
- `src/main.rs:59` adds `CombatPlugin`; `src/combat/plugin.rs:29` inits `ExtRegistries`, `:57-60` registers kernel builtins + `register_blueprints(app)` (which runs `register_agumon_ext`). So the windowed render system can take `Res<ExtRegistries>` and resolve placement fns by id. (The windowed `RenderPlugin` is added separately at `main.rs` / `windowed/mod.rs:201`, but the same `App` holds `ExtRegistries`.) **This is the key enabler — verify the planner does not assume a separate VFX registry is needed.**

**The schema to extend (S01 left it minimal):**
- `src/animation/vfx_asset.rs` — `Placement { verb: String }` (`:64-69`) currently carries **no params**. `Appearance` (`:75-86`) carries `count/spread_px/ttl_ticks/scale/color` but **no anchor, no per-particle size, no texture/sprite id**. Removing `VfxParticleKind` removes `vfx_particle_size/anchor` and the texture match, so those concerns must move into the typed schema (e.g. `Appearance.size_px`, an `anchor: Option<...>`, a `texture: ...` id). `resolve_effect` (`:137`), `spawn_plan` (`:143`), `eval_scale`/`eval_color` (`:157`,`:172`) are reusable as-is.

**The windowed file to gut (`src/windowed/render.rs`, ~2045 lines):**
- **Delete:** `VfxParticleKind` enum (`:49-61`); `vfx_particle_kind` (`:863-872`); `vfx_particle_ttl` (`:874-884`); `vfx_particle_size` (`:886-895`); `vfx_particle_anchor` (`:897-907`); per-kind color match (`:1068-1085`); per-kind texture match (`:1088-1101`); `baby_flame_ember_offset/alpha` + `baby_flame_shard_offset/alpha` (`:948-991`, the math moves into pure placement verbs/curves); `spawn_baby_flame_embers` (`:994-1019`); the per-kind motion match in the update loop (`:1341-1368`); the kind-keyed shard branch + its `None` fallback (`:1388-1423`, generalize the `Some(effect)` half); the kind-keyed `projectile→impact` chain (`:1428-1436`, replace with `on_expire` data); the kind-keyed charge/ember despawn-on-release (`:750-761`).
- **Re-shape:** `VfxParticle` component (`:37-47`) drops `kind: VfxParticleKind`; instead carries the resolved effect id (and/or a resolved placement-verb id + its params, plus `phase`/`anchor` derived from data).
- **Keep / unchanged (release seam — D031/D032, do NOT touch):** the FSM cue/barrier release path around `:735-818` (`barrier.request_release`, `fire_kernel_cue`); `VfxMotion`/`VfxLocus`/`VfxSpawnDescriptor` (`src/animation/anim_graph.rs:343-355`, `src/animation/vfx.rs`) — these are the *authored-timeline* `Command::SpawnParticle` release path, **separate** from vfx.ron's per-tick placement verbs. The dispatcher replaces only *kind resolution*, per the milestone constraint.
- `AGUMON_IMPACT_EFFECT_ID` (`:338`) is the existing single hardcoded effect id — generalize to all five effect ids.

**The asset to grow:**
- `assets/digimon/agumon/vfx.ron` — currently 2 effects (`baby_flame.impact`, `baby_flame.impact_flash`, per S01 summary). Add `baby_flame.charge`, `baby_flame.ember`, `baby_flame.projectile` (launch arc), wiring `projectile`→`impact` via `on_expire` (replacing the hardcoded burst at `render.rs:1428`). Each needs its placement verb id + typed params + appearance (ttl, size, anchor, texture, scale/color curves) reproducing the current hardcoded constants (`:182-198`) so the visual is preserved as the starting point for the user's K001 review.

## Key Design Tension (planner must resolve — consider `design-an-interface`)

D033 says placement dispatch must have **"no closed match in the core"** (open registry by string id). D034 says params must be **typed structs + Reflect** (not a string map). Reconciling *open dispatch* with *typed params* is the one non-trivial decision. `Registry<E>` stores one uniform `E::Fn` signature per axis, so the verb fn must receive its params through that uniform signature. Candidate strategies:

- **(A) Closed `PlacementParams` enum in the lib.** `Placement { verb: String, params: PlacementParams }` where `PlacementParams` is a typed enum (`ConvergeInward { radius_px, omega }`, `FanOut { spread_px }`, `ArcLaunch { .. }`, `Static`). Verb fn reads `&PlacementParams`. *Pro:* simplest, fully `Reflect`-able, compiler-validated, deterministic. *Con:* a novel motion adds an enum variant — a small **core change**, mildly violating the "RON-only/one-register, no core change" criterion. Lowest-risk MVP for one Digimon.
- **(B) Reflect-erased params (`Box<dyn Reflect>` / `DynamicStruct`).** Verb fn downcasts via `FromReflect` to its own param struct **defined in the blueprint crate**. *Pro:* truly open — novel verb = register fn + param struct in the blueprint, **zero core change**; matches D033's extension-first intent and the deferred expression-tree-as-a-verb note. *Con:* RON→`DynamicStruct`→`FromReflect` plumbing, fallible downcast ⇒ this is exactly the load-time validation error the CONTEXT mandates; most complex.
- **(C) Shared fixed param fields on the effect.** A small typed shared set (`radius_px`, `omega`, `spread_px`, …) read by verbs. *Pro:* trivial. *Con:* not extensible for arbitrary novel params; a typed regression toward the param-bag D034 rejected.

**Recommendation:** start from **(A)** as the MVP — it satisfies R004 + D034 (editor-ready) with the least risk and one Digimon on screen, and the registry is the same seam **(B)** would later evolve into additively (D033 explicitly designed both as additive). Flag to the user/planner that (A) trades a tiny core-change cost on *novel* motions; if they want the pure "zero core change for novel motion" guarantee now, pick **(B)**. Either way, verb **resolution** stays open (string id via `Registry::get`), so the grep-removal criterion and the "reuse = RON-only" criterion both hold for (A).

## Natural Seams (independent work units)

1. **Core axis** — add `PlacementExt` + `placements` field to `ExtRegistries`; define the pure verb signature + the chosen param representation; unit-tests for the registry round-trip. (Unblocks everything; smallest.)
2. **Pure placement verbs (lib)** — port `converge_inward` (ember swirl, from `:948-957`), `fan_out` (shard, from `:972-981`), `arc_launch` (projectile lerp, from `:1371-1385`), `static`. Pure fns + R004 determinism tests. Register them in the Agumon blueprint.
3. **Schema extension** — grow `Placement`/`Appearance` for params + anchor + size + texture; round-trip + `deny_unknown_fields` + Reflect-introspection tests; load-time validation (verb id resolvable, params well-formed).
4. **Asset authoring** — write the three new effects + `on_expire` chain into `vfx.ron`; headless load tests asserting presence + spawn-plan + sampled curve eval (mirror S01's `include_str!` tests).
5. **Render dispatcher** — rewrite `render.rs` to resolve placement verb from `Res<ExtRegistries>`, drive position/appearance from data for *all* effects, re-shape `VfxParticle`, delete the enum + all per-kind fns, drive chaining from `on_expire`. (Largest; depends on 1–4.)
6. **Grep guard test** — a test asserting `VfxParticleKind` and `kind_from_name`/`vfx_particle_kind` are absent from `render.rs` (criterion 2 made CI-provable).

Seams 1–4 are largely independent and headless; 5 integrates them; 6 is trivial and gates the removal.

## First Proof (highest-risk unblocker)

Seam **1 + 2**: stand up the `PlacementExt` axis with the chosen param representation and port **`converge_inward`** as the first pure verb with a determinism test, registered in the Agumon blueprint and resolvable via `ExtRegistries::placements.get("agumon/baby_flame/converge_inward")`. This proves the open-dispatch + typed-param design end-to-end in the headless lib before any `render.rs` surgery, and resolves the Key Design Tension concretely (a compiling verb fn signature is the real answer to "(A) vs (B)").

## Don't Hand-Roll / Reuse

- `eval_scale`/`eval_color`/`eval_curve` (`vfx_asset.rs:157-234`) — keep; the appearance axis is done.
- `resolve_effect`/`spawn_plan` (`:137-149`) — reuse for all effects, not just impact.
- S01's `Some(effect) => data / None => fallback` branch (`render.rs:1388-1423`) is the **template** for the generalized dispatcher — but the `None` fallback arm is **deleted** in S02 (no hardcoded path may remain).
- The `register("ns/name", fn)` idiom (`agumon/mod.rs:36-51`) — copy verbatim for placement verbs.
- `MEM072` (data-driven windowed rendering with headless fallback) and `MEM073` (sorted-indices curve eval) — established patterns; S02 generalizes MEM072 and removes its fallback arm.

## Verification

Headless (CI, R004 — auto-mode safe):
- `cargo test --test animation` — placement-verb determinism, schema round-trip/Reflect, vfx.ron load + on_expire chain resolution, curve eval at sampled progresses.
- Grep-guard test (seam 6): `VfxParticleKind` / `kind_from_name` / `vfx_particle_kind` absent from `render.rs`.
- `cargo build` (headless) — confirms verbs/params/schema stay lib-side, no windowed dep leak (R016/R005).
- `cargo build --features windowed` — dispatcher compiles.
- `cargo test --features windowed --test windowed_only` — extend the S01 `vfx_asset_impact` contract tests to cover the generalized dispatcher's lib contract.

Visual (manual, **K001 — auto-mode must NOT run this**): `cargo winx` user sign-off that charge ember-swirl + fast launch look right. Auto-mode certifies only the headless half; the visual half is the user's explicit review step.

## Skills

- `bevy-ecs-expert` — `Registry<E>`/`ExtRegistries` resource access in a windowed system (`Res<ExtRegistries>`), component re-shaping.
- `rust-skills` / `rust-development` — pure-fn design, enum-vs-trait-object dispatch (relevant to the (A)/(B) param decision), `Reflect`/`FromReflect` if (B).
- `design-an-interface` — recommended to resolve the Key Design Tension (param-passing under open dispatch) before seam 5; D033/D034 themselves came from a four-design pass.
- `tdd` — verbs have a clean observable contract (`PlacementCtx → [f32;2]`); red-green is natural for seams 1–4.

No new external libraries needed (`bevy_reflect` already a resolved dep per S01; `bevy_common_assets` `RonAssetPlugin` already wired). No `npx skills find` install required.

## Risks / Watch-outs (Forward Intelligence)

- **Enum removal is all-or-nothing for criterion 2.** You cannot port "charge + launch" and leave `impact`/`projectile` on the enum — the grep must come back empty. Budget for porting **all five** effects + deleting ~8 helper fns + re-shaping `VfxParticle`.
- **`on_expire` must replace the hardcoded `projectile→impact` burst** (`render.rs:1428-1436`). S01 modeled `impact_flash` as a *sibling* rather than a chain "to keep the tracer bullet minimal" (S01 summary) — S02 must actually exercise `on_expire` for the projectile→impact handoff, so the chaining code path gets its first real use here.
- **Appearance schema must absorb anchor + size + texture.** These are currently enum-keyed (`vfx_particle_anchor:897`, `vfx_particle_size:886`, texture match `:1088`). Don't forget them when extending `Appearance`, or the dispatcher loses information the enum carried.
- **Param/dispatch tension is real, not cosmetic** — picking (A) vs (B) changes the schema shape (`Placement` struct) and the verb signature. Decide before seam 3/5, or risk a schema refactor mid-slice.
- **Charge appearance is more than placement** — the charge quad's growth/pulse scale+alpha (`render.rs:1342-1356`) is appearance-curve territory, not placement. Map it onto `scale`/`color` curves in the asset, not a placement verb.
- **`emitter_ticks` ↔ anim_graph coupling** (CONTEXT) is flagged for load-time validation but is more central to S03 timing; for S02 keep ttl/spread reproducing the current constants so the visual baseline is unchanged for K001 review.
- **Do not expand into S03 / expanded-scope work** (Baby Burner, variant selection, HDR/bloom/additive, Sharp Claws, texture spike). The roadmap predates that scope; reassess the roadmap before pulling any of it forward.

Slice S02 researched.
