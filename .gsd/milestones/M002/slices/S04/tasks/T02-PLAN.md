---
estimated_steps: 13
estimated_files: 4
skills_used: []
---

# T02: Register Agumon Baby Burner detonate with headless tests

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

Do: Add Agumon-specific Baby Burner logic under `src/combat/blueprints/agumon/` and register it through the seam from T01. Trigger only when `skill_id == SkillId("agumon_ult")`, the primary target died in that same cast, and `heated_remaining > 0`. Resolve adjacent alive enemies using existing slot-index/`TargetShape::Blast` semantics or a pure helper built from `TargetableSnapshot`, then exclude the dead primary and any KO/non-adjacent targets. Enqueue deterministic Fire `DealDamage` intents at `8 * heated_remaining` per adjacent target and emit a registered generic blueprint signal/transition named `baby_burner_detonate` for each actual detonation target; prefer existing generic payloads such as `SignalPayload::UnitTarget(target)` unless implementation requires an equally generic payload extension. Register the signal in `SignalTaxonomy` during Agumon runtime setup.

Add `tests/agumon_baby_burner_reactive.rs` covering lethal Heated Baby Burner detonates both adjacent alive enemies once; primary is not hit by detonate; non-lethal Baby Burner does not detonate; lethal non-Baby-Burner does not detonate; zero Heated payload does not detonate; repeated `app.update()`/duplicate release-like no-op does not duplicate detonate; `OnKernelTransition::Blueprint` is emitted exactly for real detonate targets.

Done when: the new test file passes and existing UnitDied payload coverage still proves Heated payload preservation.

## Inputs

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/agumon/signals.rs`
- `src/combat/resolution/types.rs`
- `src/combat/runtime/post_action.rs`
- `src/combat/runtime/signal.rs`
- `src/combat/runtime/applier/mod.rs`
- `assets/data/digimon/agumon/skills.ron`
- `tests/common/app.rs`
- `tests/unit_died_payload.rs`

## Expected Output

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/agumon/baby_burner.rs`
- `tests/agumon_baby_burner_reactive.rs`
- `tests/common/app.rs`

## Verification

cargo test --test agumon_baby_burner_reactive --test unit_died_payload

## Observability Impact

Adds a deterministic detonate transition that lets headless tests and future windowed code identify exactly when reactive detonate fired.
