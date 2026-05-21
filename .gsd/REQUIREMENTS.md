# Requirements

This file is the explicit capability and coverage contract for the project.

## Active

### R004 — AnimGraph runtime player + wgpu sprite render: Agumon's stance graph drives an on-screen sprite via the typed AnimGraph schema (not hardcoded). Introduces the RenderPlugin/UiPlugin split gated #[cfg(feature="windowed")].
- Class: core-capability
- Status: active
- Description: AnimGraph runtime player + wgpu sprite render: Agumon's stance graph drives an on-screen sprite via the typed AnimGraph schema (not hardcoded). Introduces the RenderPlugin/UiPlugin split gated #[cfg(feature="windowed")].
- Why it matters: M001 produced data, not behavior. Nothing draws a sprite. This is the foundation every later slice builds on; without it the milestone has no visible output.
- Source: user
- Primary owning slice: M002/S01
- Validation: cargo run --features windowed shows Agumon cycling idle via the stance graph; M001 headless tests stay green; clip↔atlas geometry parity test present and passing.
- Notes: Repomix validation: RenderPlugin/UiPlugin split, Predicate::KernelCue, cues/ReleaseKernelCue, and the graph registries are all NEW M002 deliverables in S01, not pre-existing. animation/plugin.rs is at 494/500 LOC — must be pre-split before sprite render is added.

### R005 — Per-digimon Stance FSM (idle/hurt/death/victory) authored in the same AnimGraph RON schema, in a separate file, linked by skill-id via a registry. Default graph when no skill is active; a skill's Exit returns the sprite to stance.
- Class: core-capability
- Status: active
- Description: Per-digimon Stance FSM (idle/hurt/death/victory) authored in the same AnimGraph RON schema, in a separate file, linked by skill-id via a registry. Default graph when no skill is active; a skill's Exit returns the sprite to stance.
- Why it matters: Separating stance from per-skill graphs keeps files small, DRY across the 6-digimon roster, and editor-openable one graph at a time — directly serves the user's architecture mandate and unblocks M003+.
- Source: user
- Primary owning slice: M002/S01
- Validation: Agumon stance graph loads as a distinct RON asset; skill Exit returns sprite to stance idle; target blink/hurt reactions driven by CombatEvent on the target's stance, never authored in the attacker's skill graph.
- Notes: SkillGraphRegistry/StanceGraphRegistry are NEW S01 deliverables (no registry exists; AnimGraph asset currently has no id field).

### R006 — Two-clock impact sync: damage lands on the visible impact frame via a player-side barrier. The AnimGraph player holds the barrier and calls resume_cue() when the clip reaches the authored frame; the kernel stays frame-ignorant, its resolution clock gated waiting for the released Intent.
- Class: core-capability
- Status: active
- Description: Two-clock impact sync: damage lands on the visible impact frame via a player-side barrier. The AnimGraph player holds the barrier and calls resume_cue() when the clip reaches the authored frame; the kernel stays frame-ignorant, its resolution clock gated waiting for the released Intent.
- Why it matters: This is the defining "does it even work" risk of the whole portfolio — the first proof the two-clock model holds in presentation, not just headless.
- Source: user
- Primary owning slice: M002/S02
- Validation: Sharp Claws windup→strike→recovery on screen; damage falls on the impact frame via ReleaseKernelCue; invariant I3 extended to the new handshake stays green (identical Intent stream headless vs windowed, only timing differs).
- Notes: Repomix: Clock does NOT yet stall the turn pipeline (timeline_exec.rs:118 auto-resumes; windowed never suspends). Wiring Clock into the turn pipeline (run_to_completion → per-frame step() suspending CombatPhase on AwaitingCue) is the true S02 risk and a NEW deliverable.

### R007 — Gameplay/presentation seam: zero gameplay numbers in anim_graph.ron. EmitDamage/EmitStatus/EmitHeal are never authored in the AnimGraph; damage, hit count and loop budget live only in the kernel (skills.ron beats). Enforced by an executable anti-DRY validation test.
- Class: constraint
- Status: active
- Description: Gameplay/presentation seam: zero gameplay numbers in anim_graph.ron. EmitDamage/EmitStatus/EmitHeal are never authored in the AnimGraph; damage, hit count and loop budget live only in the kernel (skills.ron beats). Enforced by an executable anti-DRY validation test.
- Why it matters: The M001 mul:18 duplicate proved authored gameplay numbers in the anim graph silently diverge from the kernel. Turning the seam into an executable invariant is the architecture mandate made enforceable.
- Source: user
- Primary owning slice: M002/S02
- Validation: A test fails if anim_graph.ron contains any gameplay Command (EmitDamage/EmitStatus/EmitHeal); the M001 mul:18 duplicate at agumon/anim_graph.ron:20 is remediated behind that test.
- Notes: GameplayCommandForbidden validation check + test is a NEW S01 deliverable (command.rs validation does not currently forbid it).

### R008 — Per-skill graph 1:1 with the kernel CompiledTimeline (same skill-id = natural sync point), loaded by skill-id via registry. RON schema is the editor contract: nodes + edges + cues, parametric per-cast via typed graph input.
- Class: differentiator
- Status: active
- Description: Per-skill graph 1:1 with the kernel CompiledTimeline (same skill-id = natural sync point), loaded by skill-id via registry. RON schema is the editor contract: nodes + edges + cues, parametric per-cast via typed graph input.
- Why it matters: A per-skill graph mapped 1:1 to the kernel timeline is the DRY sync mechanism between the two runtimes and keeps a future RON editor able to open one graph at a time. A mega-FSM would break the alignment and duplicate ×6.
- Source: user
- Primary owning slice: M002/S01
- Validation: CompiledTimeline.id = skill_id confirmed (skill_timeline.rs:73); animation-side registry resolves skill-id→graph with zero if-else dispatch.
- Notes: cues: Vec<FrameCue> on AnimNode and ReleaseKernelCue are NEW S01 deliverables (AnimNode currently has only frames/on_enter/modifier/reverse).

### R010 — §9 phase strip (turn order) live, driven by EventReader<CombatEvent>; the UI never mutates combat state (D008 enforced structurally, not by convention).
- Class: primary-user-loop
- Status: active
- Description: §9 phase strip (turn order) live, driven by EventReader<CombatEvent>; the UI never mutates combat state (D008 enforced structurally, not by convention).
- Why it matters: The turn-order strip is the core readable combat UI; honoring "UI observes, never mutates" from the green field keeps the kernel authoritative.
- Source: user
- Primary owning slice: M002/S03
- Validation: Phase strip updates from EventReader<CombatEvent>; a structural test asserts the UI path never mutates combat state.
- Notes: Nothing reads EventReader<CombatEvent> today — D008 is a green-field constraint honored from the start.

### R011 — Full Agumon kit playable vs an Agumon dummy: Sharp Claws + Baby Flame + Baby Burner + Twin Core fire side (placeholder ally for the Heated trigger). Multi-hit loop is visible (N repetitions = N kernel hops, no N authored in the anim graph).
- Class: core-capability
- Status: active
- Description: Full Agumon kit playable vs an Agumon dummy: Sharp Claws + Baby Flame + Baby Burner + Twin Core fire side (placeholder ally for the Heated trigger). Multi-hit loop is visible (N repetitions = N kernel hops, no N authored in the anim graph).
- Why it matters: This is the milestone's user-visible payoff — the full kit on screen — and the proof the per-hit handshake and kernel-owned loop budget compose correctly.
- Source: user
- Primary owning slice: M002/S05
- Validation: Agumon vs dummy Agumon at full kit on screen; multi-hit loop visibly = kernel hop count; target blink/hurt driven by CombatEvent.
- Notes: DamageCurve::PerHop exists only in the data layer (skills_ron/types.rs:203); kernel hop-budget wiring is a NEW S05 deliverable.

### R012 — VFX is an opaque Id handle resolved to a VFX entity configured in Rust code in M002 (a placeholder flash for Baby Burner). Internal VFX phases are owned by the VFX entity on its own clock; only the single gameplay impact cue is frame-synced. The seam is left open so bevy_enoki / Omagari (RON + editor) can back it later without touching anim graph or kernel.
- Class: integration
- Status: active
- Description: VFX is an opaque Id handle resolved to a VFX entity configured in Rust code in M002 (a placeholder flash for Baby Burner). Internal VFX phases are owned by the VFX entity on its own clock; only the single gameplay impact cue is frame-synced. The seam is left open so bevy_enoki / Omagari (RON + editor) can back it later without touching anim graph or kernel.
- Why it matters: Keeping VFX an opaque handle now avoids inventing a custom format/editor in M002 while preserving extensibility for a future RON VFX pipeline.
- Source: user
- Primary owning slice: M002/S04
- Validation: Baby Burner reactive detonate + flash VFX via Rust-configured entity; no custom RON/editor; swapping the backing implementation would not touch anim graph or kernel.
- Notes: bevy_enoki / Omagari evaluation is explicitly outside M002.

### R014 — Windowed smoke end-to-end: cargo run --features windowed runs Agumon vs dummy Agumon at full kit with no panic, stable FPS (qualitative — no perceptible stutter, no unbounded VFX entity growth), and hot-reload mid-skill does not corrupt world state.
- Class: launchability
- Status: active
- Description: Windowed smoke end-to-end: cargo run --features windowed runs Agumon vs dummy Agumon at full kit with no panic, stable FPS (qualitative — no perceptible stutter, no unbounded VFX entity growth), and hot-reload mid-skill does not corrupt world state.
- Why it matters: This is the launchable proof the assembled runtime stack actually works in a real environment, not just in fixtures.
- Source: user
- Primary owning slice: M002/S06
- Validation: Operational UAT with captured console output (not just a documented procedure): a windowed session showing no panic, stable FPS, and hot-reload mid-skill leaving world state intact.
- Notes: Memory constraint: a procedure without captured evidence does not satisfy the Operational verification class.

### R015 — Repomix architectural review gate as final milestone validation: at M002 closeout, pack the source with repomix and review code + M002 artifacts against the implementation objective and CONTEXT's technical/architectural spec, with the prompt "Please review the overall structure and suggest any improvements or refactoring opportunities, focusing on maintainability, scalability and extensibility." Re-runnable per-slice; the produced report is attached as S06 evidence.
- Class: operability
- Status: active
- Description: Repomix architectural review gate as final milestone validation: at M002 closeout, pack the source with repomix and review code + M002 artifacts against the implementation objective and CONTEXT's technical/architectural spec, with the prompt "Please review the overall structure and suggest any improvements or refactoring opportunities, focusing on maintainability, scalability and extensibility." Re-runnable per-slice; the produced report is attached as S06 evidence.
- Why it matters: The user explicitly wants an external structural review grounded in packed code as the milestone's final architectural acceptance — it caught 4 phantom-capability assumptions during planning and must guard closeout too.
- Source: user
- Primary owning slice: M002/S06
- Validation: A repomix-grounded architectural review report (maintainability/scalability/extensibility) is produced at closeout and attached as S06 evidence; findings are triaged before milestone completion.
- Notes: npx repomix is available. This is a milestone validation gate, not a product slice.

### R016 — Determinism and headless-first preserved: R002 (headless-first), R004 (determinism), R005 (windowed dep-gating), R006 (no .md in repo root) all hold; all M001 headless Agumon tests stay green; invariant I3 (two-clock parity) is extended to cover the new cue handshake.
- Class: constraint
- Status: active
- Description: Determinism and headless-first preserved: R002 (headless-first), R004 (determinism), R005 (windowed dep-gating), R006 (no .md in repo root) all hold; all M001 headless Agumon tests stay green; invariant I3 (two-clock parity) is extended to cover the new cue handshake.
- Why it matters: M002 introduces the windowed runtime; the entire value of the kernel depends on presentation never corrupting deterministic gameplay truth.
- Source: inferred
- Primary owning slice: M002/S06
- Validation: cargo test green (M001 suite intact); extended I3 parity test green; no winit/wgpu/egui deps outside windowed; no .md added to repo root.
- Notes: Supporting slices: S01–S05 each keep their headless tests green as a per-slice gate.

## Validated

### R003 — Untitled — clip geometry parity validation.
- Class: quality-attribute
- Status: validated
- Description: Untitled — clip geometry parity validation.
- Why it matters: Clip.ron geometry must match the authoritative atlas exactly; silent authoring errors corrupt frame references at runtime.
- Source: inferred
- Validation: cargo test --test clip_geometry_parity passes (1 test: agumon_clip_ron_matches_authoritative_atlas_geometry). clip.ron corrected to frame_size w=512/h=512, total_frames=93, ranges heavy_attack 23-45, hurt 46-52, idle 53-58, skill 59-75, victory 76-92 — matching agumon_atlas.json exactly. anim_graph.ron node frame references updated to match. Validated in M001 remediation before milestone completion (2026-05-19).
- Notes: Pre-existing S02 geometry regression (w=557,h=561,total_frames=95) corrected in working tree before M001 milestone closeout. The clip_geometry_parity test is the only guard against silent authoring errors; it must be run and pass before any clip authoring task is marked complete (see MEM010, MEM013).

### R009 — Typed graph input (xstate input lens): the AnimGraph is a pure function of (typed input, kernel cue, frame clock) and never reads world globals. Input is a closed Role enum (Caster, PrimaryTarget, …) injected read-only by the kernel at cast; cues/guards reference roles, never literals/numbers/coordinates.
- Class: quality-attribute
- Status: validated
- Description: Typed graph input (xstate input lens): the AnimGraph is a pure function of (typed input, kernel cue, frame clock) and never reads world globals. Input is a closed Role enum (Caster, PrimaryTarget, …) injected read-only by the kernel at cast; cues/guards reference roles, never literals/numbers/coordinates.
- Why it matters: Pure-function-of-input makes the graph deterministic, testable, and a known contract for the future editor — the flexibility the user wanted from xstate without xstate's mutable context.
- Source: user
- Primary owning slice: M002/S02
- Validation: Verified by cargo test --test animation anim_graph_input_purity and the windowed regression sweep. Tests prove AnimGraph evaluation uses a closed typed AnimGraphRole/AnimGraphInput seam, rejects stringly or unknown roles, and keeps player advancement behaviorally equivalent without any world-global or mutable graph-context read path.
- Notes: S08 supplied executable purity proof for the typed input lens and preserved legacy wrappers as default-input shims.

### R013 — Failure visibility: cue-never-released → safety timeout in frame-budget forces resume_cue + structured error log (graph/cue/node/frame); missing skill-id → strict-on-boot for the known M002 set, runtime fallback to a degenerate instant graph + error log; hot-reload mid-skill takes effect at next spawn; target dead mid-loop → presentation completes without branching on liveness. No silent swallow; combat headless stays authoritative.
- Class: failure-visibility
- Status: validated
- Description: Failure visibility: cue-never-released → safety timeout in frame-budget forces resume_cue + structured error log (graph/cue/node/frame); missing skill-id → strict-on-boot for the known M002 set, runtime fallback to a degenerate instant graph + error log; hot-reload mid-skill takes effect at next spawn; target dead mid-loop → presentation completes without branching on liveness. No silent swallow; combat headless stays authoritative.
- Why it matters: A deadlocked AwaitingCue with no signal is the worst failure for an unattended runtime; every failure mode must be noisy and non-corrupting.
- Source: user
- Primary owning slice: M002/S02
- Validation: Verified by cargo test --test timeline r013_failure_visibility, cargo test --test animation anim_registry_failure_visibility, and cargo test --features windowed --test animation --test timeline --test windowed_only. Tests prove cue timeout force-resume with structured diagnostics, missing skill graph runtime fallback plus boot-time load-state visibility, hot reload applying only to newly spawned/resolved players, and dead-target mid-loop remaining observable without branching presentation flow on liveness.
- Notes: S08 hardened structured diagnostic surfaces across cue barriers, animation registry fallback, boot load-state reporting, and post-KO overshoot observability.

### R021 — The animation system must live behind one cohesive generic module boundary covering schema, loading, validation, orchestration, and future runtime/player behavior without hardcoded Digimon-specific logic.
- Class: quality-attribute
- Status: validated
- Description: The animation system must live behind one cohesive generic module boundary covering schema, loading, validation, orchestration, and future runtime/player behavior without hardcoded Digimon-specific logic.
- Why it matters: A clean module seam prevents early asset-pipeline decisions from coupling the future animation runtime to individual Digimon or one-off content assumptions.
- Source: user
- Primary owning slice: M001/S01
- Validation: Validated in M001: animation module boundary established in src/animation/ with no Digimon-specific logic in the core module.
- Notes: Recovered from git history (commit 647a8af). Original R001 from M001 planning. User explicitly requested one animation FSM module and a strict separation between motore and specificità Digimon.

### R022 — The project must load anim_graph.ron as a typed Bevy asset through the animation module using closed schema types for graph nodes, edges, predicates, commands, parameter references, and target shapes.
- Class: core-capability
- Status: validated
- Description: The project must load anim_graph.ron as a typed Bevy asset through the animation module using closed schema types for graph nodes, edges, predicates, commands, parameter references, and target shapes.
- Why it matters: Later visual runtime work depends on a validated animation graph contract rather than ad hoc stringly-typed asset data.
- Source: user
- Primary owning slice: M001/S01
- Validation: Validated in M001: anim_graph.ron loads as typed Bevy asset via AnimationAssetPlugin; closed RON schema types in src/animation/anim_graph.rs.
- Notes: Recovered from git history (commit 647a8af). Original R002 from M001 planning. Seeded from M022 S01, adapted so the owner module is animation rather than a Digimon-specific or blueprint-specific seam.

### R023 — Invalid animation assets at boot must fail fast with typed diagnostics instead of being deferred to runtime behavior.
- Class: continuity
- Status: validated
- Description: Invalid animation assets at boot must fail fast with typed diagnostics instead of being deferred to runtime behavior.
- Why it matters: Bad animation data should be caught when assets load so later runtime and rendering work is not debugging schema mistakes indirectly.
- Source: user
- Primary owning slice: M001/S03
- Validation: Validated in M001: boot-time validation with typed diagnostics in src/animation/validation/; names offending file/check where practical.
- Notes: Recovered from git history (commit 647a8af). Original R004 from M001 planning (before DB reset reassigned that ID to M002). Advisory checks remain warnings.

### R024 — Cross-asset animation validation must prove references against real project data through explicit adapter seams rather than hard-coupling the animation core to Digimon or gameplay internals.
- Class: integration
- Status: validated
- Description: Cross-asset animation validation must prove references against real project data through explicit adapter seams rather than hard-coupling the animation core to Digimon or gameplay internals.
- Why it matters: The milestone needed real validation strength without turning the generic animation engine into a dependency sink for all game-specific data.
- Source: user
- Primary owning slice: M001/S03
- Validation: Validated in M001: cross-asset validation via adapter seams; parameter references, clip names and other catalogs supplied by adapters, not hardcoded in the validator.
- Notes: Recovered from git history (commit 647a8af). Original R005 from M001 planning (before DB reset reassigned that ID to M002).

### R025 — M001 completion must include a real manual cargo run --features windowed hot-reload proof showing edited animation assets reload without crash or corrupted world state.
- Class: launchability
- Status: validated
- Description: M001 completion must include a real manual cargo run --features windowed hot-reload proof showing edited animation assets reload without crash or corrupted world state.
- Why it matters: Hot reload is an operational authoring capability and cannot be fully proven by headless contract tests alone.
- Source: user
- Primary owning slice: M001/S04
- Validation: Validated in M001: manual windowed hot-reload demo completed; invalid hot reload keeps last valid asset and logs clearly.
- Notes: Recovered from git history (commit 647a8af). Original R006 from M001 planning. User confirmed the manual demo is required, not optional.

### R026 — The architecture must support the non-Agumon roster from the start through the same generic validation/loading path, rather than treating non-Agumon as careless one-off stubs.
- Class: quality-attribute
- Status: validated
- Description: The architecture must support the non-Agumon roster from the start through the same generic validation/loading path, rather than treating non-Agumon as careless one-off stubs.
- Why it matters: A pipeline that only works for Agumon risks baking content-specific assumptions into the engine at the exact point the module boundary is being established.
- Source: user
- Primary owning slice: M001/S04
- Validation: Validated in M001: Renamon animation assets authored and validated through the same generic path; no Agumon-specific assumptions in the loader.
- Notes: Recovered from git history (commit 647a8af). Original R007 from M001 planning. User asked to be stronger for non-Agumon to get good architecture from the beginning.

### R027 — Animation asset loading and validation must remain headless-first, with windowed used only for the live hot-reload demo and any UI-dependent behavior.
- Class: constraint
- Status: validated
- Description: Animation asset loading and validation must remain headless-first, with windowed used only for the live hot-reload demo and any UI-dependent behavior.
- Why it matters: The project relies on deterministic, agent-friendly command verification and must not make asset validation depend on a graphical runtime.
- Source: inferred
- Primary owning slice: M001/S03
- Supporting slices: M001/S01, M001/S02, M001/S04
- Validation: Validated in M001: all validation runs headless; windowed gated by #[cfg(feature = windowed)]; no winit/wgpu/egui deps outside the feature gate.
- Notes: Recovered from git history (commit 647a8af). Original R008 from M001 planning. Carries forward project rules R002/R005.

### R028 — Recovered M015 validated baseline: preserve the validated M015 Combat Authority Closure contract covering deterministic headless verification, canonical combat authority boundaries, shared combat surfaces (CombatEvent, OnCombatBeat, OnKernelTransition, ValidationSnapshot), and truthful supersession of incomplete M013 closure evidence.
- Class: core-capability
- Status: validated
- Description: Recovered M015 validated baseline: preserve the validated M015 Combat Authority Closure contract covering deterministic headless verification, canonical combat authority boundaries, shared combat surfaces (CombatEvent, OnCombatBeat, OnKernelTransition, ValidationSnapshot), and truthful supersession of incomplete M013 closure evidence.
- Why it matters: This baseline defines the last validated pre-M001 combat contract and should remain queryable in the DB rather than only in git history.
- Source: git-history 00c0812
- Validation: Validated baseline from recovered historical requirements snapshot; evidence in .gsd/milestones/M015/M015-VALIDATION.md, docs/combat_current.md, and M015 contract docs. Validated requirements: R086, R088, R089-R100 (historical numbering).
- Notes: Recovered from git history (commit 647a8af / 00c0812). Original R013 from post-M015 planning. Establishes: green deterministic headless verification baseline; failure-ledger-first repair; generic branch-light kernel; RON as declarative data layer (non-authoritative for gameplay logic); trait Skill + SkillCtx target (D010).

## Deferred

### R029 — Runtime animation player and FSM execution — intentionally deferred until after the typed asset and validation contract (M001) was proven.
- Class: core-capability
- Status: deferred
- Description: Runtime animation player and FSM execution — intentionally deferred until after the typed asset and validation contract (M001) was proven.
- Why it matters: Building runtime playback before the asset contract is proven would blur milestone risk and make schema issues harder to isolate.
- Source: user
- Notes: Recovered from git history (commit 647a8af). Original R009 from M001 planning. Now addressed by M002 (R004-R016). Preserved as historical record: no tick_fsm, runtime player, or playback system in M001 unless required for validation seams.

### R030 — Command-to-gameplay runtime translation from animation graph commands into gameplay/kernel effects — deferred until the runtime/player milestone (M002).
- Class: integration
- Status: deferred
- Description: Command-to-gameplay runtime translation from animation graph commands into gameplay/kernel effects — deferred until the runtime/player milestone (M002).
- Why it matters: Avoids coupling the schema and validator milestone (M001) to runtime behavior that belongs to the next stage.
- Source: inferred
- Notes: Recovered from git history (commit 647a8af). Original R010 from M001 planning. Now addressed by M002. M001 defined command schema and validated references, but did not implement live gameplay translation.

### R031 — Complete per-Digimon blueprint migration for the whole roster (beyond Agumon).
- Class: differentiator
- Status: deferred
- Description: Complete per-Digimon blueprint migration for the whole roster (beyond Agumon).
- Why it matters: The M015 validated baseline explicitly tracked full roster blueprint migration as future work beyond the combat authority closure.
- Source: git-history 00c0812
- Notes: Recovered from git history. Original R014 from post-M015 planning. Deferred historical work item; no current proof required.

### R032 — Complete revised 12-Digimon roster behavior and balance validation.
- Class: quality-attribute
- Status: deferred
- Description: Complete revised 12-Digimon roster behavior and balance validation.
- Why it matters: Historical planning kept full roster behavior and balance validation out of the validated baseline while preserving it as explicit future work.
- Source: git-history 00c0812
- Notes: Recovered from git history. Original R015 from post-M015 planning.

### R033 — Full playable CLI UX and windowed presentation pipeline consuming canonical combat surfaces.
- Class: launchability
- Status: deferred
- Description: Full playable CLI UX and windowed presentation pipeline consuming canonical combat surfaces.
- Why it matters: The M015 contract distinguished validated combat authority from later playable UX and presentation delivery.
- Source: git-history 00c0812
- Notes: Recovered from git history. Original R016 from post-M015 planning. M002 delivers Agumon-only windowed proof; full playable UX/CLI is downstream.

### R034 — Integrate Roguelite Fatigue and the run-loop into the combat stack.
- Class: primary-user-loop
- Status: deferred
- Description: Integrate Roguelite Fatigue and the run-loop into the combat stack.
- Why it matters: The historical requirements snapshot preserved run-loop integration as a future capability beyond the validated combat baseline.
- Source: git-history 00c0812
- Notes: Recovered from git history. Original R017 from post-M015 planning.

### R035 — Complete boss conversion and hard-control policy integration.
- Class: integration
- Status: deferred
- Description: Complete boss conversion and hard-control policy integration.
- Why it matters: The historical contract kept boss conversion and hard-control policy as explicit pending integration work.
- Source: git-history 00c0812
- Notes: Recovered from git history. Original R018 from post-M015 planning.

### R036 — Complete the Heavy taxonomy.
- Class: core-capability
- Status: deferred
- Description: Complete the Heavy taxonomy.
- Why it matters: The historical requirements snapshot tracked Heavy taxonomy completion as unresolved future capability work.
- Source: git-history 00c0812
- Notes: Recovered from git history. Original R019 from post-M015 planning.

## Out of Scope

### R001 — Deprecated placeholder — ID slot consumed to preserve R003+ numbering alignment.
- Class: constraint
- Status: out-of-scope
- Description: Deprecated placeholder — ID slot consumed to preserve R003+ numbering alignment.
- Why it matters: ID alignment: R001 and R002 were renumbered into R021/R022 during a DB reset; these placeholder entries preserve the original ID sequence.
- Source: migration

### R002 — Deprecated placeholder — ID slot consumed to preserve R003+ numbering alignment.
- Class: constraint
- Status: out-of-scope
- Description: Deprecated placeholder — ID slot consumed to preserve R003+ numbering alignment.
- Why it matters: ID alignment: R001 and R002 were renumbered into R021/R022 during a DB reset; these placeholder entries preserve the original ID sequence.
- Source: migration

### R017 — Other 5 Digimon (the remaining roster beyond Agumon) — rendered, kitted, playable.
- Class: core-capability
- Status: out-of-scope
- Description: Other 5 Digimon (the remaining roster beyond Agumon) — rendered, kitted, playable.
- Why it matters: M002 is deliberately Agumon-only to retire the runtime-boundary risk once; the roster extension reuses the M002 seam and is sequenced into M003–M007.
- Source: user
- Notes: Out of scope for M002 — the per-skill graph + stance schema is designed so this is a data-only roster extension later.

### R018 — Custom RON VFX format + VFX editor (bevy_enoki / Omagari). The VFX Id handle seam is left open, but no RON VFX format or editor is implemented in M002.
- Class: anti-feature
- Status: out-of-scope
- Description: Custom RON VFX format + VFX editor (bevy_enoki / Omagari). The VFX Id handle seam is left open, but no RON VFX format or editor is implemented in M002.
- Why it matters: Prevents scope creep: M002 proves the runtime stack with Rust-configured placeholder VFX; a VFX authoring pipeline is a separate effort behind the opaque-handle seam.
- Source: user
- Notes: Seam open (opaque Id handle); implementation explicitly excluded from M002.

### R019 — Meta-loop / encounter chain / save-load / evolution / enemy AI / balance.
- Class: anti-feature
- Status: out-of-scope
- Description: Meta-loop / encounter chain / save-load / evolution / enemy AI / balance.
- Why it matters: These are downstream game-systems milestones; including any would dilute M002's single focus of getting combat on screen with a clean seam.
- Source: user
- Notes: M002 uses an Agumon dummy opponent, no AI; deferred to later milestones.

### R020 — Parallel graph topology (fork/join, parallel/orthogonal states). Concurrency is achieved only via fire-and-forget VFX entities on their own clock plus event-driven observers (GAS GameplayCue model), never via parallel graph nodes.
- Class: constraint
- Status: out-of-scope
- Description: Parallel graph topology (fork/join, parallel/orthogonal states). Concurrency is achieved only via fire-and-forget VFX entities on their own clock plus event-driven observers (GAS GameplayCue model), never via parallel graph nodes.
- Why it matters: True parallel tracks need a deterministic interleave scheduler (violates R004) and a much harder editor (violates KISS) — explicitly rejected so the seam stays simple and deterministic.
- Source: user
- Notes: If a real blocking-join case ever emerges, extend the closed enum deliberately then — not speculatively now.

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | constraint | out-of-scope | none | none | unmapped |
| R002 | constraint | out-of-scope | none | none | unmapped |
| R003 | quality-attribute | validated | none | none | cargo test --test clip_geometry_parity passes (1 test: agumon_clip_ron_matches_authoritative_atlas_geometry). clip.ron corrected to frame_size w=512/h=512, total_frames=93, ranges heavy_attack 23-45, hurt 46-52, idle 53-58, skill 59-75, victory 76-92 — matching agumon_atlas.json exactly. anim_graph.ron node frame references updated to match. Validated in M001 remediation before milestone completion (2026-05-19). |
| R004 | core-capability | active | M002/S01 | none | cargo run --features windowed shows Agumon cycling idle via the stance graph; M001 headless tests stay green; clip↔atlas geometry parity test present and passing. |
| R005 | core-capability | active | M002/S01 | none | Agumon stance graph loads as a distinct RON asset; skill Exit returns sprite to stance idle; target blink/hurt reactions driven by CombatEvent on the target's stance, never authored in the attacker's skill graph. |
| R006 | core-capability | active | M002/S02 | none | Sharp Claws windup→strike→recovery on screen; damage falls on the impact frame via ReleaseKernelCue; invariant I3 extended to the new handshake stays green (identical Intent stream headless vs windowed, only timing differs). |
| R007 | constraint | active | M002/S02 | none | A test fails if anim_graph.ron contains any gameplay Command (EmitDamage/EmitStatus/EmitHeal); the M001 mul:18 duplicate at agumon/anim_graph.ron:20 is remediated behind that test. |
| R008 | differentiator | active | M002/S01 | none | CompiledTimeline.id = skill_id confirmed (skill_timeline.rs:73); animation-side registry resolves skill-id→graph with zero if-else dispatch. |
| R009 | quality-attribute | validated | M002/S02 | none | Verified by cargo test --test animation anim_graph_input_purity and the windowed regression sweep. Tests prove AnimGraph evaluation uses a closed typed AnimGraphRole/AnimGraphInput seam, rejects stringly or unknown roles, and keeps player advancement behaviorally equivalent without any world-global or mutable graph-context read path. |
| R010 | primary-user-loop | active | M002/S03 | none | Phase strip updates from EventReader<CombatEvent>; a structural test asserts the UI path never mutates combat state. |
| R011 | core-capability | active | M002/S05 | none | Agumon vs dummy Agumon at full kit on screen; multi-hit loop visibly = kernel hop count; target blink/hurt driven by CombatEvent. |
| R012 | integration | active | M002/S04 | none | Baby Burner reactive detonate + flash VFX via Rust-configured entity; no custom RON/editor; swapping the backing implementation would not touch anim graph or kernel. |
| R013 | failure-visibility | validated | M002/S02 | none | Verified by cargo test --test timeline r013_failure_visibility, cargo test --test animation anim_registry_failure_visibility, and cargo test --features windowed --test animation --test timeline --test windowed_only. Tests prove cue timeout force-resume with structured diagnostics, missing skill graph runtime fallback plus boot-time load-state visibility, hot reload applying only to newly spawned/resolved players, and dead-target mid-loop remaining observable without branching presentation flow on liveness. |
| R014 | launchability | active | M002/S06 | none | Operational UAT with captured console output (not just a documented procedure): a windowed session showing no panic, stable FPS, and hot-reload mid-skill leaving world state intact. |
| R015 | operability | active | M002/S06 | none | A repomix-grounded architectural review report (maintainability/scalability/extensibility) is produced at closeout and attached as S06 evidence; findings are triaged before milestone completion. |
| R016 | constraint | active | M002/S06 | none | cargo test green (M001 suite intact); extended I3 parity test green; no winit/wgpu/egui deps outside windowed; no .md added to repo root. |
| R017 | core-capability | out-of-scope | none | none | unmapped |
| R018 | anti-feature | out-of-scope | none | none | unmapped |
| R019 | anti-feature | out-of-scope | none | none | unmapped |
| R020 | constraint | out-of-scope | none | none | unmapped |
| R021 | quality-attribute | validated | M001/S01 | none | Validated in M001: animation module boundary established in src/animation/ with no Digimon-specific logic in the core module. |
| R022 | core-capability | validated | M001/S01 | none | Validated in M001: anim_graph.ron loads as typed Bevy asset via AnimationAssetPlugin; closed RON schema types in src/animation/anim_graph.rs. |
| R023 | continuity | validated | M001/S03 | none | Validated in M001: boot-time validation with typed diagnostics in src/animation/validation/; names offending file/check where practical. |
| R024 | integration | validated | M001/S03 | none | Validated in M001: cross-asset validation via adapter seams; parameter references, clip names and other catalogs supplied by adapters, not hardcoded in the validator. |
| R025 | launchability | validated | M001/S04 | none | Validated in M001: manual windowed hot-reload demo completed; invalid hot reload keeps last valid asset and logs clearly. |
| R026 | quality-attribute | validated | M001/S04 | none | Validated in M001: Renamon animation assets authored and validated through the same generic path; no Agumon-specific assumptions in the loader. |
| R027 | constraint | validated | M001/S03 | M001/S01, M001/S02, M001/S04 | Validated in M001: all validation runs headless; windowed gated by #[cfg(feature = windowed)]; no winit/wgpu/egui deps outside the feature gate. |
| R028 | core-capability | validated | none | none | Validated baseline from recovered historical requirements snapshot; evidence in .gsd/milestones/M015/M015-VALIDATION.md, docs/combat_current.md, and M015 contract docs. Validated requirements: R086, R088, R089-R100 (historical numbering). |
| R029 | core-capability | deferred | none | none | unmapped |
| R030 | integration | deferred | none | none | unmapped |
| R031 | differentiator | deferred | none | none | unmapped |
| R032 | quality-attribute | deferred | none | none | unmapped |
| R033 | launchability | deferred | none | none | unmapped |
| R034 | primary-user-loop | deferred | none | none | unmapped |
| R035 | integration | deferred | none | none | unmapped |
| R036 | core-capability | deferred | none | none | unmapped |

## Coverage Summary

- Active requirements: 11
- Mapped to slices: 11
- Validated: 11 (R003, R009, R013, R021, R022, R023, R024, R025, R026, R027, R028)
- Unmapped active requirements: 0
