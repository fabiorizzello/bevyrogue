# S06 Architectural Review (R015 gate)

**Milestone:** M002 — First on-screen combat (Agumon-only)
**Slice:** S06 — Windowed smoke end-to-end + repomix review gate
**Date:** 2026-05-21
**Reviewer:** GSD auto-mode (M002/S06/T02)
**Pack:** `.gsd/milestones/M002/slices/S06/repomix-pack.xml`
(446 files, ~14.8 MB; built via `scripts/repomix-review.sh` — repomix@1.14.0,
`--style xml`, excludes `target/**`, `.gsd/**`, `.planning/**`, `.audits/**`,
`assets/**`, `*.lock`)

## Prompt + Scope

> "Please review the overall structure and suggest any improvements or
> refactoring opportunities, focusing on maintainability, scalability and
> extensibility."

**Files reviewed:** the entire reviewable source tree captured in the repomix
pack above. Spot-grounded reads were performed on the following high-leverage
surfaces:

- Runtime engine: `src/combat/runtime/{mod,runner,cue_barrier,post_action,registry,intent,timeline,builtins,passive_runner,signal,event_filter,applier/*}.rs`
- Pipeline: `src/combat/turn_system/pipeline/{timeline_exec,application,declaration,paths/*}.rs`
- Blueprints: `src/combat/blueprints/{mod,agumon/*,twin_core/*,…}.rs`
- Windowed assembly: `src/windowed/{mod,render}.rs`
- UI: `src/ui/{mod,phase_strip,combat_panel/*}.rs`
- Animation: `src/animation/{mod,plugin,player,anim_graph,registry,clip,validation}.rs`
- Combat root: `src/combat/mod.rs`, `src/lib.rs`
- Test layout: `tests/` (per-area subfolders + thin `*.rs` adapters per R003)

**M002 objective grounding:**
- `.gsd/milestones/M002/M002-CONTEXT.md` — first on-screen combat; player-side
  two-clock barrier; per-skill 1:1 graphs; closeout gated on this review.
- `.gsd/REQUIREMENTS.md` — R003 (test-layout / clip-geometry parity), R005
  (windowed dep gating), R006 (no .md in repo root + cue handshake on impact
  frame), R015 (this review), R016 (determinism + headless-first preserved + I3
  parity).
- `.gsd/DECISIONS.md` — D025 (two-clock: headless resolution vs windowed
  presentation), D026 (post-application reaction seam / kernel-event
  dispatch).
- `.gsd/KNOWLEDGE.md` — P003 (generic blueprint envelope + typed owner-side
  observability), P004 (canonical damage modifier fold order), P005
  (`*Snapshot::last_transition` as typed observability contract).

The review concentrates on whether the assembled M002 stack is
maintainable/scalable/extensible enough to absorb M003–M007 (more Digimon) and
a future RON editor without rewrites.

---

## Maintainability

### Module boundaries

The combat crate is partitioned along the lines the design intent demands:

- **`src/combat/runtime/`** is the kernel-side execution engine. Its `mod.rs`
  doc-comment makes the constraint explicit and re-asserts it as a no-import
  rule (`No use bevy::winit, use bevy::render, or use bevy_egui in this
  module tree`). The 18-file structure (intent, registry, signal, post-action,
  clock, cue-barrier, runner, applier/effects/*, timeline, builtins, …) keeps
  each concern in a file ≤ ~430 LOC; only `runner.rs` (426) and `cue_barrier.rs`
  (293) push the upper end and both have clear single responsibilities.
- **`src/combat/blueprints/`** is owner-scoped (agumon, dorumon, gabumon,
  patamon, renamon, tentomon, twin_core). The shared layer in `mod.rs` is a
  small dispatch table (`DispatchFn`, `BlueprintRegistration`,
  `amount_payload`) that encodes P003: routing stays generic at the envelope,
  typed contract lives inside each owner. This is the right seam for M003–M007.
- **`src/combat/turn_system/pipeline/`** sits where it should: as the
  *driver* of the runtime, not the runtime itself. `timeline_exec.rs` is the
  one large file (557 LOC) and is the natural integration point — it owns
  initiation, suspend/resume on the cue barrier, post-action dispatch (D026),
  and finalization. `paths/` (`single_target`, `multi_target`, `self_target`,
  `bounce`) separates target-shape strategies cleanly.
- **`src/windowed/`** (2 files, 813 LOC) is the only place that imports
  `bevy_egui`, `EguiPlugin`, validation soak harness, and presentation
  wiring. Combined with `src/ui/`, which gates *every* file in
  `combat_panel/` behind `#[cfg(feature = "windowed")]`, this honours R005
  dep-gating cleanly (see Findings F1 for the one residual scoping question).
- **`src/ui/`** holds presentation state derivable from `CombatEvent`
  (`PhaseStripDisplay`, `HpBarView`, `FloatingDamageView`, `TargetHurtState`,
  `TwinCoreBadgeState`). It reads truth, never writes it — matching the
  presentation-vs-truth contract.

### Test layout (R003)

`tests/` follows the per-area subfolder + thin `*.rs` adapter pattern
consistently (e.g. `tests/timeline/` has 17 files and a sibling
`timeline.rs` adapter; same for `animation/`, `damage_resolution/`,
`status_effects/`, `runtime_events_obs/`, etc.). The S05 cue-barrier and
loop-hop parity tests already live in the right place
(`tests/timeline/timeline_cue_barrier_pipeline.rs`,
`tests/timeline/timeline_loop_hop_cue_parity.rs`). No R003 violations were
observed in the pack.

### Decision discipline

- **D025 (two-clock):** `Clock { HeadlessAuto, Windowed }` is owned in
  `src/combat/runtime/clock.rs` and lifted to a per-cast Resource via
  `TimelineClock` in `cue_barrier.rs`. The kernel runner stalls on
  `StepOutcome::AwaitingCue` only when `Clock::Windowed` is selected; the
  *player side* (windowed bootstrap + animation player) is what flips the
  clock. The headless-first invariant is preserved by default
  (`TimelineClock::default()` = `HeadlessAuto`).
- **D026 (post-application reaction seam):** `post_action.rs` defines
  `PostActionContext`, `PostActionUnitDied`, `PostActionUnitSnapshot`, and
  `dispatch_post_action_reactions`. The seam is owner-neutral (no Digimon
  names), receives a roster + KO context, and emits both `Intent`s and
  `CombatKernelTransition`s. `timeline_exec.rs::dispatch_timeline_post_action`
  is the only call site, and it expands transitions through
  `CombatKernelRegistry::dispatch` before writing them as `CombatEvent`s.

### Comment / dead-code hygiene

`src/combat/runtime/mod.rs` carries a deliberate public-API facade with a
comment explaining why re-exports stay even when the library has no
in-crate consumer (`tests/` imports). `src/combat/mod.rs` opens with a
banner-comment module map that mirrors the actual layout — a maintenance
asset, not noise. No TODO/FIXME or `#[allow(dead_code)]` was observed in the
runtime tree during spot checks.

---

## Scalability

### Kernel hop cost

`BeatRunner` (in `src/combat/runtime/runner.rs`) caps loop iterations at
`MAX_HOPS = 256` (line 22) with a circuit breaker that returns
`StepOutcome::Halted` and logs `cast_id` + `timeline_id` + hop count. The
loop frame (`LoopFrame`) is a `Vec`-backed stack but the comment is clear
that depth ≤ 1 for S02 — that matches what we need for M002 multi-hit, and
the data structure already supports nesting when M005-class skills demand it.
Per-hop work in the runner is dominated by `find_beat` (HashMap lookup) and
predicate evaluation; no per-hop allocations besides the caller-owned
`pending: VecDeque<Intent>` push.

### Timeline runner allocations

`run_timeline_backed_action` constructs the `BeatRunner`, a fresh
`VecDeque::new()` for `pending`, and a `cast_hit_set: HashSet<UnitId>`
once per cast. The bottleneck for M002 is *not* allocations but the unsafe
raw-pointer dance into `ExtRegistries` (see Findings F2) — not a perf issue
but a maintainability scalability concern.

### Asset-driven kit data

`AnimationGraphPaths` and `AnimationClipPaths` (in
`src/animation/plugin.rs`) are `Resource`-typed `Vec<String>` lists with
`Default` impls keyed off `DEFAULT_ANIM_GRAPH_PATHS` /
`DEFAULT_ANIM_CLIP_PATHS`. Adding a new Digimon for M003–M007 is one entry
per list + the RON files; no Rust changes to the animation crate are
required to load assets. `SkillBook` / RON hot-reload via
`bevy_common_assets::ron::RonAssetPlugin` is already wired in the same
plugin.

### M003–M007 readiness

- Blueprint dispatch is plugin-shaped (P003): adding `gabumon`, `dorumon`,
  etc., is local to each owner module + one `BlueprintRegistration` entry.
- Per-skill 1:1 graph granularity (CONTEXT D004-class) means new skills
  scale linearly with file count, not with cross-cutting Rust changes.
- The `paths/` strategy split (single/multi/self/bounce) already covers the
  shapes the documented kits use; new shapes are additive.
- One scalability caveat: the **windowed bootstrap presets** in
  `src/windowed/mod.rs` hard-code Agumon-specific constants
  (`AGUMON_STANCE_GRAPH_ID`, `AGUMON_SKILL_GRAPH_ID`, `SHARP_CLAWS_*`).
  These are deliberately scoped to M002 but should be lifted into the
  encounter/preset layer before M003 (see Findings F3).

---

## Extensibility

### Animation/skill seam (AnimGraph + timeline + cue barrier)

The seam is symmetric and complete:

- `AnimGraph` (RON-loaded, `src/animation/anim_graph.rs`) carries `cues:
  Vec<FrameCue>` per node. The anti-DRY invariant is enforced by an
  executable test in `tests/animation/anim_gameplay_command_forbidden.rs`.
- `BeatRunner::AwaitingCueInfo` exposes `{beat_id, cue_id, animation_node,
  hop_index}` so the player can correlate a stall to an animation node
  without back-channels.
- `request_timeline_cue_release(world, cue_id)` is the one public
  in-process API (`src/combat/runtime/cue_barrier.rs:288`). It is total:
  `Released | DuplicateRelease | NoSuspendedTimeline | CueMismatch` are
  enumerated outcomes, every branch logs at an appropriate level, and the
  last result/status is inspectable via `SuspendedTimelineState`.

### Two-clock contract (D025)

`TimelineClock(Clock)` is a `Resource`, and the runner only stalls when the
clock is `Windowed`. `timeline_exec::run_timeline_backed_action` reads the
clock once per cast and threads it via `BeatRunner::with_clock`. Headless
tests can leave the resource at default to get the auto-completing
behaviour; the windowed bootstrap flips it at app start. This is the right
direction of control flow (player owns the barrier).

### Post-application reaction seam (D026)

`dispatch_post_action_reactions` consumes a `PostActionContext` and returns
a `PostActionQueue { intents, transitions }`. The dispatch is *immediately*
followed by `intent_applier(world)` and then a registry expansion of
transitions into `CombatEvent::OnKernelTransition`. This lets blueprints
(Baby Burner reactive detonate, twin_core counterplay, …) extend behaviour
without touching the pipeline.

### RON editor readiness

- Skills, anim graphs, clips, and signal taxonomies are all RON-typed and
  re-loadable via `RonAssetPlugin`.
- `validate_timeline_refs`, `validate_anim_graph`,
  `AnimationValidationReport`, and `DanglingTimelineRefs` are public —
  any editor can validate before saving.
- `GameplayCommandForbidden` is an executable seam, not a lint comment.
- The closed serde enums (`Intent`, `BeatKind`, `BeatPayload`, `Presentation`,
  `CustomSignalPayload`) give an editor an exhaustive schema.

### Blueprint envelope (P003)

`blueprints/mod.rs` defines a small, generic envelope
(`SkillCustomSignal`, `CustomSignalDispatchError`, the `DispatchFn`
function-pointer table) while each owner module preserves *typed*
contracts internally (e.g. agumon's `baby_burner` signals). This is the
P003 pattern in practice and is already extensible to the next owners.

### Modifier ledger fold order (P004)

The damage applier
(`src/combat/runtime/applier/effects/damage.rs`) is the canonical fold
site. The fold order is documented in P004 (Intrinsic → Status → Buff →
Passive) and the relevant tests live under
`tests/damage_resolution/`. No cross-cutting refactor is required to add
M003–M007 modifier sources — they slot into the existing fold.

### Typed observability contracts (P005)

`*Snapshot::last_transition` fields on blueprint-side snapshots are
treated as part of the public contract for ValidationExt rows and JSONL
transition logging. Tests guard them. Adding new owners simply requires
the same typed-snapshot shape.

---

## Findings

> Numbered, severity-tagged, real-path-grounded. Locations refer to files
> present in the reviewed repomix pack.

### F1 — Windowed presets hard-code Agumon identifiers
- **Severity:** low
- **Location:** `src/windowed/mod.rs:35-40` (`AGUMON_STANCE_GRAPH_ID`,
  `AGUMON_SKILL_GRAPH_ID`, `SHARP_CLAWS_SKILL_ID`,
  `SHARP_CLAWS_WINDUP_NODE`, `SHARP_CLAWS_STRIKE_NODE`)
- **Rationale:** These constants are correct for M002 (Agumon-only), but
  the bootstrap path is the entry point that M003–M007 will extend. Each
  added Digimon will tempt a copy-paste of the same const block in the
  same file, growing `windowed/mod.rs` linearly per owner.
- **Suggested action:** Lift the per-owner graph/skill/node IDs into the
  owner blueprint modules (e.g. `combat::blueprints::agumon::ids`)
  re-exported via a small `WindowedCharacterPreset` trait or `&'static`
  table consumed by `bootstrap_encounter`. Keep `windowed/mod.rs` free
  of per-character constants. Defer to M003/S01 if M002 closeout is
  immediate; capture as a follow-up.

### F2 — `unsafe` raw-pointer dance into `ExtRegistries`
- **Severity:** medium
- **Location:** `src/combat/turn_system/pipeline/timeline_exec.rs:85-91,
  122-130, 149-161, 504-540, 543-557` (`resolve_regs_ptr` +
  `unsafe { &*regs_ptr }` call sites)
- **Rationale:** `resolve_regs_ptr` returns a `*const ExtRegistries`
  either into the live `World` or into a `fallback_regs: Option<…>` held
  on the caller's stack, and the `unsafe` blocks then dereference it
  while the same `world` is being mutated (the runner pushes to
  `IntentQueue`, dispatches reactions, writes `CombatEvent`s). The
  safety is real today because the read-only registry is borrowed
  immutably and the mutable world borrows are taken/released around
  each call, but the contract is fragile: any reviewer adding a system
  that mutates `ExtRegistries` (or a new pipeline branch that holds the
  pointer across an `intent_applier` call that itself touches the
  registry) silently breaks aliasing rules. M003–M007 will add reaction
  blueprints; this is exactly when the seam will be touched again.
- **Suggested action:** Replace the raw-pointer wrapper with one of (a)
  cloning `ExtRegistries` once per cast into a local
  (`ExtRegistries` is plain data — verify it's `Clone`), or (b)
  extracting the registry via `world.remove_resource` + reinsert in a
  `Drop` guard, or (c) shaping the registry access as a small immutable
  view passed by reference into the runner. Option (a) is the lowest
  effort and removes the only `unsafe` blocks in the pipeline.

### F3 — `timeline_exec.rs` is the de-facto integration "god file"
- **Severity:** low
- **Location:** `src/combat/turn_system/pipeline/timeline_exec.rs` (557
  LOC) — single file owns initiation, preflight, suspend, resume,
  failure, finalization, post-action dispatch, runtime-resource priming,
  and the `unsafe` registry seam.
- **Rationale:** It's the only file in `pipeline/` over 200 LOC and is
  doing five distinct things (start / preflight / suspend / resume /
  finalize) plus the post-action seam from D026. Each future feature
  (new clock mode, new reaction shape, new failure surface) is going to
  land here. Maintainability is fine today but the file will continue
  to grow with each milestone unless split.
- **Suggested action:** Split into sibling files inside
  `pipeline/timeline/`:
  `start.rs` (`run_timeline_backed_action`, preflight),
  `resume.rs` (`continue_suspended_timeline*`),
  `finalize.rs` (`finalize_timeline_action`,
  `prepare_timeline_intent_runtime`, ult/SP accounting),
  `post_action.rs` (`dispatch_timeline_post_action`),
  `errors.rs` (`fail_*`, `preflight_fail_*`). Mechanical, no behaviour
  change. Pair with F2 to clean up `resolve_regs_ptr` in one PR.

### F4 — Unbounded `Box::leak` of custom signal owner/name strings on every cast
- **Severity:** low
- **Location:** `src/combat/turn_system/pipeline/timeline_exec.rs:443-447`
  (`prepare_timeline_intent_runtime` — leaks `owner` and `signal`
  strings for every signal in the SkillBook fallback path, on every
  cast).
- **Rationale:** This path is the *fallback* used when an
  `ExtRegistries` resource is not already present, but
  `prepare_timeline_intent_runtime` runs once per cast. If the fallback
  path is ever exercised in the soak (e.g. a setup omits the registry,
  or a hot-reload re-enters), the leak grows unbounded for the
  process lifetime. For headless tests this is harmless; for the
  long-running windowed soak harness in `WindowedValidationConfig`, it
  is a latent leak.
- **Suggested action:** Pre-register the SignalTaxonomy once at app
  build time (in the kernel plugin) from the loaded SkillBook, using
  string interner storage owned by `SignalTaxonomy` (or `Arc<str>`)
  rather than `Box::leak`. The fallback path then becomes a no-op
  after first registration.

### F5 — Single `bevy::log::info!` per cue barrier event in a hot path
- **Severity:** info
- **Location:** `src/combat/runtime/cue_barrier.rs:166, 266` (and the
  surrounding `info!` calls in `suspend`/`request_release`).
- **Rationale:** Every suspend and accepted release writes a formatted
  `info!` line. Under a multi-hit skill the runner may stall N times
  per cast (hop_index 0..N-1). For M002 (Agumon dummy, ~3 skills,
  ≤ a handful of hops) this is fine, but under M005-class long loops
  in the soak it produces O(hops·skills·seconds) log volume.
- **Suggested action:** Demote `suspend`/`request_release` accepted
  paths from `info!` to `debug!`; keep `warn!` for `CueMismatch` and
  `info!` only on the *first* suspend per cast. No structural change.

### F6 — `src/combat/mod.rs` line "Recovered from git history" is leftover doc noise
- **Severity:** info
- **Location:** `src/combat/mod.rs` (overall file is healthy; banner
  comments are valuable) — and `.gsd/REQUIREMENTS.md` rows mentioning
  "Recovered from git history (commit 647a8af)" are scattered in the
  requirements file rather than src. No src-level dead comments were
  observed in the spot read.
- **Rationale:** Confirms that the in-tree comments are intentional,
  not stale TODOs. Logged only to certify the scan covered
  hygiene.
- **Suggested action:** None for src/. Optional: trim
  recovered-history annotations in REQUIREMENTS.md once M002 closes.

### F7 — `windowed/mod.rs` system-set wiring scales by per-UI-resource init
- **Severity:** info
- **Location:** `src/windowed/mod.rs:56-96` (`UiPlugin::build` —
  init_resource × 8 followed by a flat chain of 8 systems, then four
  `EguiPrimaryContextPass` adders).
- **Rationale:** Each new presentation resource for M003–M007 (new
  badges, new HP bar shapes, new chips) will append another
  `init_resource` + system to this chain. With 8 already chained,
  ordering correctness becomes harder to reason about.
- **Suggested action:** Group the presentation systems into named
  `SystemSet`s (e.g. `PresentationCompute`, `PresentationRender`) and
  configure `.chain()` between sets rather than between individual
  systems. Mechanical, additive.

### Hygiene scans (no findings, recorded for the gate)
- **No .md files in repo root (R006):** `ls /home/fabio/dev/bevyrogue/*.md`
  returns empty. PASS.
- **No winit/wgpu/egui leakage outside `windowed` (R005):** `grep`
  across `src/` returns matches only in `src/windowed/mod.rs`,
  `src/ui/combat_panel/{labels,render,widgets}.rs`,
  `src/ui/phase_strip.rs`, and one *comment* in
  `src/combat/runtime/mod.rs` (the prohibition itself).
  `src/ui/combat_panel/mod.rs` gates *every* sub-module behind
  `#[cfg(feature = "windowed")]`. PASS.
- **R003 test layout:** every `tests/<area>/` has a sibling
  `tests/<area>.rs` adapter; no flat-file regressions observed. PASS.
- **Presentation-vs-truth (R016 / I3):** the UI side
  (`src/ui/combat_panel/*`) reads `EventReader<CombatEvent>` and
  writes only to view-typed Resources (`HpBarView`,
  `FloatingDamageView`, `TargetHurtState`, `TwinCoreBadgeState`,
  `PhaseStripDisplay`). No write-back into kernel state was
  observed. PASS.

---

## Verdict

**pass-with-followups.**

The M002 assembled stack meets the maintainability, scalability, and
extensibility bar for first on-screen combat and is structurally ready for
M003–M007 plus a RON editor. The two-clock barrier, post-action seam,
blueprint envelope, and animation/skill cue handshake are all on the right
seams. R005/R006/R003 invariants hold in the reviewed pack. No
needs-remediation findings were identified.

### Follow-ups (suggested owners)

| ID  | Severity | Suggested owner / milestone | Item                                                                          |
| --- | -------- | --------------------------- | ----------------------------------------------------------------------------- |
| F1  | low      | M003/S01 bootstrap          | Lift per-owner graph/skill/node IDs out of `src/windowed/mod.rs`              |
| F2  | medium   | M002 close-of-cycle or M003/S01 | Remove `unsafe` raw-pointer registry dance in `timeline_exec.rs`         |
| F3  | low      | M003/S01 housekeeping       | Split `timeline_exec.rs` (557 LOC) into `pipeline/timeline/{start,resume,finalize,post_action,errors}.rs` |
| F4  | low      | M003/S01 plugin wiring      | Pre-register `SignalTaxonomy` at app build to drop the per-cast `Box::leak` fallback |
| F5  | info     | Anytime                     | Demote per-suspend/release `info!` to `debug!` in `cue_barrier.rs`            |
| F7  | info     | M003/S01 UI grouping        | Group `UiPlugin` systems into named `SystemSet`s with set-level `.chain()`    |

F2 is the only finding above "low"; it is medium because the `unsafe` is
fragile under future blueprint additions, but the current safety contract
is sound and no soundness bug was observed. Recommend triaging F2 as the
first item in M003/S01 housekeeping if M002 closes immediately.
