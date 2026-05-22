# S02 Research: Skill + Ultimate windowed cue bridges (Baby Flame + Baby Burner)

**Lane:** research · **Depth:** targeted→deep (known codebase, but a real data-authoring gap + multi-barrier orchestration push this toward deep)

## Summary

S01 generalized atlas binding and proved the impact-frame-on-rendered-frame invariant for **Sharp Claws only**, riding the single windowed cue bridge that already existed. S02 must extend that bridge to **Baby Flame** (`skill_id "baby_flame"`, skill range) and **Baby Burner** (`skill_id "agumon_ult"`, `heavy_attack` range) so their kernel barriers release on the **rendered impact frame** instead of being auto-released.

The central finding: the three skills are **not symmetric**, and Baby Flame as currently authored **cannot land damage on its rendered impact frame without new data authoring**. This is the biggest unblocker and should be proven first.

## Key Findings (read these before planning)

### 1. The cue barrier only suspends on beats carrying `presentation: Some(...)`
`SuspendedTimeline::new` (src/combat/runtime/cue_barrier.rs:114) reads `runner.awaiting_cue_info()`, which is populated only for presentation-bearing beats (src/combat/runtime/runner.rs:150-159, `animation_node: presentation.anim`). In `Clock::Windowed`, the runner returns `AwaitingCue` only on presentation beats. **No presentation → no barrier → damage resolves immediately, no rendered-frame gating.**

### 2. Baby Flame's impact beats have `presentation: None` (CRITICAL GAP)
In `assets/data/digimon/agumon/skills.ron` (`SkillId("baby_flame")`, lines 17-44):
- `impact_damage`, `impact_break`, `impact_signal` beats all have `presentation: None`.
- The **only** presentation cue in Baby Flame is inside the `bounce_loop` body: `bounce_hop` → `(cue_id: "agumon/baby_flame/bounce_hop", anim: Some("baby_flame_impact"))`, and that loop is gated behind `agumon/has_bouncing_fire`.

Consequence: for base Baby Flame (no Bouncing Fire passive) there is **no impact barrier at all** — damage lands the moment the kernel steps the impact beat, not on the rendered frame. Also, `anim_graph.ron`'s `baby_flame_impact` node (frames 69-72, `Hold(extra_frames:2)`) carries **no `ReleaseKernel` cue** — unlike `sharp_claws_strike` and `baby_burner_launch`, which both have `(at: 1, command: ReleaseKernel(()))`.

**To satisfy the S02 outcome ("Baby Flame release fires on the cue frame, not auto-released"), the planner must author two things:**
- (a) Add `presentation: Some((cue_id: "agumon/baby_flame/impact", anim: Some("baby_flame_impact"), ...))` to Baby Flame's `impact_damage` beat in `skills.ron`.
- (b) Add a `ReleaseKernel` cue to the `baby_flame_impact` node in `anim_graph.ron` at the impact local frame (mirror Sharp Claws' `at: 1`).

Verify there is no compiled-timeline layer that injects presentation onto baby_flame impacts (grep `src/combat/runtime/` + `src/data/skill_timeline.rs`); current evidence (sharp_claws explicitly sets presentation, baby_flame explicitly `None`) indicates this is authored intent, not a defaulting bug.

### 3. Baby Burner is a THREE-barrier sequence, not one
`SkillId("agumon_ult")` (skills.ron:86-127) has presentation on **three** beats:
- `windup` → cue `agumon/baby_burner/windup`, anim `baby_burner_charge` (frames 23-30, **no ReleaseKernel cue**).
- `impact_damage` → cue `agumon/baby_burner/impact`, anim `baby_burner_launch` (frames 31-37, **has `ReleaseKernel` at:1** → impact clip frame 32).
- `recovery` → cue `agumon/baby_burner/recovery`, anim `baby_burner_recovery` (frames 38-45, no cue).

The anim graph (`anim_graph.ron`) gates `baby_burner_charge → baby_burner_launch` on `KernelCue` and `baby_burner_launch → baby_burner_recovery` on `KernelCue` (lines 86-95). So advancing the FSM through Baby Burner requires firing a kernel cue at the charge and launch boundaries — but only `launch` has an authored `ReleaseKernel` frame marker. The windup barrier (charge node) has no cue frame, so the bridge needs an explicit release rule for it (e.g. release at end-of-node / `TimeInNode`-equivalent, or author a cue). The current Sharp Claws bridge handles exactly **one** barrier; Baby Burner needs sequential barrier handling.

### 4. The current bridge hardcodes Sharp Claws and auto-releases everything else
`src/windowed/render.rs`:
- `AgumonPlaybackMode` enum (lines 32-39) is `Idle | SharpClaws { cue_id, presentation_node }` — must generalize to carry skill identity + start node.
- Lines 264-277: any barrier whose `skill_id.0 != SHARP_CLAWS_SKILL_ID` is **auto-released immediately** (`request_release`) to avoid a stall. This is the code S02 replaces with real Baby Flame / Baby Burner bridges.
- `sync_agumon_mode` (394-465), `sharp_claws_barrier` (467-469), `sharp_claws_start_node` (497-503), and the `SHARP_CLAWS_*` constants in `src/windowed/mod.rs:39-43` are all Sharp-Claws-specific seams needing per-skill generalization.
- Release detection `should_release_kernel` (480-484) scans the current node's cues for `ReleaseKernel` at the current local frame — works for any node that **has** such a cue (so it works for `baby_burner_launch` once routed, and for `baby_flame_impact` only after gap #2(b) is fixed).
- The dedup guard (`ReleaseFrameKey`, `already_released_frame`) is keyed on `(cue_id, node, local_frame)` and is reusable for the new skills.

### 5. clip↔atlas range parity: Baby Burner clean, Baby Flame overshoots (data observation)
Ranges in `clip.ron`: `skill: 59-75`, `heavy_attack: 23-45`.
- Baby Burner nodes (charge 23-30, launch 31-37, recovery 38-45) = union `23-45` → **exact match** with `heavy_attack`.
- Baby Flame nodes (cast 60-68, impact 69-72 +2 hold, recover **73-77**) → recover frames **76 & 77 exceed `skill` end 75**, and cast starts at 60 (frame 59 unused). A strict "every player frame ∈ skill range" parity test (the S01 idle/attack pattern) would **FAIL** on recover. The planner must decide: scope the membership assertion to cast+impact nodes, assert membership in the union of node frame ranges, or treat per-node ranges (not the coarse clip "skill" label) as the parity authority. Atlas-index identity still holds (all frames < 93), so `atlas_index(f)==Some(f)` is safe across the board.

## Implementation Landscape

**Code (lib — reachable from `tests/`):** none strictly required; the testable contract (`AtlasGeometry`, `atlas_index`, parsed RON, cue→clip-frame math) already lives in the lib (S01). Per MEM030, anything a windowed-only test must call must live in the lib, not `src/windowed/`.

**Code (windowed binary — `src/windowed/render.rs` + `mod.rs`):**
- Generalize `AgumonPlaybackMode` and the `sync_agumon_mode`/`*_barrier`/`*_start_node` seams to be skill-parameterized (cue_id, presentation_node, start_node, skill_id).
- Replace the lines 264-277 auto-release with real bridges for `baby_flame` and `agumon_ult`; keep auto-release as the fallback for genuinely unbridged skills.
- For Baby Burner, drive the multi-barrier sequence: each presentation barrier maps to a node; release at that node's `ReleaseKernel` frame (launch) or an explicit end-of-node rule (charge/recovery), firing `player.fire_kernel_cue()` to advance the KernelCue-gated transitions.
- Add `BABY_FLAME_*` / `BABY_BURNER_*` skill-id + node-name constants to `src/windowed/mod.rs`.

**Data (assets):**
- `skills.ron`: add presentation cue to Baby Flame `impact_damage`.
- `anim_graph.ron`: add `ReleaseKernel` cue to `baby_flame_impact` at the impact local frame.

**Tests (headless, `tests/animation/atlas_binding.rs` — extend S01's TC-6 pattern):**
- Baby Burner: scan `baby_burner_launch` for `ReleaseKernel`, compute `impact = start()+at = 32` (honor reverse via `clip_frame_at_cue`), assert `∈ heavy_attack [23,45]` and `atlas_index(32)==Some(32)`.
- Baby Flame: after authoring (2b), scan `baby_flame_impact` for `ReleaseKernel`, compute impact frame, assert `∈ skill [59,75]` and identity.
- Player-frame parity drives (mirroring the idle/attack tests) for baby_flame_cast→impact and the full baby_burner charge→launch→recovery sequence, firing `fire_kernel_cue()` at the KernelCue boundaries — **but scope Baby Flame membership per finding #5**.
- A windowed-mode integration assertion (under `tests/windowed_only/` or in render.rs unit tests) that a Baby Burner / Baby Flame barrier is **released by the bridge at the cue frame**, not auto-released — i.e. assert `last_release_result == Released` driven by the rendered frame, not the lines-264-277 fallback. Note the `tests/` link-only-against-lib gotcha (MEM030): a true end-to-end barrier-release assertion may need a lib-side helper or stay as a `#[cfg(test)]` unit test inside `render.rs` (the existing pattern at render.rs:515-558).

## Natural Seams / Build Order

1. **First proof / highest unblocker:** author Baby Flame presentation cue + anim `ReleaseKernel` cue (gap #2), and extend the headless impact-frame invariant tests for both Baby Flame and Baby Burner (TC-6 pattern). This proves the data contract before any render.rs churn and de-risks the "no barrier for baby flame" surprise.
2. Generalize `AgumonPlaybackMode` + the Sharp-Claws-specific seams to skill-parameterized form (no behavior change yet — Sharp Claws still green).
3. Bridge Baby Burner's three-barrier sequence (release launch on its `ReleaseKernel` frame; define explicit windup/recovery release rules); remove its auto-release.
4. Bridge Baby Flame's single impact barrier (now authored); remove its auto-release.
5. Keep the lines-264-277 auto-release only as the fallback for any still-unbridged skill.

## Verification

- `cargo test --test animation` — all S01 tests stay green + new Baby Flame / Baby Burner impact-frame + parity tests pass.
- `cargo test` (full headless suite) — green; no `windowed`-gated deps leak (R002/R005).
- `cargo build --features windowed` — green.
- `cargo test --features windowed --test windowed_only` (if a windowed-mode bridge assertion is added there).
- **K001:** auto-mode must NOT launch `cargo winx`; the smooth-animation + damage-on-impact visual for skill/ultimate on both actors is the user's manual sign-off.

## Constraints / Gotchas

- **R002/R005:** systems must run headless; egui/winit/wgpu only behind `#[cfg(feature = "windowed")]`. New skill-id/node constants and bridge logic live in the windowed layer; the testable contract stays in the lib.
- **R004:** no wall-clock/RNG in resolution; barrier timing stays in `Clock::Windowed`.
- **MEM030:** `tests/` link only the lib, not the binary — windowed-only barrier-release assertions need a lib seam or a `#[cfg(test)]` unit test inside `render.rs`.
- **MEM025:** combat kernel stays feature-agnostic and deterministic; windowed only controls *when* a queued barrier releases. Do not move release-timing decisions into `src/combat`.
- **CAP-7c065a44:** VFX via Cue/reactive bus, no physics — relevant to S03, but Baby Flame's `SpawnParticle(name: "baby_flame", ...)` on_enter and Baby Burner's cues are the seam S03 will render; S02 should not break them.
- **bevy-ecs-expert (system ordering):** `advance_agumon_presentation` is ordered `.after(resolve_action_system).before(continue_suspended_timeline_system)` — preserve this; the bridge must request release before the suspended-timeline continuation system runs in the same frame.

## Skills Discovered

No new skills installed. Relevant installed skills: `bevy-ecs-expert` (system scheduling/ordering for the render bridge), `rust-skills`/`rust-development` (idiomatic enum/match generalization of `AgumonPlaybackMode`), `cargo-nextest` (test runner). Bevy `TextureAtlasLayout`/`Sprite` API already used as-is from S01 — no doc lookup needed.

## Open Questions (resolve at planning)

- Confirm no compiled-timeline layer injects presentation onto Baby Flame impacts (would change finding #2). Grep `src/data/skill_timeline.rs` + `src/combat/runtime/` during planning.
- Baby Burner windup/recovery barrier release rule: release at end-of-node (`TimeInNode`-equivalent) vs authoring `ReleaseKernel` cues into charge/recovery. End-of-node avoids touching assets; cues make the contract explicit and headless-testable.
- Baby Flame parity-test scoping (finding #5): per-node ranges vs coarse clip `skill` range. Recommend asserting against the union of authored node frame ranges, not the clip label, to avoid baking the 76/77 overshoot into a brittle test.
