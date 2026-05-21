---
id: S05
parent: M002
artifact: RESEARCH
generated: 2026-05-21
slice_title: "Full kit: Agumon vs Agumon dummy"
depth: targeted
---

# S05 RESEARCH — Full kit: Agumon vs Agumon dummy

## Summary

S05 assembles M002's first user-facing proof: an interactive `cargo run --features windowed --bin bevyrogue` session where two Agumon sprites face off, the player clicks each kit action in the egui action panel, damage lands on the visible impact frame, the dummy's HP visibly depletes via a minimal HUD (per-unit HP bars + on-hit damage numbers), targets visually react to hits (blink/hurt), the multi-hit loop iterates exactly the kernel's `hop_index` count, and the dummy dies at 0 HP.

S02 already shipped the deterministic two-clock cue barrier and Sharp Claws. S04 shipped Baby Burner *reactive* detonate (post-KO Rust seam) and the windowed flash projection. What's still **missing** to deliver the demo:

1. A windowed-mode encounter bootstrap (currently only `combat_cli` and headless tests bootstrap rosters; the windowed app starts with no units on screen).
2. A second on-screen sprite for the dummy (only one `AgumonSprite` is spawned today).
3. Per-unit HP bars + sprite-anchored damage-number positioning (the `FloatingDamage` primitive + `FdDisplay` projection already exist but render only in egui overlays).
4. Target blink/hurt driven by `CombatEventKind::OnHitTaken` (no consumer wired today).
5. The **Baby Burner primary timeline** in `skills.ron` (S04 only proved the reactive path; the Ultimate has `implementation: Implemented`, sp_cost=0, but no timeline beats — UI launch would no-op).
6. A windowed-side handshake that makes the **multi-hit loop visibly iterate per kernel hop** (Baby Flame's `BeatKind::Loop` fires N hops with `hop_index` on each `BeatEvent`, but the anim player today only knows `PlaybackModifier::Loop { count }`, which would hardcode N and break the anti-DRY rule).
7. The Baby Flame talent `agumon/has_bouncing_fire` is currently false by default; the kit demo needs the bouncing loop active so the multi-hit visualization has something to show.

Twin Core ships as a placeholder (per CONTEXT: "Twin Core fire side via placeholder ally") — its kernel state machine is registered in the blueprint but there is no second-actor unit, no kit slot binding, and no animation graph; the slice should keep Twin Core minimal (signal-fires-only) and not bloat into a full ally system.

## Recommendation

Build the slice as five independent, mostly-vertical tasks ordered by unblock value. **First proof (T01)** is the windowed encounter bootstrap + two sprites on screen + per-unit HP bars: the smallest end-to-end pass that lights up the visible scene, validates that S02/S04 work survives a real encounter, and gives every later task something to verify against. After T01 the remaining tasks (target blink, multi-hit hop binding, Baby Burner primary timeline, kit-slot wiring) are independently testable.

Reuse the established **windowed-only projection pattern** (MEM052, MEM046, MEM056): `CombatEvent` → UI-owned resource with a deterministic frame budget → render-only system. Do **not** mutate `CombatState` from any windowed/UI code. Do **not** hardcode hop counts in `anim_graph.ron` (anti-DRY invariant `GameplayCommandForbidden` — extend the test if a new edge appears).

For the multi-hit loop visibility, prefer **kernel-cue-driven anim restarts per hop** (extend the two-clock barrier so each loop iteration is a presentation beat that the AnimGraphPlayer consumes) over teaching the player a "loop N times based on a number". The first preserves the seam; the second leaks gameplay numbers into presentation.

## Implementation Landscape

### A. Windowed encounter bootstrap (NEW)
- `src/windowed/mod.rs:49-80` — `UiPlugin::register()` mounts egui panels but has no `Startup` system that builds the roster, calls `apply_composition`, or emits `PartySelected`/`TurnOrderSeeded`.
- `src/combat/encounter/bootstrap.rs` — headless reference; mirrors what windowed needs but for `EncounterPreset::BossEncounter`. Slice should add a windowed-friendly variant or parameterize a same-team-different-side Agumon-vs-Agumon preset.
- `src/headless.rs:245` — sample call site for `bootstrap_encounter(roster, request, preset)`.
- `assets/data/digimon/agumon/unit.ron` — both player and dummy reuse this; differ only in `Team` / `Commander` flags.

### B. Per-unit sprite scene (NEW)
- `src/windowed/render.rs:17-22` — `AgumonSprite` component drives the FSM.
- `src/windowed/render.rs:87-206` — `advance_agumon_presentation()` spawns/updates a **single** sprite; no Transform/Sprite2d layout for a second actor.
- Slice needs: a per-`Unit` query that spawns one sprite-bundle per combatant, places them at fixed left/right Transforms, and ties anim state to that unit's current skill cast.

### C. HUD: per-unit HP bars + damage numbers (PARTIAL)
- `src/ui/combat_panel/display.rs:16-44` — `UnitDisplay { hp_cur, hp_max, ... }` and `FdDisplay` already exist and are populated from `Unit`/`FloatingDamage`.
- `src/ui/combat_panel/render.rs:60` — combat panel already reads `FloatingDamage` and renders FdDisplay in egui overlays.
- `src/combat/observability/floating.rs:6-28` — `FloatingDamage` component spawned in `src/combat/turn_system/pipeline/paths/single_target.rs:236-259` on `OnDamageDealt`.
- `src/combat/turn_system/pipeline/paths/single_target.rs:254` — `spawn_time: time.elapsed_secs()` — note this couples lifetime to wall clock; verify R004 stays clean (S05 should switch to frame-counter if any new code is added).
- **Gap:** No per-sprite-anchored HP bar widget. The roster panel shows HP textually (`src/windowed/mod.rs:220-250`), but the milestone CONTEXT explicitly requires HP **bars** + damage **numbers** as a HUD, anchored near each unit.

### D. Target blink/hurt (MISSING)
- `src/combat/observability/events.rs:92-95` — `CombatEventKind::OnHitTaken { amount }` already fires per hit with `source/target/cast_id` in the envelope.
- No `MessageReader<CombatEvent>` in animation/render code mutates sprite color/alpha. Plan: add a windowed-only resource `TargetHurtState` keyed by `UnitId` with a deterministic N-frame countdown (mirroring `BabyBurnerFlashState` at `src/ui/combat_panel/mod.rs:103-158`), and a render-side system that tints the matching sprite for that window.
- Determinism: drive by frame counter, not `Instant::now` (the existing `floating.rs:17-28` uses 1.2s decay — acceptable for popups but new blink code should be frame-counted).

### E. Multi-hit loop visibility = kernel hop count (HARDEST)
- `src/combat/runtime/timeline.rs:151-157` — `BeatEvent { hop_index: u32, ... }` natively threads hop count.
- `src/combat/runtime/runner.rs:33-273` — `LoopFrame { hop_index }`; each loop body iteration fires `fire_beat(cur_beat, hop_index, params)` and increments at line 273.
- `src/animation/player.rs:123-141` — `PlaybackModifier::Loop { count }` is a presentation-only intrinsic loop; **must not** be set from any gameplay number.
- `src/combat/runtime/cue_barrier.rs:32-69` — the suspension barrier already supports per-beat presentation gating; S02 proved this for the single-impact case.
- **Approach:** Treat each Baby Flame loop hop as a presentation barrier. Emit a `ReleaseKernelCue`-equivalent per hop so the AnimGraphPlayer consumes one `KernelCue` per hop and visually restarts/loops the strike node. The player intrinsically loops the strike clip frames; the *number of visible iterations* is then governed by how many kernel hops fired.
- `assets/digimon/agumon/anim_graph.ron` — Baby Flame nodes (cast → impact → recover) currently exist; the impact node will need either a self-transition gated on `Predicate::KernelCue` (preferred) or a new "hop" node that loops back via consumable cue.
- `assets/data/digimon/agumon/skills.ron:25-36` — `BeatKind::Loop` body `bounce_hop` with selector `agumon/bounce_pick_next` and gate `agumon/bounce_exit`. Loop body damage is 9 per hop. **Talent `agumon/has_bouncing_fire` must be enabled** for the loop to iterate (otherwise the gate exits immediately) — slice should either enable the talent on the demo Agumon or have S05 set up a default-on configuration for the dummy match.
- `tests/timeline_chain_bolt_port.rs:74-76` — reference for hooks reading `ev.hop_index`.

### F. Baby Burner primary timeline (MISSING)
- `assets/data/digimon/agumon/skills.ron:86-102` — `agumon_ult` defined but **no `timeline` field**; only legacy_ops + `apply_thermal_spark(3)` signal. Clicking Ultimate in the UI will not produce a beat sequence.
- `src/combat/blueprints/agumon/baby_burner.rs:16-26` — reactive detonate already wired; only fires on a heated-target KO.
- `assets/digimon/agumon/anim_graph.ron` — no `baby_burner_charge` / `baby_burner_launch` graph nodes; only Sharp Claws + Baby Flame are authored.
- Slice must add a minimal timeline (windup → impact + Heated/Thermal stack → recovery) and matching anim nodes so the Ultimate button does something visible and lethal-on-Heated targets, which then chains into the **S04** reactive flash (closing the M002 narrative).

### G. Twin Core placeholder
- `src/combat/blueprints/twin_core/mod.rs` + `src/combat/blueprints/agumon/mod.rs:22,54` — passive timeline dormant→proc→resolve on `ult_used` event already wired. No ally unit, no anim graph.
- `assets/data/digimon/agumon/unit.ron:11-13` — kit slots are basic/skill/ultimate only; no Twin Core slot.
- Slice should keep Twin Core as a **signal-fires-only** placeholder: when the Ultimate resolves, the existing passive emits its blueprint signal; windowed projects that signal to a small UI badge ("Twin Core synergy primed") and stops there. **Do not** spawn a second Agumon entity for Twin Core; that's M003+ territory.

### H. Egui action panel dispatch (READY)
- `src/ui/combat_panel/render.rs:277-306` — `PendingKind::{Basic,Skill(SkillId),Ultimate}` → `ActionIntent::Skill` already wired.
- `src/ui/combat_panel/mod.rs:44-56` — `PendingAction` resource carries the player's pending move.
- No change needed in the dispatch layer beyond making sure Baby Flame and the new Ultimate timeline resolve cleanly.

### I. Determinism and R004
- Scout confirmed no `rand`/`thread_rng`/`Instant::now`/`SystemTime` under `src/animation/`, `src/windowed/`, `src/ui/`. Keep it that way: any new blink/HP-bar state must use frame counters, not wall clock.

### J. Hot-reload posture (READY)
- `src/windowed/mod.rs:130-132` — `watch_for_changes_override: Some(true)`.
- `src/animation/registry.rs:59-68` + `src/animation/plugin.rs:176-335` — anim graph + clip hot reload listeners already in place. New `skills.ron` edits for the Baby Burner timeline must respect this; S06 will exercise mid-skill hot reload as its own demo, but S05 should not regress it.

## Natural Seams (proposed task decomposition)

These are independent enough to land in parallel after T01.

- **T01 — Windowed scenario bootstrap + two-sprite scene + per-unit HP-bar widget.** Highest unblock value: lights up the demo skeleton.
- **T02 — `OnHitTaken` → target blink/hurt projection.** Pure UI projection, mirrors S04's flash pattern.
- **T03 — Baby Flame multi-hit loop visibility via per-hop KernelCue.** Touches cue barrier + anim player + skills.ron loop body + anim graph; the highest-risk seam.
- **T04 — Baby Burner primary timeline + anim nodes.** Adds windup/strike/recovery beats and matching `agumon/anim_graph.ron` nodes; reactive detonate (S04) automatically chains on lethal heated hits.
- **T05 — Twin Core placeholder UI signal (no ally spawn).** Project the existing blueprint signal to a small badge; smallest task, lowest risk, easy closer.

The slice's "demo" success — a playable run that exercises all four kit actions on the dummy with HP visibly depleting to death — is the final integration verification.

## First Proof (recommended T01)

The minimum viable beachhead is:

1. Windowed app starts → encounter bootstrap runs at `Startup` → two `Unit` entities are present (player Agumon, dummy Agumon).
2. Both sprites render in the windowed scene at fixed Transforms (left/right).
3. Per-unit HP bar widget appears anchored near each sprite (egui overlay positioned by sprite Transform, or a Bevy-UI Node bound to UnitId).
4. Clicking **Sharp Claws** (already shipped in S02) on the dummy lands damage, the existing `FloatingDamage` number renders, and the dummy's HP bar visibly decreases.
5. Existing tests stay green; one new test covers the bootstrap producing exactly two `Unit` entities with the right teams.

If T01 passes, every subsequent task (blink, multi-hit, Baby Burner primary, Twin Core badge) has a stable demo surface to plug into and verify visually.

## Verification

Each task must keep the slice/milestone invariants green:

- `cargo test --test anim_gameplay_command_forbidden` — anti-DRY guard (no gameplay numbers in anim_graph.ron); extend with a hop-count assertion if T03 introduces a new cue.
- `cargo test --test clip_atlas_parity` — R003 geometry parity.
- `cargo test --test anim_player_fsm --test anim_graph_asset` — animation FSM + asset shape.
- `cargo test --test timeline_two_clock_parity` — R002 headless/windowed intent-stream parity (must remain identical after T03's per-hop cue work; this is the test that catches a desync).
- `cargo test --test timeline_cue_barrier_pipeline` — R004 deterministic suspension/resume (must extend if T03 changes barrier semantics).
- `cargo test --test agumon_baby_burner_reactive` — S04 invariants (must stay green after T04 adds the primary timeline; lethal heated hit must still chain into the reactive detonate exactly once).
- `cargo test --test agumon_sharp_claws_asset` — S02 invariants.
- `cargo test --features windowed --test windowed_preview_cache` — feature-gated windowed surface (extend with HP-bar + blink projection tests).
- `cargo test --lib` — workspace tests.
- `cargo build --no-default-features` and `cargo build --features windowed` — R005 dependency gating.
- **New tests S05 should add:**
  - Windowed bootstrap produces two Agumon units on opposing teams (T01).
  - `TargetHurtState` countdown reflects exact `OnHitTaken` event count and clears after N frames (T02).
  - Baby Flame loop body fires exactly `hop_index` presentation barriers, each consumed by the anim player, with identical final HP between headless and windowed (T03, extends `timeline_two_clock_parity`).
  - `agumon_ult` timeline resolves windup → impact damage → Heated → recovery; lethal heated hit emits exactly one `baby_burner_detonate` chain (T04).
  - Twin Core blueprint signal projects to UI resource without mutating combat state (T05).
- **Closeout (matches S04 pattern):** the test/build matrix above is the canonical evidence. Live `cargo run --features windowed --bin bevyrogue` (MEM042/MEM043) is the human-eyeball demo when a display/GPU is available; otherwise the windowed test coverage substitutes.

## Constraints to Honor

- **Anti-DRY (D001):** `anim_graph.ron` must not author gameplay numbers. The Baby Flame hop count comes from kernel `BeatKind::Loop`; the anim graph stays presentation-only.
- **Two-clock barrier (D002):** kernel stays frame-ignorant; the player holds the barrier and calls `resume_cue()` at authored frames. Extending the barrier per loop hop must not introduce a kernel-side frame dependency.
- **R002 / R005:** all egui/winit/wgpu/Bevy-UI dependencies stay behind `#[cfg(feature = "windowed")]`.
- **R004 determinism:** no wall-clock, no unseeded RNG in new windowed/UI code; frame counters only.
- **R003 clip↔atlas parity:** any new Baby Burner anim nodes must use existing tracked atlas frames (or extend `agumon_atlas.json` and regenerate `clip.ron`).
- **MEM037:** `assets/digimon/agumon/anim_graph.ron` must stay on `clip: "all"` (Baby Flame uses `skill` atlas range, Sharp Claws uses `attack` atlas range; new Baby Burner nodes will need to fit within whatever atlas range the new frames occupy or also live under "all").
- **MEM052/MEM046/MEM056:** windowed presentation = projection of generic `CombatEvent`/`OnKernelTransition` into UI-owned resources with deterministic frame budgets. Never mutate `CombatState` from windowed code.
- **MEM033:** the kernel skill timeline uses `BeatPayload::DealDamage` with concrete `i32` — do not introduce `ParamRef::Static` to skill RON; the existing 9/hop in skills.ron is fine.

## Skills Discovered

- `bevy-ecs-expert` (already installed) — directly relevant for the systems/resources/observers added in T01–T05.
- No new skills required; `make-interfaces-feel-better`, `observability`, and `verify-before-complete` (already-available) are useful at implementation time but no novel installs were needed for the research phase.

## Risks and Watch-outs

- **R1 (high):** The per-hop cue plumbing (T03) is the only seam that can desync the headless/windowed Intent streams (R002). The mitigation is the existing `timeline_two_clock_parity` test — extend it to cover Baby Flame's loop before changing barrier semantics.
- **R2 (medium):** Baby Burner's *primary* timeline interacts with the S04 reactive seam. Adding a primary lethal-on-Heated hit must still trigger the reactive detonate exactly once (existing `tests/agumon_baby_burner_reactive.rs` should catch a regression — verify it covers the primary-launch flow, not just synthetic KOs).
- **R3 (medium):** Windowed bootstrap needs an Agumon-vs-Agumon preset that doesn't exist (`EncounterPreset::BossEncounter` targets Devimon). Either parameterize the preset or add a small new variant. Either way, headless tests should bootstrap the same fixture so the demo is reproducible.
- **R4 (medium):** Talent `agumon/has_bouncing_fire` defaults to false; the multi-hit Baby Flame loop won't iterate without it. Decide between (a) enabling it on the demo Agumon or (b) hardcoding a temporary "demo unit" override. Don't quietly invert the default for all unit data.
- **R5 (low):** The existing `FloatingDamage::spawn_time` uses wall clock — touching it for sprite-anchored placement is fine; introducing new wall-clock dependencies in T02's blink system is not. Frame counters only.
- **R6 (low):** Twin Core scope creep — milestone explicitly defers ally mechanics. Keep T05 to a UI badge.

## Sources

- Inlined milestone CONTEXT and ROADMAP (M002).
- S02 SUMMARY and S04 SUMMARY (dependency slices).
- Memory: MEM019, MEM030, MEM033, MEM037, MEM042, MEM043, MEM045, MEM046, MEM049, MEM050, MEM052, MEM054, MEM056.
- Direct code reads / parallel scout traversal: `src/windowed/`, `src/ui/combat_panel/`, `src/animation/player.rs`, `src/combat/runtime/{timeline,runner,cue_barrier}.rs`, `src/combat/observability/events.rs`, `src/combat/turn_system/pipeline/paths/single_target.rs`, `src/combat/blueprints/agumon/{mod,baby_burner}.rs`, `src/combat/blueprints/twin_core/mod.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/data/digimon/agumon/{skills,unit}.ron`, `tests/timeline_chain_bolt_port.rs`.
