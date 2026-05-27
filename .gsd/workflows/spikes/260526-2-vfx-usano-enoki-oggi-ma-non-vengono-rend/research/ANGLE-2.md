# Angle 2 — Renamon presentation and skill-selection path

## Question

Why does Renamon fail to show its sprite/presentation correctly, and why can the UI fail to offer/select Renamon skills even though Renamon data exists?

## Findings

### 1. Renamon data is present

Renamon is not missing from authored content:
- `assets/data/digimon/renamon/unit.ron` defines Renamon + Kyubimon
- `assets/data/digimon/renamon/skills.ron` defines `diamond_storm`, `renamon_ult`, follow-up skills, etc.
- `src/windowed/digimon/renamon/mod.rs` registers:
  - stance path
  - skill-start node for `diamond_storm`
  - sprite presentation entry
  - windowed demo entry

So the failure is **not** “Renamon assets do not exist”.

---

## A. Renamon sprite/presentation: likely root cause is graph-registry population

### 2. Fresh runtime evidence shows the Renamon sprite path stalls before spawn

A filtered windowed validation run showed:
- both Agumon and Renamon atlases were built
- then: `stance graph not yet readable; sprite spawn deferred` for `UnitId(7)` / `presentation_id="renamon"` / `graph_id="renamon_stance"`
- after that, only Agumon playback ticks appeared

This is strong evidence that the Renamon sprite is not failing because of atlas image load. It is failing because the Renamon **stance graph is not available through the runtime registry when sprite spawn runs**.

### 3. `populate_graph_registries` has a structural early-return bug

`src/animation/registry.rs::populate_graph_registries` reads animation graph asset events, inserts the first matching graph into either the skill or stance registry, and then immediately `return`s.

That means one system pass processes at most **one** matching graph insertion before exiting.

Why this matters here:
- the app loads multiple graphs at boot (Agumon skill, Renamon skill, Agumon stance, Renamon stance)
- Renamon stance is additionally appended dynamically through `AnimationStancePaths`
- if only the first matching graph per pass gets inserted, later graphs can be delayed or starved
- `spawn_unit_sprites` depends on `StanceGraphRegistry::resolve_snapshot("renamon_stance")`
- if that registry entry is missing, Renamon never spawns

This matches the live warning exactly.

### 4. Secondary symptom: Renamon skill graph lookup is also at risk

The same registry-population bug can affect not just the stance graph, but also the Renamon skill graph. Even if the sprite issue is fixed first, the skill presentation path may still fall back or misbehave until graph registration is made deterministic for all graphs.

---

## B. Renamon skill selection in the combat panel: direct root cause found

### 5. The combat panel reads an arbitrary `SkillBook`, not the aggregated one

`src/ui/combat_panel/render.rs` does this:
- `skill_books.iter().next()`

That means the UI uses the **first asset in `Assets<SkillBook>`**, not the assembled `SkillBookHandle` aggregate that the rest of the app expects.

Why this is a bug:
- the data layer loads many per-digimon partial skill books
- the data plugin later assembles one merged `SkillBook` and exposes it via `SkillBookHandle`
- asset iteration order is not a safe contract for “the canonical skill book”
- if the first iterated partial book does not include Renamon’s skills, the UI asks legality questions against the wrong book

### 6. That produces `MissingSkill`, which disables the action button

`query_action_affordance` returns a disabled affordance with `LegalityReasonCode::MissingSkill` when the requested skill is absent from the provided `SkillBook`.

So this one line in the combat panel can directly explain:
- skill buttons present but disabled, or
- inconsistent skill availability depending on whichever `SkillBook` asset is visited first

### 7. There is already an in-repo example of the correct pattern

`src/ui/combat_panel/preview_cache.rs` fetches the canonical skill book via:
- `SkillBookHandle`
- then resolves that handle inside `Assets<SkillBook>`

So the project already contains the intended access pattern. The combat panel simply diverges from it.

---

## C. Additional Renamon presentation gap: skill VFX are authored but unregistered

### 8. Renamon’s skill graph emits a particle cue that nothing registers

`assets/digimon/renamon/anim_graph.ron` contains:
- `SpawnParticle(name: "diamond_storm_leaf", ...)`

But `src/windowed/digimon/renamon/mod.rs` only registers:
- stance path
- skill start node
- sprite presentation
- demo entry

It intentionally registers **no**:
- `EnokiVfxRegistry`
- `OnEnterEffectRegistry`
- `SkillReleaseEffectRegistry`
- `DetonateEffectRegistry`

This absence is even enforced by `tests/windowed_only/renamon_extension_contract.rs`.

Result:
- the Renamon skill graph can enter the cast node
- the particle name has no effect-id mapping
- `advance_digimon_presentation` resolves that particle name to `[]`
- no Renamon particle effect spawns

This is not the same as the skill-selection bug, but it is another real Renamon presentation hole.

## Evidence

- `assets/data/digimon/renamon/unit.ron`
- `assets/data/digimon/renamon/skills.ron`
- `src/windowed/digimon/renamon/mod.rs`
- `src/animation/registry.rs::populate_graph_registries`
- `src/windowed/render.rs::spawn_unit_sprites`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/preview_cache.rs`
- `src/combat/action_query/legality/action.rs::query_action_affordance`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/windowed_only/renamon_extension_contract.rs`
- filtered windowed validation logs showing Renamon stance-graph spawn deferral

## Conclusion

There are **two high-confidence Renamon bugs** and one additional presentation gap:

1. **Sprite/presentation bug:** Renamon stance graph is not reliably present in `StanceGraphRegistry` when sprite spawn runs; `populate_graph_registries` is the strongest code-level root cause.
2. **Skill-selection bug:** the combat panel uses an arbitrary `SkillBook` asset (`iter().next()`) instead of the canonical aggregated `SkillBookHandle`, which can disable Renamon skills as `MissingSkill`.
3. **Skill-VFX gap:** `diamond_storm_leaf` is authored in the skill graph but has no runtime effect registration, so even a selectable Renamon skill would currently spawn no Renamon particle VFX.

## Confidence

**High** for (1) and (2). **High** for the missing `diamond_storm_leaf` registration gap as well.