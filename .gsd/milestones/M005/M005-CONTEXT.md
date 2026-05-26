# M005: Combat visual feedback completion (reactions + enoki VFX)

**Gathered:** 2026-05-26
**Status:** Ready for planning

## Project Description

Close the gap between the combat **logic** (already complete) and the combat **presentation** so a full encounter can be watched end-to-end on screen. Two tracks: (1) **reactions** — wire the already-emitted `OnHitTaken` and `UnitDied` events to the already-authored stance `hurt`/`death` nodes so a struck unit visibly flinches and a defeated unit visibly dies, plus on-hit feedback (damage flash / small shake, floating damage numbers on the pixel canvas); (2) **VFX rework** — replace the current flat-quad particle system with `bevy_enoki`, a real GPU 2D particle backend, authored in `.particle.ron`, because the existing minimal spawn surface produces VFX the user calls "pietà" (dreadful). The enemy stays an Agumon dummy this milestone; because every unit renders through the shared Agumon sprite/stance graph, wiring hurt/death gives a visible reaction on **both** sides for free.

## Why This Milestone

The kernel is rich and 6 blueprints exist, but the encounter can't be evaluated visually: a hit lands (damage numbers move in the egui panel) yet the struck sprite does nothing, and a unit at 0 HP keeps standing because nothing consumes `UnitDied`. The user cannot "test the combat completely" until the struck/defeated states are visible. Separately, the M004 VFX seam is intentionally a one-shot spawn of static colored quads with closed-form motion — no emission-over-time, no real additive blending, no per-particle animation — which is why it reads as placeholder. The user proposed adopting `bevy_enoki` to get a genuine particle authoring surface. Both are presentation-layer; the gameplay contract is untouched.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Launch `cargo winx`, run an encounter, and **see the struck unit flinch** (stance `hurt` node) on every hit — on both the attacker-target and the dummy.
- See a unit **visibly die / leave the field** when it reaches 0 HP (stance `death` node + sprite fade/removal), instead of standing post-KO.
- See **hit feedback** on the shared sprite — a damage flash and small shake — and floating damage numbers rendered on the pixel canvas (today they appear only in the egui side panel).
- See Agumon's skill VFX (Sharp Claws, Baby Flame, Baby Burner) rendered through `bevy_enoki` and sign off that they look meaningfully better than the current flat-quad placeholder.

### Entry point / environment

- Entry point: `cargo winx` (== `cargo run --features windowed`) for the visual surface; `cargo test` / nextest `agent` profile for the headless contract.
- Environment: local dev (windowed egui/winit for visuals; headless for CI-provable math).
- Live dependencies involved: none. `bevy_enoki` is a new windowed-gated crate; particle assets are local `.particle.ron` + texture files.

## Completion Class

- **Contract complete means:** the event→stance-reaction mapping is a pure, headless-tested function (mirrors the R009 typed `AnimGraphInput` purity pattern): `OnHitTaken` → hurt role, `UnitDied` → death role, deterministic and snapshot-stable, with no windowed dependency. A static dep-gating test proves `bevy_enoki` stays behind `#[cfg(feature = "windowed")]` (R005).
- **Integration complete means:** in `cargo winx`, hurt fires on every hit and death fires at 0 HP through the existing `EventReader<CombatEvent>` consumer path without the UI mutating combat state (R010); the enoki spawn replaces the old quad path at the existing `spawn_effect_by_id` seam without touching the FSM cue/barrier release (D031/D032).
- **Operational complete means:** none beyond the above — presentation milestone, no unattended lifecycle.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Headless (CI-provable): the reaction-mapping function maps `OnHitTaken`/`UnitDied` to the correct stance roles deterministically; dep-gating test confirms no enoki/winit/wgpu leak outside `windowed`; full `cargo test` + `cargo test --features windowed` + `cargo build --features windowed` green.
- Visual (manual, K001): the user reviews a full encounter in `cargo winx` and confirms — flinch on every hit (both sides), death/fade at 0 HP, hit flash/shake + canvas damage numbers, and the enoki-rendered Agumon VFX looking better than today.
- What cannot be simulated: visual quality and the "I can now watch a fight" experience. Auto-mode cannot launch the windowed binary (K001); the user's manual sign-off is the only valid proof of the visual half.

## Architectural Decisions

### Reaction mapping is a pure lib function, consumed by the windowed renderer

**Decision:** Event→stance-reaction resolution (`OnHitTaken` → `hurt`, `UnitDied` → `death`) lives as a pure function in the library crate (alongside the existing typed `AnimGraphInput` seam), headless-tested; `render.rs` calls it to drive the bound stance graph on the struck/dead sprite.

**Rationale:** Integration tests link only against the lib crate, not the windowed binary (project gotcha). Putting the mapping in the lib gives the reactions a real deterministic contract instead of leaving them entirely behind K001. It also mirrors the R009 input-purity pattern already proven in M002.

**Alternatives Considered:**
- Drive stance transitions inline in `render.rs` from the event reader — rejected; unreachable from `tests/`, so the reaction logic would be K001-only with no headless net.

### Reactions ride the shared Agumon sprite; no per-species enemy assets

**Decision:** Both combatants keep rendering through the shared Agumon sprite + stance graph. Hurt/death/flash/shake are driven on whichever sprite is struck/killed; the enemy remains an Agumon dummy.

**Rationale:** The user confirmed the enemy can stay an Agumon dummy. Because the shared sprite already binds the stance graph (which has hurt/death nodes), wiring the reaction gives a visible flinch/death on both sides with zero new art. Per-species enemy atlases + enemy attack animations are heavy asset work and are explicitly out of scope.

**Alternatives Considered:**
- Author enemy-specific atlases/anim now — rejected by the user as out of scope for "completing the combat visually."

### Adopt bevy_enoki as the VFX backend (windowed-gated), spike before full migration

**Decision:** Add `bevy_enoki` (windowed-gated) as the particle backend, authoring effects in `.particle.ron`. Prove the integration on **one** Agumon effect first (spike slice) — dependency compat, seam fit at `spawn_effect_by_id`, and a K001 look check — before migrating all Agumon effects.

**Rationale:** `bevy_enoki 0.6` supports Bevy 0.18 and gives emission-over-time, real additive blending, color/scale-over-lifetime curves, and animated per-particle sprites — exactly what the current closed-form one-shot quad spawner cannot do. But it is a new dependency whose visual result is only K001-verifiable; a one-effect spike retires the integration and quality risk before the expensive full reauthoring.

**Alternatives Considered:**
- Keep tuning the existing flat-quad system — rejected; flat alpha quads with no additive blending cannot read as fire regardless of tuning (same root cause established in M004/D037).
- Migrate all effects in one slice — rejected; commits to the dependency before the visual payoff is confirmed.

### enoki authoring leaves the deterministic test boundary (accepted)

**Decision:** enoki effects are authored data verified only visually (K001); they do not get the headless `eval_color`/`eval_scale`/placement-verb math contract the current VFX system has.

**Rationale:** VFX rendering is already outside the deterministic test boundary and already K001-gated (accepted since M004). enoki moves the authoring fully into that already-manual zone. The FSM cue/barrier impact release stays in the existing deterministic seam — it is independent of the VFX backend — so the gameplay-critical timing contract is unaffected.

**Alternatives Considered:**
- Build a headless-testable shim over enoki params — rejected; speculative, and the impact-timing contract that actually matters lives in the FSM, not the particle backend.

---

> See `.gsd/DECISIONS.md` for the full append-only register of all project decisions.

## Error Handling Strategy

Reaction mapping is total over the event taxonomy: events that are not hit/death map to "no reaction" explicitly (no panic, no silent stance corruption). A hurt reaction arriving while a unit is already in `death` must not resurrect it — death takes precedence; the mapping encodes that ordering and it is headless-tested. The stance graph's existing `hurt → idle` / `death → Exit` transitions are reused, so a dropped or duplicated event degrades to "stays idle" rather than a stuck frame. enoki asset load failures (missing `.particle.ron`, bad schema) must fail loudly at load with a contextual error (which effect) rather than spawning nothing silently; the old quad path is retired only after the enoki path renders, so a load failure is visible, not invisible. Rendering failures remain visual-only and are caught by the manual `cargo winx` review.

## Risks and Unknowns

- **Visual quality and "watchability" are not CI-assertable (K001).** — The whole point of the milestone is the on-screen experience; auto-mode can never run the windowed binary, so the visual half self-certifies only through manual sign-off.
- **bevy_enoki ↔ Bevy 0.18 / existing render stack integration.** — 0.6 claims 0.18 support, but it must coexist with the existing HDR `Camera2d` + bloom + sprite atlas pipeline and the FSM-gated spawn seam; the spike slice exists to retire this before full migration.
- **enoki spawn vs the two-clock impact barrier.** — The impact-frame release (D031/D032) must keep driving *gameplay* timing; enoki only renders. Need to confirm the particle spawn hangs off the existing cue seam without the particle lifetime leaking into the kernel timeline.
- **Death-fade vs post-KO overshoot observability.** — M002 deliberately lets presentation complete without liveness branching (post-KO overshoot is observable via events). Removing/fading the sprite on `UnitDied` must not break that documented behavior or the multi-hit loop.
- **Full enoki migration shape is unknown until the spike lands.** — How many effects, whether textures are needed, and whether the old quad path is deleted or kept behind the seam depend on spike findings; the migration slice is a sketch.

## Existing Codebase / Prior Art

- `src/windowed/render.rs` — drives the stance graph for idle + skill cues and owns the current quad-based VFX spawn (`spawn_effect_by_id`, `advance_vfx_particles`); it consumes neither `OnHitTaken` nor `UnitDied` today. This is where reactions get wired and where the enoki spawn replaces the quad spawn.
- `src/combat/observability/events.rs` — defines `CombatEventKind::OnHitTaken` and `UnitDied`, both already emitted across the single-target, multi-target, bounce, and self-target pipeline paths.
- `assets/digimon/agumon/stance.ron` — already has `hurt` (frames 46–52) and `death` (14–22) nodes with transitions back to idle / Exit; the reaction target nodes already exist.
- `src/animation/anim_graph.rs` / `src/animation/player.rs` — typed `AnimGraphInput` / `AnimGraphRole` purity seam (R009) the reaction mapping function mirrors.
- `src/animation/registry.rs` — `StanceGraphRegistry` resolving the per-Digimon stance snapshot bound to the sprite.
- `assets/digimon/agumon/vfx.ron` + the M004 `VfxAsset`/verb seam — the current data-driven VFX path enoki replaces; the FSM cue/barrier release it hangs off (D031/D032) stays.
- `src/combat/observability/` — the `EventReader<CombatEvent>` consumer convention (R010) the reactions follow without mutating combat state.

## Relevant Requirements

- **R002 (headless-first)** — reaction mapping runs without `windowed`; enoki/egui/winit gated behind `#[cfg(feature = "windowed")]`.
- **R004 (determinism)** — the event→reaction mapping is pure, seeded, snapshot-stable.
- **R005 (dep gating)** — `bevy_enoki` and all particle/render deps stay windowed-gated; a static test enforces no leak.
- **R010 (UI from events)** — reactions are driven from `EventReader<CombatEvent>`; the presentation path never mutates `CombatState`.
- **R013 (failure visibility)** — reaction mapping is total; enoki load failures fail loudly with context.

## Scope

### In Scope

- Pure, headless-tested event→stance-reaction mapping (`OnHitTaken` → hurt, `UnitDied` → death, with death-precedence ordering).
- Windowed wiring: struck sprite flinches (hurt node), defeated sprite plays death node + fades/leaves the field.
- On-hit feedback on the shared sprite: damage flash + small shake.
- Floating damage numbers rendered on the pixel canvas (today egui-panel-only).
- `bevy_enoki` adoption (windowed-gated) with a one-effect spike: dependency wired, one Agumon effect reauthored in `.particle.ron` at the existing spawn seam, K001 look check.
- Full migration of Agumon's Sharp Claws / Baby Flame / Baby Burner VFX to enoki (sketch — shape confirmed after the spike).
- Dep-gating regression test for the new enoki dependency.

### Out of Scope / Non-Goals

- Per-species enemy atlases and enemy-specific **attack** animations (enemy stays an Agumon dummy; it reacts via the shared sprite but has no own attack anim).
- Other Digimon's VFX (Renamon, Gabumon, etc.) — Agumon only.
- Roguelite run loop, node chain, progression, digivolution, HUD redesign, roster expansion (explicitly deferred by the user to after combat is visually complete).
- The GUI VFX editor.
- Retiring the M004 `vfx.ron` verb-math headless contract beyond what the enoki swap requires.

## Technical Constraints

- K001: the windowed binary must never be run from auto-mode; visual verification is a manual user step. The milestone cannot self-certify its visual half.
- R002/R004/R005/R010: headless-first, deterministic reaction mapping, windowed-gated deps, events-only UI path.
- The reaction logic that windowed code calls must live in the lib crate (integration tests link only against the lib, not the binary).
- The enoki spawn must hang off the existing FSM cue/barrier release seam (D031/D032) and must not move VFX particle lifetime into the kernel timeline.
- Death-fade must preserve the documented post-KO overshoot observability (M002) and the multi-hit loop semantics.

## Integration Points

- `EventReader<CombatEvent>` — source of `OnHitTaken` / `UnitDied` for reactions (read-only; no combat-state writes).
- `StanceGraphRegistry` + the bound stance snapshot — target of the reaction (drives hurt/death nodes).
- `bevy_enoki` (new, windowed-gated) — GPU 2D particle backend consuming `.particle.ron`; spawned at the existing `spawn_effect_by_id` seam.
- FSM cue/barrier release (D031/D032) — continues to trigger VFX spawns at the right animation frames; unchanged.
- Bevy asset loader — loads `.particle.ron` enoki effect assets.

## Testing Requirements

Headless (CI-provable, R004): the event→reaction mapping function round-trips all relevant `CombatEventKind` variants to the correct stance role with death-precedence ordering, snapshot-stable; a static dep-gating test asserts `bevy_enoki` symbols are absent from the headless build (R005); full `cargo test` + `cargo test --features windowed` + `cargo build --features windowed` stay green. Tests live under the appropriate `tests/<scope>/` harness (R003), likely `animation` or a reaction-specific scope. Visual (manual, K001): user sign-off in `cargo winx` for flinch-on-hit (both sides), death/fade at 0 HP, hit flash/shake + canvas damage numbers, and the enoki VFX look — iterated until satisfied.

## Acceptance Criteria

Per-slice criteria are (re)confirmed at planning. At the milestone level:

- The event→stance-reaction mapping is a pure lib function with deterministic headless tests covering hit, death, death-precedence, and no-op cases.
- In `cargo winx`, every hit produces a visible flinch on the struck unit (both combatants), and a unit at 0 HP plays the death node and leaves the field.
- Hit flash + shake are visible on the struck sprite; damage numbers render on the pixel canvas, not only in the egui panel.
- `bevy_enoki` is wired windowed-gated, at least one Agumon effect renders through it, and a static test proves no dep leak into the headless build.
- All three Agumon skills' VFX render through enoki and the user signs off that they look better than the current placeholder.
- Full `cargo test` (headless + windowed) and `cargo build --features windowed` are green.

## Open Questions

- Full enoki migration shape — how many effects, whether bitmap textures are needed vs procedural, and whether the old quad spawn path is deleted or kept behind the seam. Current thinking: resolve after the one-effect spike; the migration slice is a sketch until then.
- Death-fade treatment — instant remove vs fade-out duration, and whether the post-KO overshoot window needs the sprite to linger. Current thinking: short fade after the death node completes, preserving event-observable overshoot.
- Damage-number canvas rendering — reuse the egui number layout or author a sprite/text overlay on the pixel canvas. Current thinking: lightweight world-space text/sprite tied to `Damage` events, decided at slice planning.
