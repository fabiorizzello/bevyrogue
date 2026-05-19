# M002: First on-screen combat (Agumon-only)

**Gathered:** 2026-05-19
**Status:** Ready for planning

## Project Description

The first time bevyrogue's combat appears on screen. M002 assembles the AnimGraph runtime player, the wgpu sprite render, the §9 UI core, and the player-side two-clock impact-sync barrier into a single playable `cargo run --features windowed` session: Agumon vs an Agumon dummy at full kit, where the user clicks each skill in the egui action panel and watches it resolve on screen with damage landing on the visible impact frame. The animation/skill seam must be complete and extensible so M003–M007 and a future RON editor lean on it without rewrites. Closeout is gated by a repomix-grounded architectural review.

## Why This Milestone

Every architectural seam for the kernel↔anim-graph relationship is already decided (D001–D042) and partially wired, but nothing has been *seen* end-to-end. M002 is the first integration proof that the two-clock barrier (kernel stays frame-ignorant, player holds the barrier and calls `resume_cue()`), FrameCue/`ReleaseKernelCue` release, and per-skill 1:1 graphs actually produce coherent on-screen combat — not just passing headless tests. It must happen now because M003–M007 (more Digimon) and the future RON editor all build on this seam; locking it visually before adding content prevents a costly rewrite later.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run `cargo run --features windowed`, see Agumon idle-cycling driven by the stance graph, and click each kit action (Sharp Claws, Baby Flame, Baby Burner) in the egui action panel to watch the corresponding animation play against an Agumon dummy.
- See damage land on the visible impact frame, the multi-hit loop visibly equal the kernel hop count, the dummy's HP deplete in a minimal HUD (per-unit HP bar + on-hit damage numbers), and the dummy die when HP reaches zero.

### Entry point / environment

- Entry point: `cargo run --features windowed` (interactive); plus a measured soak harness (`BEVYROGUE_VALIDATION_WINDOWED`-style, `SOAK_SECS`-bounded) for closeout proof.
- Environment: local dev, native window (winit/wgpu), `windowed` feature only.
- Live dependencies involved: none (no network/db); hot-reload of RON assets from disk is in play during the soak.

## Completion Class

- **Contract complete means:** the executable anti-DRY test (`GameplayCommandForbidden` / zero gameplay numbers in `anim_graph.ron`) passes; clip↔atlas geometry parity (R003) stays green; invariant I3 extended to the cue handshake stays green (identical Intent stream headless vs windowed, only timing differs); all M001 headless Agumon tests stay green; R002/R004/R005/R006 hold.
- **Integration complete means:** the assembled windowed runtime drives Agumon's full kit vs an Agumon dummy through the real two-clock pipeline — animation playback, `ReleaseKernelCue`, per-hit cue handshake, the §9 phase strip updating from `EventReader<CombatEvent>`, target hurt/blink reactions, and HUD HP/damage all working across the real subsystems together.
- **Operational complete means:** a measured soak run with the full kit looped survives with no panic, no anim-graph-attributable frame-time regression vs a kernel-only baseline, and a mid-skill RON hot-reload without corrupting world state; an evidence artifact plus the repomix architectural review report are produced and findings triaged.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- A live `cargo run --features windowed` session where the user clicks each Agumon skill and the animation resolves with damage on the visible impact frame, the multi-hit loop visibly = kernel hop count (no N authored in the anim graph), and the dummy's HP bar depletes to death — observed on screen, not simulated.
- A data-driven soak run (captured console output, `SOAK_SECS`-bounded, full kit looped) showing zero panics, zero anim-graph-attributable FPS regression vs a kernel-only baseline, and a survived mid-skill hot-reload — produced as an evidence artifact alongside the repomix architectural review report.
- What cannot be simulated: the on-screen impact-frame coherence (animation clock and kernel resolution clock staying in sync through the player-side barrier under real wgpu rendering) and the absence of anim-graph-induced frame-time lag under real window/render load.

## Architectural Decisions

> The kernel↔anim-graph seam is fully decided in `.gsd/DECISIONS.md` (D001–D042). M002 inherits those without re-litigation. The decisions below are the M002-specific behavioral resolutions reached during this discussion; the inherited seam decisions are summarized after them for execution context.

### Interaction model is interactive, not a scripted demo

**Decision:** The windowed run is driven by the user clicking skills in the existing egui action panel and watching them resolve on screen (option a). It is not a no-input scripted/auto demo loop.

**Rationale:** Validates the real player→intent→pipeline path a human will use, and exercises the two-clock barrier under genuine on-demand skill invocation rather than a canned sequence.

**Alternatives Considered:**
- Scripted/auto demo loop (fire full kit in sequence, no input) — rejected; doesn't prove the interactive intent path and is closer to a soak than a playable build.
- Both interactive + auto loop as the proof — partially adopted: interaction is interactive; the *closeout proof* uses the soak harness separately (see Error Handling / Final Integrated Acceptance).

### Dummy has real depleting HP and can die; no end screen

**Decision:** The Agumon dummy has real HP that depletes from kernel damage and dies at zero HP. No victory/defeat screen is required for M002.

**Rationale:** Real HP/death proves damage actually flows from kernel to the visible target; an end-state screen is presentation polish out of scope for a first integration proof.

**Alternatives Considered:**
- Immortal punching bag (kit loops forever) — rejected; wouldn't prove damage causes a real, terminal state change.
- HP + death + victory/defeat screen — deferred; end-state UI is not needed to prove the seam.

### Damage must be visible as numbers and HP bars in a minimal HUD

**Decision:** On-screen damage proof is numeric: a minimal HUD with per-unit HP bars and on-hit damage numbers. Hurt-blink / §9 phase strip alone is insufficient.

**Rationale:** The user explicitly wants damage legible as numbers + bars so impact-frame timing and multi-hit hop count are visually verifiable, not inferred from a blink.

**Alternatives Considered:**
- Damage shown only via target blink/hurt reaction + phase strip (no numerics) — rejected by the user; not legible enough to verify impact-frame and hop-count behavior.
- Full HUD (turn-order tray, cooldown UI) — out of scope; "minimal" = HP bars + damage numbers + the existing egui action panel only.

### Closeout proof is data-driven analysis, not human eyeballing

**Decision:** The FPS/stability acceptance bar is met by a measured soak run analyzed from captured data, producing an evidence artifact alongside the repomix architectural review report. The concrete bar is: the anim graph must not introduce FPS lag (no anim-graph-attributable frame-time regression vs a kernel-only baseline), zero panics, and a survived mid-skill hot-reload.

**Rationale:** "Stable (qualitative) FPS" in the roadmap is too soft to gate a milestone; the user wants the closeout decided by data, with a durable artifact next to the repomix report.

**Alternatives Considered:**
- Human eyeballs the window and judges FPS — rejected by the user; not reproducible or auditable.
- Qualitative success criterion as written in the roadmap — superseded by the measured-soak bar above.

### Inherited seam decisions (execution context, not re-decided here)

- **Gameplay numbers live in the kernel only** (D001-class): `skills.ron` beats (`DealDamage`, `BeatKind::Loop{exit_when}`, `DamageCurve::PerHop`) are the single source of truth; `anim_graph.ron` never authors `EmitDamage/EmitStatus/EmitHeal`; enforced by an executable anti-DRY test. The M001 `mul:18` duplicate is remediated behind it.
- **Player side owns the two-clock impact-sync barrier** (D002-class): the AnimGraph player holds the barrier and calls `resume_cue()` at the authored frame; the kernel stays frame-ignorant, its resolution clock gated on the released Intent.
- **One FrameCue mechanism** (D003-class): `AnimNode.cues: Vec<FrameCue>`, `FrameCue{at,command}` carrying either a presentation `Command` or `ReleaseKernelCue` (no id, no number) — covers impact sync, telegraph chip, VFX, footstep, sprite-move.
- **Per-skill graph granularity** (D004-class): one `AnimGraph` = one skill, mapped 1:1 to the kernel `CompiledTimeline` by shared skill-id.
- VFX-as-opaque-id, closed serde enums, repomix closeout gate, and the failure defaults remain as decided in D005–D042.

## Error Handling Strategy

- **Anti-DRY enforcement:** authored gameplay numbers in `anim_graph.ron` are a hard, test-failing error (`GameplayCommandForbidden`), not a warning — the seam is an executable invariant.
- **Two-clock desync:** if the kernel resolution clock is not gated waiting for the released Intent (the known `timeline_exec.rs` auto-resume gap), that is a milestone-invalidating bug surfaced through the I3-extended cue-handshake test, not silently tolerated.
- **Hot-reload mid-skill:** a RON hot-reload while a skill is mid-flight must not corrupt world state; the soak explicitly exercises this and a corrupted/inconsistent post-reload state fails the soak.
- **Soak/closeout:** any panic, or an anim-graph-attributable frame-time regression vs the kernel-only baseline, fails the milestone; the captured console output and frame-time data are the evidence, retained as an artifact next to the repomix report.
- **Determinism:** no wall-clock, no unseeded RNG (R004); non-determinism introduced by VFX/animation is a failure.
- General posture: explicit failure modes and observable state transitions; never suppress an undiagnosed desync with a guard.

## Risks and Unknowns

- Two-clock barrier not yet wired (`timeline_exec.rs:118` auto-resumes) — S02 must make the kernel actually suspend on `AwaitingCue`; if it doesn't, impact-frame sync is unprovable and the milestone is blocked.
- Anim-graph frame-time cost under real wgpu render load — the core acceptance bar is "no anim-graph-attributable FPS lag"; needs a credible kernel-only baseline to attribute regression cleanly.
- `agumon/clip.ron` geometry was historically off-by-one vs `agumon_atlas.json`; R003 remediation must stay green or sprite frames render wrong on screen.
- "Minimal HUD" scope creep — must stay HP bars + damage numbers + existing egui panel; richer UI would expand S05/S06 unboundedly.
- Mid-skill hot-reload corruption is a plausible real failure mode that headless tests don't exercise; only the soak catches it.

## Existing Codebase / Prior Art

- `src/windowed.rs` — current egui combat panel; the interactive entry point the user clicks skills in; extended (not replaced) for the M002 HUD.
- `src/animation/plugin.rs` — pre-split animation plugin; S01 splits it into `RenderPlugin`/`UiPlugin` gated `#[cfg(feature="windowed")]` (R002/R005).
- `src/animation/anim_graph.rs`, `src/animation/clip.rs`, `src/animation/validation/` — AnimGraph schema + the `GameplayCommandForbidden` validation check the anti-DRY test enforces.
- `src/combat/` (turn pipeline, `preview.rs`, timeline exec) — kernel side of the two-clock barrier; stays frame-ignorant.
- `tests/clip_geometry_parity.rs` — R003 clip↔atlas parity; must stay green (MEM008).
- `assets/digimon/agumon/anim_graph.ron`, `assets/digimon/agumon/clip.ron`, `assets/data/digimon/agumon/skills.ron` — Agumon's authored graph/clip/kernel data; `mul:18` duplicate remediated behind the anti-DRY test.
- `.gsd/milestones/M002/M002-ROADMAP.md` — S01–S06 slice plan and boundary map this context backs.

## Relevant Requirements

- R003 (Validated, M001) — clip↔atlas geometry parity; M002 must keep it green so on-screen Agumon sprites render correctly; it is prior-art baseline, not advanced by M002.
- No M002-specific requirements are registered yet in `.gsd/REQUIREMENTS.md`; the roadmap Success Criteria plus this context's Acceptance Criteria are the working contract. Formal requirements should be captured during planning if needed.
- Standing rules that must hold: R002 (headless-first; gate egui/winit only via `#[cfg(feature="windowed")]`), R004 (determinism), R005 (no winit/wgpu/egui deps outside `windowed`), R006 (repo hygiene).

## Scope

### In Scope

- AnimGraph runtime player driving a wgpu sprite from the stance graph (Agumon idle cycle not hardcoded).
- `RenderPlugin`/`UiPlugin` split gated `#[cfg(feature="windowed")]`.
- Two-clock impact barrier wired into the turn pipeline (kernel suspends on `AwaitingCue`; player `resume_cue()` at authored frame); per-hit cue handshake.
- Full Agumon kit on screen: Sharp Claws (basic + telegraph chip), Baby Flame, Baby Burner reactive detonate + flash VFX, Twin Core fire side via placeholder ally; multi-hit loop visibly = kernel hop count.
- §9 phase strip live, event-driven from `EventReader<CombatEvent>`; structural test that the UI path never mutates combat state.
- Minimal HUD: per-unit HP bars + on-hit damage numbers; Agumon dummy with real depleting HP and death.
- Executable anti-DRY test (zero gameplay numbers in `anim_graph.ron`); `mul:18` remediated behind it.
- Measured soak harness + evidence artifact; repomix architectural review report at closeout with findings triaged.

### Out of Scope / Non-Goals

- Victory/defeat screen or any end-state UI.
- Turn-order tray, cooldown UI, or any HUD beyond HP bars + damage numbers + the existing egui action panel.
- Any Digimon other than Agumon (and the Agumon dummy); M003–M007 content.
- A RON editor (the seam must support a future one, but none is built here).
- Network, persistence, audio, or production packaging.
- Auto/scripted demo as the interaction model (used only for the closeout soak, not as the playable build).

## Technical Constraints

- Headless-first (R002): every system must run without `windowed`; egui/winit/wgpu gated only via `#[cfg(feature="windowed")]`.
- No winit/wgpu/egui deps outside the `windowed` feature (R005).
- Determinism (R004): no wall-clock, no unseeded RNG; seeded `bevy_rand` + insta.
- Zero gameplay numbers in `anim_graph.ron` — enforced as an executable test, not convention.
- Closed serde enums, `#[serde(default)]`, no untagged, for all schema extensions (FrameCue/ReleaseKernelCue/Predicate::KernelCue).
- All M001 headless Agumon tests stay green; I3 extended to the cue handshake stays green.
- No `.md` in repo root (R006).
- Cranelift dev profile / `rust-toolchain.toml` toolchain unchanged.

## Integration Points

- wgpu / winit (via `windowed` feature) — native window + sprite render the AnimGraph player drives.
- egui — existing action panel (skill input) extended with the minimal HUD.
- Bevy ECS combat kernel — kernel side of the two-clock barrier; receives released Intents, owns gameplay numbers.
- RON asset hot-reload (Bevy asset server) — exercised by the soak for mid-skill reload resilience.
- repomix — closeout architectural review report generator; findings triaged before milestone completion.

## Testing Requirements

- **Unit/integration (headless):** all M001 Agumon headless tests stay green; `GameplayCommandForbidden` anti-DRY test passes; clip↔atlas parity (R003) passes; structural test that the §9 UI path never mutates combat state; I3 extended so the windowed Intent stream is identical to headless (only timing differs).
- **Integration (windowed-assembled):** full kit vs Agumon dummy resolves through the real two-clock pipeline with per-hit cue handshake; multi-hit loop count == kernel hop count with no N authored in the anim graph.
- **Soak (data-driven closeout):** `SOAK_SECS`-bounded run, full kit looped, captured console output; assert zero panics, zero anim-graph-attributable frame-time regression vs a kernel-only baseline, and a survived mid-skill RON hot-reload. Soak data retained as an evidence artifact.
- **Closeout gate:** repomix-grounded architectural review report produced; findings triaged.
- Determinism (R004) and headless-first (R002) hold across the suite.

## Acceptance Criteria

Per-slice (from the roadmap "After this:" lines, refined by this discussion):

- **S01:** `cargo run --features windowed` shows Agumon idle-cycling via the stance graph (not hardcoded); M001 headless tests green; clip↔atlas geometry parity test present and passing.
- **S02:** Sharp Claws windup→strike→recovery on screen; damage falls on the impact frame via `ReleaseKernelCue`; telegraph chip visible; I3 extended green (identical Intent stream headless vs windowed, only timing differs).
- **S03:** §9 phase strip updates from `EventReader<CombatEvent>`; structural test asserts the UI path never mutates combat state.
- **S04:** Baby Burner reactive detonate with a flash VFX (Rust code, no RON/editor); zero non-determinism; R004 intact; headless tests unchanged.
- **S05:** Agumon vs Agumon dummy at full kit (Sharp Claws + Baby Flame + Baby Burner + Twin Core fire side via placeholder ally); multi-hit loop visibly == kernel hop count; target blink/hurt driven by `CombatEvent`; dummy HP depletes via minimal HUD (HP bars + damage numbers) and the dummy dies at zero HP.
- **S06:** windowed soak with no panic, no anim-graph-attributable FPS regression vs kernel-only baseline, mid-skill hot-reload not corrupting world state, captured console output; evidence artifact produced alongside a repomix-grounded architectural review report with findings triaged.

Milestone-level: all roadmap Success Criteria met, with the soft "stable (qualitative) FPS" criterion replaced by the measured "no anim-graph-attributable FPS lag" bar.

## Open Questions

- Exact `SOAK_SECS` value and the precise frame-time regression threshold (absolute ms vs %) — current thinking: pick during S06 planning from a measured kernel-only baseline; the bar is "no anim-graph-attributable regression," tolerance set from observed baseline variance.
- Whether M002-specific requirements should be formally registered in `.gsd/REQUIREMENTS.md` — current thinking: capture during planning if the roadmap Success Criteria need traceable IDs; not blocking context.
- Placeholder-ally representation for the Twin Core fire side in S05 (off-screen actor vs minimal on-screen stand-in) — current thinking: minimal/non-visual placeholder sufficient since Twin Core's fire side is the proof, not a second rendered Digimon; confirm at S05 planning.
