---
estimated_steps: 3
estimated_files: 3
skills_used: []
---

# T05: Baby Burner primary timeline + animation graph + thermal stack on impact

Why: `agumon_ult` (`Baby Burner`) has `implementation: Implemented` but no `timeline` field and no animation graph nodes, so the Ultimate button in the egui combat panel produces no visible beats — the kit demo cannot land its Ultimate.

Do: (1) In `assets/data/digimon/agumon/skills.ron`, add a `timeline` to `agumon_ult` with three beats: `windup` (presentation-only), `impact` (BeatPayload::DealDamage 50 + BeatPayload::BreakToughness 30 + emit the existing `apply_thermal_spark` custom signal with payload Amount(3) + presentation barrier metadata), and `recovery` (presentation-only). Keep `legacy_ops` and `custom_signals` consistent: prefer migrating the damage/toughness/signal effects into the timeline and shrinking `legacy_ops` only if it does not break `tests/data_skills_ron_validation.rs` and friends — otherwise leave `legacy_ops` in place and rely on the timeline path for the visible beats, mirroring how Sharp Claws coexists in `unit.ron` today (executor: pick whichever path keeps the existing skill-validation suite green; document the choice in the task summary). (2) In `assets/digimon/agumon/anim_graph.ron`, add `baby_burner_charge`, `baby_burner_launch`, and `baby_burner_recovery` nodes wired through a windup→launch (`Predicate::KernelCue`)→recovery (`Predicate::KernelCue`) chain. Reuse the existing attack/skill atlas frames per MEM037 (anim_graph stays on `clip: "all"`); do not add gameplay numbers to graph data. (3) Verify the existing reactive Baby Burner detonate seam (`src/combat/blueprints/agumon/baby_burner.rs`) still fires unchanged on a lethal Heated KO — no code changes expected in that file. (4) Add `tests/agumon_baby_burner_primary.rs` asserting: timeline parses cleanly; running the timeline produces the windup→impact→recovery beat sequence; impact applies 50 damage, BreakToughness(30), and emits one `apply_thermal_spark` custom signal carrying amount=3; the reactive detonate still triggers when the primary target dies Heated; headless and windowed runners both reach `Done`. (5) Keep `tests/agumon_baby_burner_reactive.rs`, `tests/data_skills_ron_validation.rs`, `tests/data_skills_ron_roundtrip.rs`, `tests/anim_graph_asset.rs`, `tests/anim_player_fsm.rs`, `tests/anim_gameplay_command_forbidden.rs`, and `tests/clip_atlas_parity.rs` all green.

Done-when: the new test passes; the listed regression tests stay green; `cargo build --features windowed` and `cargo build --no-default-features` both pass.

## Inputs

- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/data/digimon/agumon/unit.ron`
- `src/combat/runtime/timeline.rs`
- `src/combat/runtime/runner.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/agumon/baby_burner.rs`
- `src/animation/player.rs`
- `src/animation/anim_graph.rs`
- `tests/agumon_baby_burner_reactive.rs`
- `tests/agumon_sharp_claws_asset.rs`
- `tests/data_skills_ron_validation.rs`
- `tests/anim_graph_asset.rs`

## Expected Output

- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/agumon_baby_burner_primary.rs`

## Verification

cargo test --test agumon_baby_burner_primary --test agumon_baby_burner_reactive --test data_skills_ron_validation --test data_skills_ron_roundtrip --test anim_graph_asset --test anim_player_fsm --test anim_gameplay_command_forbidden --test clip_atlas_parity

## Observability Impact

Ultimate now produces real `OnCombatBeat`/`OnDamageDealt`/`OnHitTaken` events plus the existing `apply_thermal_spark` `BlueprintSignal`, so the phase strip, floating damage numbers, target-hurt projection (T03), and reactive detonate flash (S04) all light up from the kit demo's visible Ultimate.
