# Recommendation — VFX / Renamon UI / reaction feedback spike

## Executive summary

The evidence points to **multiple presentation bugs, not one single shared root cause**.

Two issues are already high-confidence and directly actionable:
1. **Renamon sprite/presentation** is blocked by unreliable animation-graph registry population. The live validation run shows Renamon’s atlas loads, but `renamon_stance` is still unreadable when sprite spawn runs, and only Agumon playback appears afterward.
2. **Renamon skill selection** can fail because the combat panel uses an arbitrary `SkillBook` asset via `skill_books.iter().next()` instead of the canonical aggregated `SkillBookHandle`. That can surface Renamon actions as `MissingSkill` even though Renamon’s authored skills exist.

The Agumon/Enoki issue is narrower than it first looked: current evidence does **not** support “Agumon assets are missing” or “Enoki is not wired.” The remaining gap is a **runtime cast/spawn/visibility seam that the current tests do not exercise**. Meanwhile, the hurt/dead/flash/shake complaints are partly downstream of missing presentation state (Renamon sprite absent) and partly explained by an explicit current design rule: hurt reactions only fire while the target sprite is idle.

## Comparison matrix

| Area | Current evidence | Most likely root cause | Confidence | User-facing impact | Best first fix |
|---|---|---|---|---|---|
| Renamon sprite missing | Live validation logs: atlas built, then `renamon_stance` unreadable, sprite spawn deferred; only Agumon playback ticks appear | `populate_graph_registries` early-returns after first inserted graph, so later stance/skill graphs can be delayed/starved | High | Renamon absent visually; Renamon hurt/death/flash impossible to judge | Fix graph-registry population first |
| Renamon skills not selectable | UI uses `skill_books.iter().next()` instead of `SkillBookHandle`; legality layer returns `MissingSkill` when book lacks the skill | Combat panel queries affordances against the wrong skill book | High | Skill buttons can be disabled or inconsistent | Use canonical aggregated `SkillBookHandle` in combat panel |
| Renamon skill VFX missing | `diamond_storm_leaf` is authored in the skill graph, but Renamon registers no `OnEnterEffectRegistry` / Enoki mapping | Unregistered Renamon particle cue | High | Even after skill selection works, Renamon cast will have no particle effect | Add Renamon effect registration or remove the cue until implemented |
| Agumon Enoki not visible | Enoki plugin present, assets parse, load request logged, no load-failure warning; but no live cast-driven proof | Runtime trigger/spawn/visibility seam still unproven | Medium | Agumon skills feel visually broken | Add cast-driven smoke proof + louder spawn-miss diagnostics |
| Hurt animation feels wrong | Code explicitly skips hurt if target sprite is mid-cast | Current behavior is an intentional scope guard, not random failure | High | Hits during casts look like “hurt doesn’t work” | Product decision: keep or relax idle-only hurt |
| Death / flash / shake feel wrong | Structural wiring exists and tests pass at projection level; Renamon sprite failure blocks visibility on that unit | Likely secondary symptom + missing live proof | Medium | Visual feedback seems absent or inconsistent | Re-evaluate only after Renamon spawn/UI fixes |

## Primary recommendation

### Fix order

#### 1. Repair animation graph registry population
**Why first:** it is the clearest live runtime failure and directly explains the missing Renamon sprite.

Target area:
- `src/animation/registry.rs::populate_graph_registries`

Expected outcome:
- all skill + stance graphs populate deterministically
- Renamon sprite can spawn
- Renamon stance/skill presentation becomes testable in the real runtime

#### 2. Make the combat panel use the canonical aggregated skill book
**Why second:** this is the clearest explanation for “Renamon cannot select skills.”

Target area:
- `src/ui/combat_panel/render.rs`

Expected outcome:
- UI queries legality against the merged skill book, not an arbitrary partial asset
- Renamon skill buttons stop failing as `MissingSkill` for data that already exists

#### 3. Close the authored-but-unregistered Renamon particle cue gap
**Why third:** once sprite spawn and skill selection are fixed, Renamon will still have a missing cast effect because `diamond_storm_leaf` has no runtime mapping.

Target areas:
- `assets/digimon/renamon/anim_graph.ron`
- `src/windowed/digimon/renamon/mod.rs`
- possibly new Renamon particle assets / effect-id mapping

Expected outcome:
- Renamon cast presentation becomes internally consistent

#### 4. Only then isolate the remaining Agumon Enoki live-cast issue
**Why fourth:** Agumon’s problem is real, but current evidence says the asset/backend layer is already in place. The missing proof is runtime spawn/visibility.

Best next step:
- add a cast-driven windowed smoke proof that demonstrates a `ParticleSpawner` appears for a real Agumon cast
- add warn-once diagnostics when `spawn_effect_by_id` returns `0`

This is a better use of time than rewriting Agumon assets blind.

#### 5. Revisit hurt/dead/flash/shake after presentation surfaces are stable
**Why last:** some of the feedback complaint is probably downstream noise from the missing Renamon sprite and incomplete live-cast proof.

Then make a product call:
- keep hurt as idle-only, or
- allow hurt to interrupt selected in-flight states

## Alternative recommendation if Agumon VFX is the urgent visible blocker

If the immediate goal is “make Agumon feel alive again before touching Renamon,” do this short path:
1. add warn-once diagnostics for runtime spawn misses
2. add a cast-driven Agumon smoke proof
3. confirm whether `ParticleSpawner` entities appear
4. only then touch Agumon render logic

Tradeoff:
- faster investigation of the symptom the user feels first
- but leaves two already-confirmed Renamon defects in place

## What would change this recommendation

The recommendation would change if a cast-driven proof shows one of these:
- Agumon spawners are created correctly but never rendered → then the highest-priority bug becomes Enoki live render visibility/material state
- Renamon sprite appears consistently after a few frames in normal human play → then the graph-registry bug is a timing/validation nuisance rather than the main UX blocker
- the combat panel is already reading the canonical aggregated skill book in practice through some external invariant → then the Renamon skill-selection issue needs a second pass

## Next steps if accepted

1. Fix `populate_graph_registries` so it processes all matching graph events.
2. Change combat-panel skill-book lookup to use `SkillBookHandle`, matching `preview_cache` and the rest of the runtime.
3. Decide whether Renamon gets a real `diamond_storm_leaf` Enoki path now, or whether the cue should be removed until implemented.
4. Add a cast-driven Agumon presentation smoke proof and spawn-miss diagnostics.
5. Re-test hurt/dead/flash/shake after the above fixes, then decide whether idle-only hurt should remain.

## Final call

**Recommended path:**
- treat this as **three bugs plus one design limitation**, not one shared regression
- fix **Renamon graph registration** and **combat-panel skill-book lookup** first
- then close the **Renamon particle registration gap**
- finally instrument and prove the **Agumon live Enoki spawn** path before changing assets or renderer behavior

That gives the highest-confidence repair order with the least guesswork.