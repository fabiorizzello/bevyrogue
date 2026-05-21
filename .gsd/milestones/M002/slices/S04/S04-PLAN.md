# S04: Baby Burner reactive detonate + flash VFX

**Goal:** Deliver a deterministic Rust-only Baby Burner reactive detonate proof: when Agumon's `agumon_ult` kills a Heated primary target, adjacent alive enemies take deterministic detonate damage exactly once and a feature-gated windowed flash indicator projects the generic combat transition without mutating combat state.
**Demo:** Baby Burner reactive detonate with a flash VFX (Rust code, no RON/editor); zero non-determinism, R004 intact, headless tests unchanged.

## Must-Haves

- Owned requirements: R002 headless-first cue logic, R004 deterministic suspension/resume semantics, R005 feature-gated windowed surface, R006 fresh verification. Supporting requirements: R003 asset/clip parity remains unchanged.
- Must-haves:
- `agumon_ult` lethal hit on a Heated primary target detonates adjacent alive enemies only, excludes the dead primary, and applies `8 * heated_remaining` Fire damage unless implementation evidence forces a narrower scalar.
- Non-lethal Baby Burner, lethal non-Baby-Burner actions, zero Heated stacks, duplicate/repeated update ticks, and already-KO/non-adjacent targets do not detonate.
- Shared combat/runtime seam remains owner-neutral: no Agumon/Baby Burner strings in generic runtime or turn-system branching beyond registry IDs supplied by blueprint registration.
- Detonate emits a presentation-observable generic signal/transition such as `OnKernelTransition::Blueprint { owner: "agumon", name: "baby_burner_detonate", ... }` exactly once per flashed adjacent target.
- Windowed flash is presentation-only, behind `feature = "windowed"`, and verified without opening a display.
- Threat Surface (Q3): no auth, network, filesystem, or secret exposure. Input trust is local code/assets only; malformed or unregistered blueprint signals must fail visibly through taxonomy warnings/debug assertions rather than silently mutating combat.
- Requirement Impact (Q4): re-verify R002/R004 timeline barrier tests, R005 windowed build/test, and R003 clip/graph parity. Decision D026 locks Rust-side post-application reaction for this slice while remaining revisable.
- Failure Modes (Q5): missing skill context could detonate any Heated kill; missing duplicate guard could double-apply on repeated signal dispatch; missing signal taxonomy registration could drop the flash transition; scheduling after action resolution could make tests observe stale HP/events.
- Load Profile (Q6): per Baby Burner cast cost is O(unit_count) target snapshot plus O(adjacent_count) damage intents/transitions; the 10x breakpoint is event spam if detonate is emitted per non-adjacent unit instead of per actual adjacent target.
- Negative Tests (Q7): non-lethal primary, wrong skill, zero Heated payload, duplicate dispatch/update, dead/non-adjacent enemies, and released/no-awaiting timeline regressions.

## Proof Level

- This slice proves: Integration proof. Real runtime required: yes for Bevy ECS/headless tests and feature-gated windowed helper tests. Human/UAT required: no; optional live window smoke may remain environment-dependent.

## Integration Closure

Consumes S02's deterministic cue-barrier/runtime contract and existing `UnitDied { heated_remaining }` event payload. Introduces one owner-neutral post-application reaction seam, one Agumon blueprint registration, and one windowed presentation projection. Leaves S05 to assemble the full Agumon-vs-Agumon kit and target blink/hurt polish; S04 itself closes on headless detonate correctness plus feature-gated flash proof.

## Verification

- Verification:
- `cargo test --test agumon_baby_burner_reactive`
- `cargo test --test unit_died_payload`
- `cargo test --test timeline_cue_barrier_pipeline`
- `cargo test --test timeline_two_clock_parity`
- `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity`
- `cargo test --features windowed --test windowed_preview_cache`
- `cargo test --lib`
- `cargo build --no-default-features`
- `cargo build --features windowed`
- Observability / Diagnostics:
- Runtime signals: `UnitDied`, detonate `OnDamageDealt`, and `OnKernelTransition::Blueprint(owner=agumon,name=baby_burner_detonate,...)`.
- Inspection surfaces: `tests/agumon_baby_burner_reactive.rs` event assertions and feature-gated flash helper/resource state.
- Failure visibility: target/cast/signal details in transition payload or tooltip/resource state so future agents can distinguish missing reaction, dropped signal, and presentation-only failure.
- Redaction constraints: none; combat state contains no secrets.

## Tasks

- [x] **T01: Wire an owner-neutral post-KO reaction seam** `est:2h`
  ---
  estimated_steps: 7
  estimated_files: 5
  skills_used:
    - rust-best-practices
    - tdd
    - bevy
  ---
  Why: Baby Burner needs skill/cast/KO context immediately after the primary hit is committed, but shared combat code must stay Digimon-free and existing timeline/AnimGraph KernelEvent branching is not viable for S04.
  - Files: `src/combat/runtime/post_action.rs`, `src/combat/runtime/registry.rs`, `src/combat/runtime/mod.rs`, `src/combat/turn_system/pipeline/paths/single_target.rs`, `tests/registry_internals.rs`
  - Verify: cargo test --test unit_died_payload --test timeline_cue_barrier_pipeline

- [x] **T02: Register Agumon Baby Burner detonate with headless tests** `est:3h`
  ---
  estimated_steps: 9
  estimated_files: 4
  skills_used:
    - rust-best-practices
    - rust-testing
    - tdd
    - bevy
  ---
  Why: The slice's source-of-truth behavior is headless combat correctness, not a windowed particle. This task proves the reactive rule and its negative cases before presentation polish.
  - Files: `src/combat/blueprints/agumon/mod.rs`, `src/combat/blueprints/agumon/baby_burner.rs`, `tests/agumon_baby_burner_reactive.rs`, `tests/common/app.rs`
  - Verify: cargo test --test agumon_baby_burner_reactive --test unit_died_payload

- [x] **T03: Project detonate transitions into a windowed flash indicator** `est:2h`
  ---
  estimated_steps: 7
  estimated_files: 4
  skills_used:
    - bevy
    - make-interfaces-feel-better
    - rust-testing
  ---
  Why: S04 needs a visible flash proof while preserving R005: presentation must stay behind `feature = "windowed"` and must not drive combat damage.
  - Files: `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/labels.rs`, `src/ui/combat_panel/render.rs`, `tests/windowed_preview_cache.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache

- [x] **T04: Run S04 regression matrix and document live-smoke limits** `est:1h`
  ---
  estimated_steps: 5
  estimated_files: 0
  skills_used:
    - verify-before-complete
    - rust-testing
  ---
  Why: S04 touches combat reactions, events, and feature-gated UI; closeout must prove it did not regress R002/R003/R004/R005.
  - Verify: cargo test --test agumon_baby_burner_reactive --test unit_died_payload --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity

## Files Likely Touched

- src/combat/runtime/post_action.rs
- src/combat/runtime/registry.rs
- src/combat/runtime/mod.rs
- src/combat/turn_system/pipeline/paths/single_target.rs
- tests/registry_internals.rs
- src/combat/blueprints/agumon/mod.rs
- src/combat/blueprints/agumon/baby_burner.rs
- tests/agumon_baby_burner_reactive.rs
- tests/common/app.rs
- src/ui/combat_panel/mod.rs
- src/ui/combat_panel/labels.rs
- src/ui/combat_panel/render.rs
- tests/windowed_preview_cache.rs
